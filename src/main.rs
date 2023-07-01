#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pio::Pio;
use embassy_time::{Duration, Timer};
use pleiades::ws2812;
use pleiades::{Point, World};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

const NUM_LEDS: usize = 256;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0);

    let ws2812: ws2812::Ws2812<'_, embassy_rp::peripherals::PIO0, 0, NUM_LEDS> =
        ws2812::Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_28);

    let mut world: World<'_, embassy_rp::peripherals::PIO0, 0, NUM_LEDS> = World::new(16, ws2812);

    let ian = [
        Point { x: 1, y: 6 },
        Point { x: 2, y: 2 },
        Point { x: 2, y: 3 },
        Point { x: 2, y: 5 },
        Point { x: 3, y: 1 },
        Point { x: 3, y: 4 },
        Point { x: 4, y: 1 },
        Point { x: 4, y: 4 },
        Point { x: 5, y: 2 },
        Point { x: 5, y: 3 },
        Point { x: 5, y: 4 },
        Point { x: 5, y: 5 },
        Point { x: 5, y: 6 },
    ];

    let plus = [
        Point { x: 7, y: 4 },
        Point { x: 8, y: 3 },
        Point { x: 8, y: 4 },
        Point { x: 8, y: 5 },
        Point { x: 9, y: 4 },
    ];

    let tania = [
        Point { x: 10, y: 6 },
        Point { x: 11, y: 6 },
        Point { x: 12, y: 6 },
        Point { x: 13, y: 6 },
        Point { x: 14, y: 6 },
        Point { x: 12, y: 7 },
        Point { x: 12, y: 8 },
        Point { x: 12, y: 9 },
    ];

    let equal = [
        Point { x: 1, y: 8 },
        Point { x: 2, y: 8 },
        Point { x: 3, y: 8 },
        Point { x: 1, y: 10 },
        Point { x: 2, y: 10 },
        Point { x: 3, y: 10 },
    ];

    let mirich = [
        Point { x: 5, y: 11 },
        Point { x: 5, y: 12 },
        Point { x: 5, y: 13 },
        Point { x: 5, y: 14 },
        Point { x: 6, y: 10 },
        Point { x: 7, y: 11 },
        Point { x: 8, y: 12 },
        Point { x: 9, y: 11 },
        Point { x: 10, y: 10 },
        Point { x: 11, y: 11 },
        Point { x: 11, y: 12 },
        Point { x: 11, y: 13 },
        Point { x: 11, y: 14 },
    ];

    loop {
        for i in 0..16 {
            for j in 0..16 {
                world.write(i, j, RGB8::new(0, 0, 0));
            }
        }

        world.flush().await;

        // Ian
        for p in &ian {
            for light in (0..255).step_by(25) {
                world.write(p.x, p.y, RGB8::new(0, light, 0));
                world.flush().await;
                Timer::after(Duration::from_millis(1)).await;
            }
        }
        // Plus
        for light in 0..255 {
            for p in &plus {
                world.write(p.x, p.y, RGB8::new(light, light, 0));
            }
            world.flush().await;
            Timer::after(Duration::from_millis(2)).await;
        }

        // Tania
        for p in &tania {
            for light in (0..255).step_by(25) {
                world.write(p.x, p.y, RGB8::new(light, 0, 0));
                world.flush().await;
                Timer::after(Duration::from_micros(1000)).await;
            }
        }

        // Equal
        for light in 0..255 {
            for p in &equal {
                world.write(p.x, p.y, RGB8::new(0, light, light));
            }
            world.flush().await;
            Timer::after(Duration::from_millis(2)).await;
        }

        // Mirich
        for p in &mirich {
            for light in (0..255).step_by(25) {
                world.write(p.x, p.y, RGB8::new(light, light, light));
                world.flush().await;
                Timer::after(Duration::from_micros(1000)).await;
            }
        }

        Timer::after(Duration::from_secs(2)).await;
    }
}
