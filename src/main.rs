#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::Pio;
// use embassy_time::{Duration, Timer};
use pleiades::{Flush, Tick, World};
use {defmt_rtt as _, panic_probe as _};

const NUM_LEDS: usize = 256;
const NUM_LEDS_LINE: usize = 16;
const STATE_MACHINE: usize = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio { common, sm0, .. } = Pio::new(p.PIO0);

    // let world: &mut dyn World = &mut Fire::<PIO0, STATE_MACHINE, NUM_LEDS_LINE, NUM_LEDS>::new(
    //     common, sm0, p.DMA_CH0, p.PIN_28,
    // );
    let world: &mut World<'_, PIO0, STATE_MACHINE, NUM_LEDS_LINE, NUM_LEDS> =
        &mut World::new_fire(common, sm0, p.DMA_CH0, p.PIN_28);

    loop {
        match world {
            World::Fire(fire) => {
                fire.tick().await;
                fire.flush().await;
            }
        }
    }
}
