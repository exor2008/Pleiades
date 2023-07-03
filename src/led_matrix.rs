use embassy_rp::dma::Channel;
use embassy_rp::pio::{Common, Instance, PioPin, StateMachine};
use embassy_rp::Peripheral;
use smart_leds::RGB8;
mod ws2812;
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct LedMatrix<'c, P: Instance, const S: usize, const L: usize, const N: usize> {
    data: [RGB8; N],
    ws: ws2812::Ws2812<'c, P, S, N>,
}

impl<'c, P: Instance, const S: usize, const L: usize, const N: usize> LedMatrix<'c, P, S, L, N> {
    pub fn new(
        pio: Common<'c, P>,
        sm: StateMachine<'c, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'c,
        pin: impl PioPin,
    ) -> Self {
        Self {
            data: [RGB8::default(); N],
            ws: ws2812::Ws2812::new(pio, sm, dma, pin),
        }
    }

    pub fn write(&mut self, x: usize, y: usize, color: RGB8) {
        let index = self.index(x, y);
        self.data[index] = color;
    }

    pub async fn flush(&mut self) {
        self.ws.write(&self.data).await;
    }

    pub fn clear(&mut self) {
        self.data = [RGB8::default(); N];
    }
    fn index(&self, x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * L + y,
            false => x * L + (L - y) - 1,
        }
    }
}
