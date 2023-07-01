#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
use embassy_rp::pio::Instance;
use smart_leds::RGB8;

pub mod ws2812;

pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct World<'c, P: Instance, const S: usize, const N: usize> {
    line_size: usize,
    data: [RGB8; N],
    ws: ws2812::Ws2812<'c, P, S, N>,
}

impl<'c, P: Instance, const S: usize, const N: usize> World<'c, P, S, N> {
    pub fn new(line_size: usize, ws2812: ws2812::Ws2812<'c, P, S, N>) -> Self {
        Self {
            line_size: line_size,
            data: [RGB8::default(); N],
            ws: ws2812,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * self.line_size + y,
            false => x * self.line_size + (self.line_size - y) - 1,
        }
    }

    pub fn write(&mut self, x: usize, y: usize, color: RGB8) {
        let index = self.index(x, y);
        self.data[index] = color;
    }

    pub async fn flush(&mut self) {
        self.ws.write(&self.data).await;
    }
}
