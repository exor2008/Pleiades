use crate::led_matrix;
use crate::perlin;

use crate::color::{Color, ColorGradient};
use crate::ws2812::Ws2812;
use core::cmp::max;
use embassy_rp::clocks::RoscRng;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use pleiades_macro_derive::{Flush, From, Into};
use rand::Rng;
use smart_leds::RGB8;

use crate::world::{Flush, Tick};

#[derive(Flush, Into, From)]
pub struct Fire<'a, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    noise: perlin::PerlinNoise,
    colormap: ColorGradient<C>,
    sparks: Vec<Spark<C>, C>,
    t: usize,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let noise = perlin::PerlinNoise::new();
        let mut colormap = ColorGradient::new();
        let sparks: Vec<Spark<C>, C> = Vec::new();

        colormap.add_color(Color::new(0.0, RGB8::new(50, 0, 5)));
        colormap.add_color(Color::new(0.2, RGB8::new(141, 5, 0)));
        colormap.add_color(Color::new(0.8, RGB8::new(230, 10, 0)));
        colormap.add_color(Color::new(1.1, RGB8::new(230, 25, 0)));

        Self {
            led,
            noise,
            colormap,
            sparks,
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
            let xx = x as f64 / 2.6;
            let yy = self.t as f64 / 10.0;
            let noise = self.noise.get2d([xx, yy]);
            let noise = (noise - 0.3) / 0.25; // [0..1]
            let noise = noise.clamp(0.0, 1.0);

            //Determine the height of fire pillar
            let height = (noise * (C - 6) as f64) as usize;
            let height = max(2, height);

            // Process the sparks
            self.spawn_spark(x, height);

            // Color every fire pillar pixel
            // and write it to buffer
            for i in C - height..C {
                let temp = (C - i - 1) as f32 / (height - 1) as f32;
                let color = self.colormap.get(temp);
                self.led.write(x, i, color);
            }
        }
        self.process_sparks();
        self.draw_sparks();

        self.t = self.t.wrapping_add(1);
        self.led.flush().await;
        Timer::after(Duration::from_millis(10)).await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn spawn_spark(&mut self, x: usize, height: usize) {
        if height < (C - 1) && self.spawn_chanse() {
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

    fn spawn_chanse(&self) -> bool {
        let mut rng = RoscRng;
        rng.gen_ratio(1, 700)
    }

    fn draw_sparks(&mut self) {
        let mut rng = RoscRng;
        let temp = rng.gen_range(0.7f32..=1.0);

        for spark in self.sparks.iter() {
            let color = self.colormap.get(temp);
            self.led.write(spark.x as usize, spark.y as usize, color);
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
        let dir: isize = rng.gen_range(-1..=1);

        self.y -= 1;
        self.x += dir;
    }
}
