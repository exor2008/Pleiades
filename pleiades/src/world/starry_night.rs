use super::OnDirection;
use crate::apds9960::Direction;
use crate::color::{Color, ColorGradient};
use crate::world::utils::CooldownValue;
use crate::world::{Flush, Tick};
use crate::{led_matrix::WritableMatrix, perlin};
use embassy_time::{Duration, Ticker};
use heapless::Vec;
use pleiades_macro_derive::Flush;
use smart_leds::RGB8;

const STARS_COLORS: usize = 7;
const STARS: usize = 5;
const INIT_STARS: usize = 1;
const FRAMES_INIT: usize = 25;
const FRAMES_MIN: usize = 10;
const FRAMES_MAX: usize = 30;
const FRAMES_COOLDOWN: u8 = 1;
const STAR_SPAWN_COOLDOWN: usize = 10;
const Y_COOLDOWN: usize = 1;

#[derive(Flush)]
pub struct StarryNight<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> {
    led: &'led mut Led,
    stars_colormap: ColorGradient<STARS_COLORS>,
    stars: Vec<Star<C, L>, STARS>,
    ticker: Ticker,
    buffer_new: [[RGB8; L]; C],
    buffer_old: [[RGB8; L]; C],
    buffer_space: [[RGB8; L]; C],
    since_star_spawn: usize,
    frames: CooldownValue<FRAMES_COOLDOWN, FRAMES_MIN, FRAMES_MAX>,
    t: usize,
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize>
    StarryNight<'led, Led, C, L, N>
{
    pub fn new(led: &'led mut Led) -> Self {
        let ticker = Ticker::every(Duration::from_millis(50));
        let stars_colormap = Self::get_stars_colormap();
        let buffer_space = Self::get_bg();
        let buffer_new = buffer_space;
        let buffer_old = buffer_space;

        let since_star_spawn = 0;

        let mut stars: Vec<Star<C, L>, STARS> = Vec::new();
        let frames = CooldownValue::new(FRAMES_INIT);

        for _ in 0..INIT_STARS {
            stars.push(Star::new()).unwrap();
        }

        Self {
            led,
            stars_colormap,
            ticker,
            stars,
            buffer_new,
            buffer_old,
            buffer_space,
            since_star_spawn,
            frames,
            t: 0,
        }
    }

    fn step(&mut self) -> [[RGB8; L]; C] {
        let mut buffer = self.buffer_space;
        self.spawn_stars();
        self.process_stars();
        self.draw_stars(&mut buffer);
        self.remove_stars();

        buffer
    }

    fn get_stars_colormap() -> ColorGradient<STARS_COLORS> {
        let mut stars_colormap = ColorGradient::new();

        stars_colormap.add_color(Color::new(0.0, RGB8::new(133, 152, 205) / 5));
        stars_colormap.add_color(Color::new(0.16, RGB8::new(221, 148, 133) / 5));
        stars_colormap.add_color(Color::new(0.33, RGB8::new(139, 195, 230) / 5));
        stars_colormap.add_color(Color::new(0.5, RGB8::new(188, 146, 183) / 5));
        stars_colormap.add_color(Color::new(0.66, RGB8::new(186, 244, 251) / 5));
        stars_colormap.add_color(Color::new(0.83, RGB8::new(234, 211, 194) / 5));
        stars_colormap.add_color(Color::new(1.01, RGB8::new(220, 221, 225) / 5));

        stars_colormap
    }

    fn get_bg() -> [[RGB8; L]; C] {
        let mut buffer: [[smart_leds::RGB<u8>; L]; C] = [[RGB8::default(); L]; C];
        let mut bg_colormap: ColorGradient<3> = ColorGradient::new();
        bg_colormap.add_color(Color::new(0.0, RGB8::new(0, 0, 0)));
        bg_colormap.add_color(Color::new(0.8, RGB8::new(1, 2, 3)));
        bg_colormap.add_color(Color::new(1.01, RGB8::new(3, 1, 3)));

        let noise = perlin::PerlinNoise::new();
        let rnd = perlin::rand_uint(0, 100) as usize;
        let shift = perlin::rand_float(0.1, 0.8);

        for (x, buffer) in buffer.iter_mut().enumerate().take(C) {
            for (y, buffer) in buffer.iter_mut().enumerate().take(L) {
                let xx = (x.wrapping_add(rnd)) as f32 / 5.0;
                let yy = (y.wrapping_add(rnd)) as f32 / 5.0;

                let noise = noise.get2d([xx, yy]) as f32;
                let noise = noise - 0.45;
                let noise = if noise <= 0.0 {
                    0.0
                } else {
                    (noise + shift).min(1.0)
                };

                *buffer = bg_colormap.get_noised(noise, -0.1, 0.1)
            }
        }

        buffer
    }
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> Tick
    for StarryNight<'led, Led, C, L, N>
{
    async fn tick(&mut self) {
        if self.t % self.frames.value() == 0 {
            self.buffer_old = self.buffer_new;
            self.buffer_new = self.step();
        }

        let coef = (self.t % self.frames.value()) as f32 / (self.frames.value() - 1) as f32;

        for x in 0..C {
            for y in 0..L {
                let c1 = Color::new(0.0, self.buffer_old[x][y]);
                let c2 = Color::new(1.01, self.buffer_new[x][y]);

                let mut grad: ColorGradient<2> = ColorGradient::new();
                grad.add_color(c1);
                grad.add_color(c2);
                self.led.write(x, y, grad.get(coef));
            }
        }

        self.t = self.t.wrapping_add(1);
        self.ticker.next().await;
    }
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize>
    StarryNight<'led, Led, C, L, N>
{
    fn spawn_stars(&mut self) {
        if !self.stars.is_full() && self.since_star_spawn >= STAR_SPAWN_COOLDOWN {
            self.stars.push(Star::new()).unwrap();
            self.since_star_spawn = 0;
        } else {
            self.since_star_spawn += 1;
        }
    }

    fn process_stars(&mut self) {
        self.stars.iter_mut().for_each(|star| star.go())
    }

    fn remove_stars(&mut self) {
        self.stars.retain(|star| star.y() != 0);
    }

    fn draw_stars(&self, buffer: &mut [[RGB8; L]; C]) {
        self.stars.iter().for_each(|star| {
            buffer[star.x()][star.y()] =
                self.stars_colormap.get_noised(star.temperature, -0.1, 0.1);
            if star.x() < C - 2 {
                buffer[star.x() + 1][star.y()] =
                    self.stars_colormap.get_noised(star.temperature, -0.1, 0.1);
            }
        });
    }
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> OnDirection
    for StarryNight<'led, Led, C, L, N>
{
    fn on_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.frames.down(),
            Direction::Down => self.frames.up(),
        }
    }
}

#[derive(Debug)]
struct Star<const C: usize, const L: usize> {
    x: usize,
    y: usize,
    since_y_moved: usize,
    temperature: f32,
}

impl<const C: usize, const L: usize> Star<C, L> {
    fn new() -> Self {
        let x = perlin::rand_int(0, C as i32) as usize;
        let y = L - 1;
        let since_y_moved = 0;

        let temperature = perlin::fair_rand_float();

        Star {
            x,
            y,
            since_y_moved,
            temperature,
        }
    }

    fn go(&mut self) {
        self.x = if self.x < C - 1 { self.x + 1 } else { 0 };

        if self.since_y_moved >= Y_COOLDOWN {
            self.y -= 1;
            self.since_y_moved = 0;
        } else {
            self.since_y_moved += 1
        }
    }

    fn x(&self) -> usize {
        self.x
    }

    fn y(&self) -> usize {
        self.y
    }
}
