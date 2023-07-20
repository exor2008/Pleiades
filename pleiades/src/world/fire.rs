use super::OnDirection;
use crate::apds9960::Direction;
use crate::color::ColorGradient;
use crate::led_matrix;
use crate::perlin;
use crate::world::{Flush, Tick};
use crate::ws2812::Ws2812;
use core::cmp::{max, min};
use embassy_rp::clocks::RoscRng;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Ticker};
use heapless::Vec;
use pleiades_macro_derive::{Flush, From, Into};
use rand::Rng;
use smart_leds::hsv::Hsv;

#[derive(Flush, Into, From)]
pub struct Fire<'a, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    noise: perlin::PerlinNoise,
    colormap: ColorGradient<4>,
    height: Height<L, 1, 3, 15>,
    sparks: Vec<Spark<C>, C>,
    ticker: Ticker,
    t: usize,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let noise = perlin::PerlinNoise::new();
        let colormap = Fire::<P, S, L, C, N>::get_colormap();
        let height = Height::new(6);
        let ticker = Ticker::every(Duration::from_millis(50));
        let sparks: Vec<Spark<C>, C> = Vec::new();

        Self {
            led,
            noise,
            colormap,
            height,
            sparks,
            ticker,
            t: 0,
        }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Tick
    for Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    async fn tick(&mut self) {
        self.led.clear();

        for x in 0..C {
            // Generate noise for fire shape
            let xx = x as f32 / 2.6;
            let yy = self.t as f32 / 10.0;
            let noise = self.noise.get2d([xx, yy]);
            let noise = (noise - 0.3) / 0.25; // [0..1]
            let noise = noise.clamp(0.0, 1.0);

            //Determine the height of fire pillar
            let height = (noise * (L - self.height.value()) as f32) as usize;
            let height = max(2, height);

            // Process the sparks
            self.spawn_spark(x, height);

            // Color every fire pillar pixel
            // and write it to buffer
            for i in C - height..C {
                let temp = (C - i - 1) as f32 / (height - 1) as f32;
                let color = self.colormap.get_noised(temp, -0.1, 0.1);
                self.led.write(x, i, color);
            }
        }
        self.process_sparks();
        self.draw_sparks();

        self.t = self.t.wrapping_add(1);
        self.ticker.next().await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn spawn_spark(&mut self, x: usize, height: usize) {
        if height < (C - 1) && perlin::spawn_chance(1, 300) {
            let spark = Spark {
                x: x as isize,
                y: (C - 1 - height) as isize,
            };
            // Do not spawn spark if it's already too many sparks
            if let Err(_) = self.sparks.push(spark) {}
        }
    }

    fn process_sparks(&mut self) {
        self.sparks.iter_mut().for_each(|spark| spark.up());
        self.sparks
            .retain(|spark| (spark.x >= 0) && (spark.x < C as isize) && (spark.y >= 0));
    }

    fn draw_sparks(&mut self) {
        let mut rng = RoscRng;
        let temp = rng.gen_range(0.8f32..=1.0);

        for spark in self.sparks.iter() {
            let color = self.colormap.get_noised(temp, -0.2, 0.2);
            self.led.write(spark.x as usize, spark.y as usize, color);
        }
    }

    fn get_colormap() -> ColorGradient<4> {
        let pos = [0.0, 0.2, 0.8, 1.01];
        let hsv = [
            Hsv {
                hue: 0,
                sat: 255,
                val: 48,
            },
            Hsv {
                hue: 1,
                sat: 255,
                val: 100,
            },
            Hsv {
                hue: 1,
                sat: 255,
                val: 150,
            },
            Hsv {
                hue: 9,
                sat: 255,
                val: 200,
            },
        ];
        ColorGradient::from_hsv(pos, hsv)
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> OnDirection
    for Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn on_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.colormap.change_value(20);
                self.height.up();
            }
            Direction::Down => {
                self.colormap.change_value(-20);
                self.height.down();
            }
        }
    }
}

#[derive(Debug)]
struct Spark<const C: usize> {
    x: isize,
    y: isize,
}

impl<const C: usize> Spark<C> {
    fn up(&mut self) {
        let mut rng = RoscRng;
        let dir: isize = rng.gen_range(-1..=2);

        self.y -= 1;
        self.x += dir;
    }
}

struct Height<const L: usize, const COOLDOWNL: u8, const MIN: usize, const MAX: usize> {
    value: usize,
    cooldown: u8,
}

impl<const L: usize, const COOLDOWNL: u8, const MIN: usize, const MAX: usize>
    Height<L, COOLDOWNL, MIN, MAX>
{
    fn new(value: usize) -> Self {
        Height { value, cooldown: 0 }
    }

    fn down(&mut self) {
        match self.cooldown == 0 {
            true => {
                self.cooldown = COOLDOWNL;
                self.value += 1;
                self.value = min(self.value, MAX);
            }
            false => {
                self.cooldown -= 1;
            }
        }
    }

    fn up(&mut self) {
        match self.cooldown == 0 {
            true => {
                self.cooldown = COOLDOWNL;
                self.value -= 1;
                self.value = max(self.value, MIN);
            }
            false => {
                self.cooldown -= 1;
            }
        }
    }

    fn value(&self) -> &usize {
        &self.value
    }
}
