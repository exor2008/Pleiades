#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Async, Config, InterruptHandler as I2CInterruptHandler};
use embassy_rp::peripherals::{I2C0, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Ticker};
use pleiades::apds9960::{Apds9960, Command};
use pleiades::led_matrix::LedMatrix;
use pleiades::world::{OnDirection, Switch, World};
use pleiades::ws2812::Ws2812;

#[cfg(feature = "panic-probe")]
use panic_probe as _;
#[cfg(feature = "panic-reset")]
use panic_reset as _;

const NUM_LEDS_LINE: usize = 16;
const NUM_LEDS_COLUMN: usize = 16;
const NUM_LEDS: usize = NUM_LEDS_LINE * NUM_LEDS_COLUMN;
const STATE_MACHINE: usize = 0;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

static CHANNEL: Channel<ThreadModeRawMutex, Command, 1> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    // Init pins
    let p = embassy_rp::init(Default::default());
    let sda = p.PIN_20;
    let scl = p.PIN_21;

    // Init I2C and Apds9960 gesture sensor 
    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    let apds = Apds9960::new(i2c);

    // Start sensor_task asynchronously
    unwrap!(spawner.spawn(sensor_task(apds)));

    // Init PIO to support WS2812 protocol
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    Init WS2812 LED controller
    let mut ws2812: Ws2812<PIO0, STATE_MACHINE, NUM_LEDS> =
        Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_22);

    // Init 16x16 LED matrix controller
    let mut led_matrix: LedMatrix<Ws2812<PIO0, 0, NUM_LEDS>, NUM_LEDS_LINE, NUM_LEDS> =
        LedMatrix::new(&mut ws2812);

    // Create a new world
    let mut world: World<'_, _, NUM_LEDS_COLUMN, NUM_LEDS_LINE, NUM_LEDS, { 2 * NUM_LEDS }> =
        World::fire_new(&mut led_matrix);
    // > = World::matrix_from(ws2812);
    // > = World::northen_light_from(ws2812);
    // > = World::voronoi_from(ws2812);

    let mut switch = Switch::new();

    loop {
        // Handle the command from the gesture sensor
        if let Ok(command) = CHANNEL.try_receive() {
            // defmt::info!("Command!: {}", command);
            match command {
                Command::Level(direction) => world.on_direction(direction),
                Command::Swing => world = switch.switch_world(&mut led_matrix),
                Command::SwitchPower => world = switch.switch_power(&mut led_matrix),
            }
        }

        // World::tick is generated by macros
        World::tick(&mut world).await;
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
