use super::OnDirection;
use crate::apds9960::Direction;
use crate::led_matrix;
use crate::world::{Flush, Tick};
use crate::ws2812::Ws2812;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Ticker};
use pleiades_macro_derive::{Flush, From, Into};

#[derive(Flush, Into, From)]
pub struct Empty<'a, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    ticker: Ticker,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Empty<'a, P, S, L, C, N>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let ticker = Ticker::every(Duration::from_millis(50));

        Empty { led, ticker }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Tick
    for Empty<'a, P, S, L, C, N>
where
    P: Instance,
{
    async fn tick(&mut self) {
        self.led.clear();
        self.ticker.next().await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> OnDirection
    for Empty<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn on_direction(&mut self, _direction: Direction) {}
}
