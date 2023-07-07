use crate::led_matrix;
use crate::perlin;

use crate::color::{Color, ColorGradient};
use crate::ws2812::Ws2812;
use core::cmp::max;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Timer};
use smart_leds::RGB8;

pub trait Tick {
    async fn tick(&mut self);
}

pub trait Flush {
    async fn flush(&mut self);
}

pub struct Fire<'a, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    noise: perlin::PerlinNoise,
    colormap: ColorGradient<C>,
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

        colormap.add_color(Color::new(0.0, RGB8::new(50, 0, 5)));
        colormap.add_color(Color::new(0.2, RGB8::new(141, 5, 0)));
        colormap.add_color(Color::new(0.8, RGB8::new(230, 10, 0)));
        colormap.add_color(Color::new(1.1, RGB8::new(230, 25, 0)));

        Self {
            led,
            noise,
            colormap,
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
            let xx = x as f64 / 2.6;
            let yy = self.t as f64 / 10.0;
            let noise = self.noise.get2d([xx, yy]);
            let noise = (noise - 0.3) / 0.25; // [0..1]
            let noise = noise.clamp(0.0, 1.0);
            let height = (noise * (C - 6) as f64) as usize;
            let height = max(2, height);

            for i in C - height..C {
                let temp = (C - i - 1) as f32 / (height - 1) as f32;
                let color = self.colormap.get(temp);
                self.led.write(x, i, color);
            }
        }

        self.t = self.t.wrapping_add(1);
        self.led.flush().await;
        Timer::after(Duration::from_millis(1)).await;
    }
}

//TODO: Derive macro
impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Flush
    for Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    async fn flush(&mut self) {
        self.led.flush().await;
    }
}

//TODO: Derive macro
impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize>
    Into<Ws2812<'a, P, S, N>> for Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn into(self) -> Ws2812<'a, P, S, N> {
        self.led.into()
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize>
    From<Ws2812<'a, P, S, N>> for Fire<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn from(ws: Ws2812<'a, P, S, N>) -> Self {
        Self::new(ws)
    }
}
