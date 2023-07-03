#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]

use embassy_rp::dma::Channel;
use embassy_rp::pio::{Common, Instance, PioPin, StateMachine};
use embassy_rp::Peripheral;
use embassy_time::{Duration, Timer};
use smart_leds::RGB;

pub mod led_matrix;
pub mod perlin;

pub trait Tick {
    async fn tick(&mut self);
}

pub trait Flush {
    async fn flush(&mut self);
}

pub struct Fire<'a, P: Instance, const S: usize, const L: usize, const N: usize> {
    led: led_matrix::LedMatrix<'a, P, S, L, N>,
    noise: perlin::PerlinNoise,
    t: usize,
}

impl<'a, P, const S: usize, const L: usize, const N: usize> Fire<'a, P, S, L, N>
where
    P: Instance,
{
    pub fn new(
        pio: Common<'a, P>,
        sm: StateMachine<'a, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'a,
        pin: impl PioPin,
    ) -> Self {
        let led = led_matrix::LedMatrix::new(pio, sm, dma, pin);
        let noise = perlin::PerlinNoise::new();
        Self { led, noise, t: 0 }
    }
}

impl<'a, P, const S: usize, const L: usize, const N: usize> Tick for Fire<'a, P, S, L, N>
where
    P: Instance,
{
    async fn tick(&mut self) {
        for x in 0..16 {
            let xx = x as f64 / 1.6;
            let yy = self.t as f64 / 10.0;
            let noise = self.noise.get2d([xx, yy]);
            let noise = (noise - 0.3) / 0.25;
            let noise = (noise * 255.0) as u8;
            self.led.write(x, 15, RGB::new(noise, 0, 0))
        }
        self.t = self.t.wrapping_add(1);
        Timer::after(Duration::from_millis(100)).await;
    }
}

//TODO: Derive macro
impl<'a, P, const S: usize, const L: usize, const N: usize> Flush for Fire<'a, P, S, L, N>
where
    P: Instance,
{
    async fn flush(&mut self) {
        self.led.flush().await;
    }
}

pub enum World<'a, P, const S: usize, const L: usize, const N: usize>
where
    P: Instance,
{
    Fire(Fire<'a, P, S, L, N>),
}

impl<'a, P, const S: usize, const L: usize, const N: usize> World<'a, P, S, L, N>
where
    P: Instance,
{
    pub fn new_fire(
        pio: Common<'a, P>,
        sm: StateMachine<'a, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'a,
        pin: impl PioPin,
    ) -> Self {
        let fire = Fire::new(pio, sm, dma, pin);
        World::Fire(fire)
    }
}
