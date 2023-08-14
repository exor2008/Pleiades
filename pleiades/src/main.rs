#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Async, Config, InterruptHandler};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker};
use pleiades::apds9960::{Apds9960, Command};
use pleiades::world::{Flush, OnDirection, Switch, Tick, World};
use pleiades::ws2812::Ws2812;
use {defmt_rtt as _, panic_probe as _};

const NUM_LEDS_LINE: usize = 16;
const NUM_LEDS_COLUMN: usize = 16;
const NUM_LEDS: usize = NUM_LEDS_LINE * NUM_LEDS_COLUMN;
const STATE_MACHINE: usize = 0;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

static CHANNEL: Channel<ThreadModeRawMutex, Command, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());
    let sda = p.PIN_20;
    let scl = p.PIN_21;

    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    let apds = Apds9960::new(i2c);

    unwrap!(spawner.spawn(sensor_task(apds)));

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0);

    let ws2812: Ws2812<PIO0, STATE_MACHINE, NUM_LEDS> =
        Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_22);

    let mut world: World<
        '_,
        PIO0,
        STATE_MACHINE,
        NUM_LEDS_LINE,
        NUM_LEDS_COLUMN,
        NUM_LEDS,
        { 2 * NUM_LEDS },
    > = World::fire_from(ws2812);
    // > = World::matrix_from(ws2812);
    // > = World::northen_light_from(ws2812);
    // > = World::voronoi_from(ws2812);

    let mut switch = Switch::new();

    loop {
        if let Ok(command) = CHANNEL.try_recv() {
            // defmt::info!("Command!: {}", command);
            match command {
                Command::Level(direction) => world.on_direction(direction),
                Command::Swing => world = switch.switch_world(world),
                Command::SwitchPower => world = switch.switch_power(world),
            }
        }

        tick!(world, Empty, Fire, NorthenLight, Matrix, Voronoi, Solid)
    }
}

#[embassy_executor::task]
async fn sensor_task(mut apds: Apds9960<'static, I2C0, Async>) -> ! {
    apds.enable().await.unwrap();
    apds.powerup().await.unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(10));

    loop {
        // if let Ok(d) = apds.read().await {
        //     defmt::info!("Dist: {}", d);
        // }
        apds.gesture().await;
        if let Some(command) = apds.command() {
            if let Err(_err) = CHANNEL.try_send(command) {
                defmt::error!("Command channel buffer is full");
            }
        }
        ticker.next().await;
    }
}

// call tick() and flush() for every World case
#[macro_export]
macro_rules! tick {
    ($world:expr, $($variant:ident),*) => {
        match $world{
        $(
            World::$variant(ref mut v) => {
                v.tick().await;
                v.flush().await;
            }
        )*
    }
    };
}
