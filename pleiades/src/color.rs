use crate::perlin::rand_float;
use core::cmp::max;
use core::cmp::Ordering;
use heapless::Vec;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;

#[derive(Debug, Clone, Copy)]
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

pub struct ColorGradient<const COLORS: usize> {
    colors: Vec<Color, COLORS>,
    hsv: [Hsv; COLORS],
    diff: i8,
}

impl<const COLORS: usize> Default for ColorGradient<COLORS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const COLORS: usize> ColorGradient<COLORS> {
    pub fn new() -> Self {
        let colors: Vec<Color, COLORS> = Vec::new();
        let hsv = [Hsv::default(); COLORS];
        ColorGradient {
            colors,
            hsv,
            diff: 0,
        }
    }

    pub fn from_hsv(pos: [f32; COLORS], hsv: [Hsv; COLORS]) -> Self {
        let colors: Vec<Color, COLORS> = pos
            .iter()
            .zip(hsv.iter())
            .map(|(p, c)| Color::new(*p, hsv2rgb(*c)))
            .collect();
        ColorGradient {
            colors,
            hsv,
            diff: 0,
        }
    }

    pub fn add_color(&mut self, color: Color) {
        match self.colors.push(color) {
            Ok(()) => (),
            Err(_) => defmt::panic!("Gradient capacity exceeded"),
        }

        self.colors
            .sort_unstable_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap_or(Ordering::Equal));
    }

    pub fn set_rgb(&mut self, i: usize, rgb: RGB8) {
        self.colors[i].rgb = rgb;
    }

    pub fn set_color(&mut self, i: usize, color: Color) {
        self.colors[i] = color;
    }

    pub fn get(&self, value: f32) -> RGB8 {
        match self.search_closest(value) {
            Ok(left) => {
                let c1 = &self.colors[left];
                let c2 = &self.colors[left + 1];

                ColorGradient::<COLORS>::lin_interp_colors(c1, c2, value)
            }
            Err(_) => {
                defmt::panic!("Error while during bin search. Value: {}", value);
            }
        }
    }

    pub fn get_noised(&self, value: f32, min: f32, max: f32) -> RGB8 {
        match self.search_closest(value) {
            Ok(left) => {
                let c1 = &self.colors[left];
                let c2 = &self.colors[left + 1];

                let value = value + rand_float(min, max);
                let value = value.clamp(0.0, 1.0);

                ColorGradient::<COLORS>::lin_interp_colors(c1, c2, value)
            }
            Err(_) => {
                defmt::panic!("Error while during bin search");
            }
        }
    }

    pub fn lin_interp_colors(c1: &Color, c2: &Color, value: f32) -> RGB8 {
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

    pub fn change_value(&mut self, diff: i8) {
        self.diff = self.diff.saturating_add(diff);

        self.colors
            .iter_mut()
            .zip(self.hsv.iter())
            .for_each(|(color, hsv)| {
                let mut new_hsv = *hsv;
                new_hsv.val = new_hsv.val.saturating_add_signed(self.diff);
                new_hsv.val = max(new_hsv.val, 1);
                color.rgb = hsv2rgb(new_hsv)
            })
    }

    pub fn colors(&self) -> &[Color] {
        self.colors.as_slice()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BinSearchError {
    InvalidSearch,
}
