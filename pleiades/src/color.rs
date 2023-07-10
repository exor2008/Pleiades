use core::usize;

use core::cmp::Ordering;
use embassy_rp::clocks::RoscRng;
use heapless::Vec;
use rand::Rng;
use smart_leds::RGB8;

#[derive(Debug)]
pub struct Color {
    pos: f32,
    rgb: RGB8,
}
impl defmt::Format for Color {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(
            fmt,
            "Color {{ r: {}, g: {}, b: {}, pos: {=f32}}}",
            self.rgb.r,
            self.rgb.g,
            self.rgb.b,
            self.pos,
        )
    }
}

impl Color {
    pub fn new(pos: f32, rgb: RGB8) -> Self {
        Color { pos, rgb }
    }
}

impl PartialEq<f32> for Color {
    fn eq(&self, other: &f32) -> bool {
        &self.pos == other
    }
}

impl PartialOrd<f32> for Color {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.pos.partial_cmp(other)
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl PartialOrd for Color {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.partial_cmp(&other.pos)
    }
}

pub struct ColorGradient<const N: usize> {
    colors: Vec<Color, N>,
}

impl<const N: usize> ColorGradient<N> {
    pub fn new() -> Self {
        let colors: Vec<Color, N> = Vec::new();
        ColorGradient { colors }
    }

    pub fn add_color(&mut self, color: Color) {
        match self.colors.push(color) {
            Ok(()) => (),
            Err(_) => defmt::panic!("Gradient capacity exceeded"),
        }

        self.colors
            .sort_unstable_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap_or(Ordering::Equal));
    }

    pub fn get(&self, value: f32, noise: bool) -> RGB8 {
        match self.search_closest(value) {
            Ok(left) => {
                let c1 = &self.colors[left];
                let c2 = &self.colors[left + 1];

                let value = match noise {
                    true => {
                        let value = value + rand_noise(-0.1, 0.1);
                        value.clamp(0.0, 1.0)
                    }
                    false => value,
                };

                self.lin_interp_colors(c1, c2, value)
            }
            Err(_) => {
                defmt::panic!("Error while during bin search");
            }
        }
    }

    fn lin_interp_colors(&self, c1: &Color, c2: &Color, value: f32) -> RGB8 {
        let coef = (value - c1.pos) / (c2.pos - c1.pos);

        let new_r = (c1.rgb.r as f32 + (c2.rgb.r as f32 - c1.rgb.r as f32) * coef) as u8;
        let new_g = (c1.rgb.g as f32 + (c2.rgb.g as f32 - c1.rgb.g as f32) * coef) as u8;
        let new_b = (c1.rgb.b as f32 + (c2.rgb.b as f32 - c1.rgb.b as f32) * coef) as u8;

        RGB8::new(new_r, new_g, new_b)
    }
    fn search_closest(&self, value: f32) -> Result<usize, BinSearchError> {
        for i in 0..self.colors.len() {
            if self.colors[i] > value {
                return Ok(i - 1);
            }
        }
        defmt::error!("Error search: value={}", value);
        Err(BinSearchError::InvalidSearch)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BinSearchError {
    InvalidSearch,
}

pub fn rand_noise(min: f32, max: f32) -> f32 {
    let mut rng = RoscRng;
    rng.gen_range(min..max)
}
