use crate::ws2812::PioWrite;
use smart_leds::RGB8;

pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub trait WritableMatrix {
    fn write(&mut self, x: usize, y: usize, color: RGB8);
    fn write_straight(&mut self, index: usize, color: RGB8);
    fn clear(&mut self);
    fn bg(&mut self, bg: RGB8);
    fn read(&self, x: usize, y: usize) -> RGB8;
    async fn flush(&mut self);
}
pub struct LedMatrix<'ws, Ws: PioWrite<N>, const L: usize, const N: usize> {
    data: [RGB8; N],
    ws: &'ws mut Ws,
}

impl<'ws, Ws: PioWrite<N>, const L: usize, const N: usize> LedMatrix<'ws, Ws, L, N> {
    pub fn new(ws: &'ws mut Ws) -> Self {
        Self {
            data: [RGB8::default(); N],
            ws,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * L + y,
            false => x * L + (L - y) - 1,
        }
    }
}

impl<'ws, Ws: PioWrite<N>, const L: usize, const N: usize> WritableMatrix
    for LedMatrix<'ws, Ws, L, N>
{
    fn write(&mut self, x: usize, y: usize, color: RGB8) {
        let index = self.index(x, y);
        self.data[index] = color;
    }

    fn write_straight(&mut self, index: usize, color: RGB8) {
        self.data[index] = color;
    }

    fn clear(&mut self) {
        self.data = [RGB8::default(); N];
    }

    fn bg(&mut self, bg: RGB8) {
        self.data = [bg; N];
    }

    fn read(&self, x: usize, y: usize) -> RGB8 {
        let index = self.index(x, y);
        self.data[index]
    }

    async fn flush(&mut self) {
        self.ws.write(&self.data).await;
    }
}
