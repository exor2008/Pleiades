use super::OnDirection;
use crate::apds9960::Direction;
use crate::led_matrix::WritableMatrix;
use crate::world::{Flush, Tick};
use embassy_time::{Duration, Ticker};
use pleiades_macro_derive::Flush;

#[derive(Flush)]
pub struct Empty<'led, Led: WritableMatrix> {
    led: &'led mut Led,
    ticker: Ticker,
}

impl<'led, Led: WritableMatrix> Empty<'led, Led> {
    pub fn new(led: &'led mut Led) -> Self {
        let ticker = Ticker::every(Duration::from_millis(50));

        Empty { led, ticker }
    }
}

impl<'led, Led: WritableMatrix> Tick for Empty<'led, Led> {
    async fn tick(&mut self) {
        self.led.clear();
        self.ticker.next().await;
    }
}

impl<'led, Led: WritableMatrix> OnDirection for Empty<'led, Led> {
    fn on_direction(&mut self, _direction: Direction) {}
}
