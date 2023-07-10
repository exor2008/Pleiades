use crate::led_matrix;
use crate::perlin;

use crate::color::{Color, ColorGradient};
use crate::ws2812::Ws2812;
use core::iter::Sum;
// use core::ops::Add;
use embassy_rp::clocks::RoscRng;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Ticker};
use rand::Rng;

use heapless::Vec;
use pleiades_macro_derive::{Flush, From, Into};

use smart_leds::RGB8;

use crate::world::{Flush, Tick};

const PATTERNS: usize = 6;

#[derive(Flush, Into, From)]
pub struct NorthenLight<
    'a,
    P: Instance,
    const S: usize,
    const L: usize,
    const C: usize,
    const N: usize,
> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    colormap: ColorGradient<C>,
    ticker: Ticker,
    patterns: Vec<Pattern<L, C, N>, PATTERNS>,
    t: usize,
    last_spawn: isize,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize>
    NorthenLight<'a, P, S, L, C, N>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let ticker = Ticker::every(Duration::from_millis(10));
        let mut colormap = ColorGradient::new();

        colormap.add_color(Color::new(0.0, RGB8::new(0, 0, 0)));
        colormap.add_color(Color::new(0.1, RGB8::new(0, 0, 0)));
        colormap.add_color(Color::new(0.25, RGB8::new(10, 30, 60)));
        colormap.add_color(Color::new(0.5, RGB8::new(2, 237, 80)));
        colormap.add_color(Color::new(0.75, RGB8::new(108, 134, 206)));
        colormap.add_color(Color::new(1.01, RGB8::new(70, 30, 100)));

        let patterns: Vec<Pattern<L, C, N>, PATTERNS> = Vec::new();

        Self {
            led,
            colormap,
            ticker,
            patterns,
            t: 0,
            last_spawn: -1000,
        }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Tick
    for NorthenLight<'a, P, S, L, C, N>
where
    P: Instance,
{
    async fn tick(&mut self) {
        self.led.clear();

        self.spawn_patterns();
        self.process_patterns();
        let sum_pattern: Pattern<L, C, N> = self.patterns.iter().sum();

        for index in 0..N {
            let temperature = sum_pattern.data()[index];
            let color = self.colormap.get(temperature, false);
            self.led.write_straight(index, color);
        }

        self.remove_obsolete_patterns();

        self.t = self.t.wrapping_add(1);
        self.led.flush().await;
        self.ticker.next().await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize>
    NorthenLight<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn spawn_patterns(&mut self) {
        let time_till_last_spawn = self.t as isize - self.last_spawn;
        if !self.patterns.is_full() && time_till_last_spawn > 50 {
            let cutoff = rand_noise(0.47, 0.55);
            let lifetime = 300 + rand_noise(-100.0, 100.0) as usize;
            let pattern: Pattern<L, C, N> = Pattern::new(self.t, cutoff, lifetime);
            self.patterns.push(pattern).unwrap();
            self.last_spawn = self.t as isize;
        }
    }

    fn process_patterns(&mut self) {
        self.patterns.iter_mut().for_each(|pattern| pattern.tick());
    }

    fn remove_obsolete_patterns(&mut self) {
        self.patterns
            .retain(|pattern| pattern.t <= pattern.lifetime);
    }
}

#[derive(Debug)]
struct Pattern<const L: usize, const C: usize, const N: usize> {
    data: [f32; N],
    lifetime: usize,
    t: usize,
}

impl<const L: usize, const C: usize, const N: usize> Pattern<L, C, N> {
    pub fn new(t: usize, cutoff: f32, lifetime: usize) -> Self {
        let noise = perlin::PerlinNoise::new();
        let data = Self::fill(noise, t, cutoff);
        Self {
            data,
            lifetime,
            t: 0,
        }
    }

    fn index(x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * L + y,
            false => x * L + (L - y) - 1,
        }
    }

    fn fill(noise: perlin::PerlinNoise, t: usize, cutoff: f32) -> [f32; N] {
        let mut data = [f32::default(); N];
        let shift = rand_noise(0.1, 0.8);

        for x in 0..C {
            for y in 0..L {
                // Generate noise for northen light
                let xx = (x.wrapping_add(t)) as f64 / 5.0;
                let yy = (y.wrapping_add(t)) as f64 / 5.0;
                // let zz = t as f32;

                let noise = noise.get2d([xx, yy]) as f32;
                let noise = noise - cutoff;
                let noise = if noise <= 0.0 {
                    0.0
                } else {
                    (noise + shift).min(1.0)
                };
                let index = Self::index(x, y);
                data[index] = noise;
            }
        }
        data
    }

    fn data(&self) -> &[f32; N] {
        &self.data
    }

    fn impact(&self, index: usize) -> f32 {
        self.data[index] * self.coef()
    }

    fn coef(&self) -> f32 {
        let t = self.t as f32;
        let lifetime = self.lifetime as f32;

        match t {
            t if t <= lifetime * 0.5 => t / (lifetime * 0.5),
            t => 1.0 - (t - lifetime * 0.5) / (lifetime * 0.5),
        }
    }

    fn tick(&mut self) {
        self.t += 1;
    }
}

impl<'a, const L: usize, const C: usize, const N: usize> Sum<&'a Pattern<L, C, N>>
    for Pattern<L, C, N>
{
    fn sum<I: Iterator<Item = &'a Pattern<L, C, N>>>(iter: I) -> Self {
        let mut sum_data: [f32; N] = [0.0; N];

        for item in iter {
            for i in 0..N {
                sum_data[i] = sum_data[i] + item.impact(i);
            }
        }
        for i in 0..N {
            sum_data[i] = sum_data[i].clamp(0.0, 1.0)
        }
        Pattern::from(sum_data)
    }
}

impl<const L: usize, const C: usize, const N: usize> From<[f32; N]> for Pattern<L, C, N> {
    fn from(data: [f32; N]) -> Self {
        Self {
            data,
            lifetime: 0,
            t: 0,
        }
    }
}

pub fn rand_noise(min: f32, max: f32) -> f32 {
    let mut rng = RoscRng;
    rng.gen_range(min..max)
}
