use crate::ws2812::Ws2812;
use embassy_rp::pio::Instance;
use smart_leds::RGB8;

pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct LedMatrix<
    'c,
    P: Instance,
    const S: usize,
    const L: usize,
    const C: usize,
    const N: usize,
> {
    data: [RGB8; N],
    ws: Ws2812<'c, P, S, N>,
}

impl<'c, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize>
    LedMatrix<'c, P, S, L, C, N>
{
    pub fn new(ws: Ws2812<'c, P, S, N>) -> Self {
        Self {
            data: [RGB8::default(); N],
            ws,
        }
    }

    pub fn write(&mut self, x: usize, y: usize, color: RGB8) {
        let index = self.index(x, y);
        self.data[index] = color;
    }

    pub fn write_straight(&mut self, index: usize, color: RGB8) {
        self.data[index] = color;
    }

    pub async fn flush(&mut self) {
        self.ws.write(&self.data).await;
    }

    pub fn clear(&mut self) {
        self.data = [RGB8::default(); N];
    }

    pub fn bg(&mut self, bg: RGB8) {
        self.data = [bg; N];
    }

    pub fn read(&self, x: usize, y: usize) -> RGB8 {
        let index = self.index(x, y);
        self.data[index]
    }

    fn index(&self, x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * L + y,
            false => x * L + (L - y) - 1,
        }
    }
}

impl<'c, P, const S: usize, const L: usize, const C: usize, const N: usize>
    Into<Ws2812<'c, P, S, N>> for LedMatrix<'c, P, S, L, C, N>
where
    P: Instance,
{
    fn into(self) -> Ws2812<'c, P, S, N> {
        self.ws
    }
}
