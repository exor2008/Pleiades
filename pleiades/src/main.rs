#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::Pio;
use pleiades::world::{Flush, Tick, World};
use pleiades::ws2812::Ws2812;

use {defmt_rtt as _, panic_probe as _};

const NUM_LEDS_LINE: usize = 16;
const NUM_LEDS_COLUMN: usize = 16;
const NUM_LEDS: usize = NUM_LEDS_LINE * NUM_LEDS_COLUMN;
const STATE_MACHINE: usize = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0);

    let ws2812: Ws2812<PIO0, STATE_MACHINE, NUM_LEDS> =
        Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_28);

    let mut world: World<'_, PIO0, STATE_MACHINE, NUM_LEDS_LINE, NUM_LEDS_COLUMN, NUM_LEDS> =
        World::fire_from(ws2812);

    loop {
        match world {
            World::Fire(ref mut fire) => {
                fire.tick().await;
                fire.flush().await;
            }
        }
    }

    // let mut ws2812: Ws2812<PIO0, STATE_MACHINE, NUM_LEDS> = world.into();
    // ws2812.write(&[RGB::new(0, 0, 0); 256]).await;
    // }
}
