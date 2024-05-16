use super::OnDirection;
use crate::apds9960::Direction;
use crate::color::Color;
use crate::color::ColorGradient;
use crate::led_matrix::WritableMatrix;
use crate::perlin;
use crate::world::utils::CooldownValue;
use crate::world::{Flush, Tick};
use embassy_time::{Duration, Ticker};
use pleiades_macro_derive::Flush;
use smart_leds::RGB8;

const HUE_COOLDOWN: u8 = 0;
const HUE_MIN: usize = 0;
const HUE_MAX: usize = 75;

#[derive(Flush)]
pub struct Solid<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> {
    led: &'led mut Led,
    colormap: ColorGradient<8>,
    hue: CooldownValue<HUE_COOLDOWN, HUE_MIN, HUE_MAX>,
    ticker: Ticker,
    t: usize,
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize>
    Solid<'led, Led, C, L, N>
{
    pub fn new(led: &'led mut Led) -> Self {
        let colormap = Solid::<'led, Led, C, L, N>::get_colormap();
        let init_hue = perlin::rand_uint(HUE_MIN as u32, HUE_MAX as u32) as usize;
        let hue = CooldownValue::new(init_hue);
        let ticker = Ticker::every(Duration::from_millis(50));

        Self {
            led,
            colormap,
            hue,
            ticker,
            t: 0,
        }
    }

    fn get_colormap() -> ColorGradient<8> {
        let mut colormap = ColorGradient::new();
        colormap.add_color(Color::new(0.0, RGB8::new(255, 0, 255)));
        colormap.add_color(Color::new(0.15, RGB8::new(255, 0, 0)));
        colormap.add_color(Color::new(0.3, RGB8::new(255, 255, 0)));
        colormap.add_color(Color::new(0.45, RGB8::new(0, 255, 0)));
        colormap.add_color(Color::new(0.6, RGB8::new(0, 255, 255)));
        colormap.add_color(Color::new(0.75, RGB8::new(0, 0, 255)));
        colormap.add_color(Color::new(0.9, RGB8::new(255, 255, 255)));
        colormap.add_color(Color::new(1.01, RGB8::new(255, 255, 255)));

        colormap
    }
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> Tick
    for Solid<'led, Led, C, L, N>
{
    async fn tick(&mut self) {
        self.led.clear();

        let value = (*self.hue.value()) as f32 / HUE_MAX as f32;
        let color = self.colormap.get(value);
        self.led.bg(color);

        self.t = self.t.wrapping_add(1);
        self.ticker.next().await;
    }
}

impl<'led, Led: WritableMatrix, const C: usize, const L: usize, const N: usize> OnDirection
    for Solid<'led, Led, C, L, N>
{
    fn on_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.hue.up();
            }
            Direction::Down => {
                self.hue.down();
            }
        }
    }
}
