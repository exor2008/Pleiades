#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Async, Config, InterruptHandler};
use embassy_rp::peripherals::{DMA_CH1, I2C0, PIN_23, PIN_25, PIO0, PIO1};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker};
use pleiades::apds9960::{Apds9960, Command};
use pleiades::wifi::{Create, Join, Wifi};
use pleiades::world::{Flush, OnDirection, Switch, Tick, World};
use pleiades::ws2812::Ws2812;
use {defmt_rtt as _, panic_probe as _};

const NUM_LEDS_LINE: usize = 16;
const NUM_LEDS_COLUMN: usize = 16;
const NUM_LEDS: usize = NUM_LEDS_LINE * NUM_LEDS_COLUMN;
const STATE_MACHINE: usize = 0;
static WIFI_SSID: &str = "pleiades";
static WIFI_PASSWORD: &str = "pleiades";

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

static CHANNEL: Channel<ThreadModeRawMutex, Command, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    // Init periferals
    let p = embassy_rp::init(Default::default());
    let sda = p.PIN_20;
    let scl = p.PIN_21;

    // Sensor
    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    let apds: Apds9960<'_, I2C0, Async> = Apds9960::new(i2c);

    // Wifi
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let Pio {
        common, sm0, irq0, ..
    } = Pio::new(p.PIO1);

    // let stack: &'static mut Stack<NetDriver<'static>>;
    // let runner: Runner<'static, Output<'_, PIN_23>, PioSpi<'_, PIN_25, PIO1, 0, DMA_CH1>>;

    match true {
        true => {
            let (net_device, control, runner) =
                Wifi::<Create>::pre_init(common, sm0, irq0, cs, pwr, p.PIN_24, p.PIN_29, p.DMA_CH1)
                    .await;

            unwrap!(spawner.spawn(wifi_task(runner)));

            let (mut wifi, stack) = Wifi::init(net_device, control).await;

            unwrap!(spawner.spawn(net_task(stack)));
            wifi.create(WIFI_SSID, WIFI_PASSWORD).await;
        }
        false => {
            let (net_device, control, runner) =
                Wifi::<Join>::pre_init(common, sm0, irq0, cs, pwr, p.PIN_24, p.PIN_29, p.DMA_CH1)
                    .await;

            unwrap!(spawner.spawn(wifi_task(runner)));

            let (mut wifi, stack) = Wifi::init(net_device, control).await;

            unwrap!(spawner.spawn(net_task(stack)));
            wifi.join().await;
        }
    }

    // Run tasks
    unwrap!(spawner.spawn(sensor_task(apds)));

    // LED
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0);

    let ws2812: Ws2812<PIO0, STATE_MACHINE, NUM_LEDS> =
        Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_22);

    // World
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

        match world {
            World::Empty(ref mut empty) => {
                empty.tick().await;
                empty.flush().await;
            }
            World::Fire(ref mut fire) => {
                fire.tick().await;
                fire.flush().await;
            }
            World::NorthenLight(ref mut nl) => {
                nl.tick().await;
                nl.flush().await;
            }
            World::Matrix(ref mut night) => {
                night.tick().await;
                night.flush().await;
            }
            World::Voronoi(ref mut voronoi) => {
                voronoi.tick().await;
                voronoi.flush().await;
            }
        }
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

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO1, 0, DMA_CH1>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}
