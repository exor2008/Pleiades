#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::pio::{
    Common, Config, FifoJoin, Instance, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use embassy_rp::relocate::RelocatedProgram;
use embassy_rp::{clocks, into_ref, Peripheral, PeripheralRef};
use embassy_time::{Duration, Timer};
use fixed::types::U24F8;
use fixed_macro::fixed;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

pub struct Ws2812<'d, P: Instance, const S: usize, const N: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize, const N: usize> Ws2812<'d, P, S, N> {
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
    ) -> Self {
        into_ref!(dma);

        // Setup sm0

        // prepare the PIO program
        let side_set = pio::SideSet::new(false, 1, false);
        let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

        const T1: u8 = 2; // start bit
        const T2: u8 = 5; // data bit
        const T3: u8 = 3; // stop bit
        const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

        let mut wrap_target = a.label();
        let mut wrap_source = a.label();
        let mut do_zero = a.label();
        a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
        a.bind(&mut wrap_target);
        // Do stop bit
        a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
        // Do start bit
        a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
        // Do data bit = 1
        a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
        a.bind(&mut do_zero);
        // Do data bit = 0
        a.nop_with_delay_and_side_set(T2 - 1, 0);
        a.bind(&mut wrap_source);

        let prg = a.assemble_with_wrap(wrap_source, wrap_target);
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);

        let relocated = RelocatedProgram::new(&prg);
        cfg.use_program(&pio.load_program(&relocated), &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        // TODO CLOCK_FREQ should come from embassy_rp
        let clock_freq = U24F8::from_num(clocks::clk_sys_freq() / 1000);
        let ws2812_freq = fixed!(800: U24F8);
        let bit_freq = ws2812_freq * CYCLES_PER_BIT;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            threshold: 24,
            direction: ShiftDirection::Left,
        };

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.map_into(),
            sm,
        }
    }

    pub async fn write(&mut self, colors: &[RGB8; N]) {
        // Precompute the word bytes from the colors
        let mut words = [0u32; N];
        for i in 0..N {
            let word = (u32::from(colors[i].g) << 24)
                | (u32::from(colors[i].r) << 16)
                | (u32::from(colors[i].b) << 8);
            words[i] = word;
        }

        // DMA transfer
        self.sm.tx().dma_push(self.dma.reborrow(), &words).await;
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

struct Coord {
    line_size: usize,
}

struct Point {
    x: usize,
    y: usize,
}

impl Coord {
    fn index(&self, x: usize, y: usize) -> usize {
        match x % 2 == 0 {
            true => x * self.line_size + y,
            false => x * self.line_size + (self.line_size - y) - 1,
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0);

    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 256;
    let mut data = [RGB8::default(); NUM_LEDS];

    // For the thing plus, use pin 8
    // For the feather, use pin 16
    let mut ws2812 = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_28);

    // Loop forever making RGB values and pushing them out to the WS2812.
    let coord = Coord { line_size: 16 };

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
        // for j in 0..(256 * 5) {
        //     // debug!("New Colors:");
        //     for i in 0..NUM_LEDS {
        //         data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
        //         // debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
        //     }
        //     ws2812.write(&data).await;

        //     Timer::after(Duration::from_micros(200)).await;
        // }
        // for i in 0..NUM_LEDS {
        //     data[i] = black;
        //     if i > 0 {
        //         data[i - 1] = black;
        //     } else if i == 0 {
        //         data[NUM_LEDS - 1] = black;
        //     }
        //     ws2812.write(&data).await;
        //     Timer::after(Duration::from_micros(250)).await;
        // }

        for i in 0..16 {
            for j in 0..16 {
                data[coord.index(i, j)] = RGB8::new(0, 0, 0);
            }
        }

        ws2812.write(&data).await;

        // Ian
        for p in &ian {
            for light in (0..255).step_by(25) {
                data[coord.index(p.x, p.y)] = RGB8::new(0, light, 0);
                ws2812.write(&data).await;
                Timer::after(Duration::from_millis(1)).await;
            }
        }
        // Plus
        for light in 0..255 {
            for p in &plus {
                data[coord.index(p.x, p.y)] = RGB8::new(light, light, 0);
            }
            ws2812.write(&data).await;
            Timer::after(Duration::from_millis(2)).await;
        }

        // Tania
        for p in &tania {
            for light in (0..255).step_by(25) {
                data[coord.index(p.x, p.y)] = RGB8::new(light, 0, 0);
                ws2812.write(&data).await;
                Timer::after(Duration::from_micros(1000)).await;
            }
        }

        // Equal
        for light in 0..255 {
            for p in &equal {
                data[coord.index(p.x, p.y)] = RGB8::new(0, light, light);
            }
            ws2812.write(&data).await;
            Timer::after(Duration::from_millis(2)).await;
        }

        // Mirich
        for p in &mirich {
            for light in (0..255).step_by(25) {
                data[coord.index(p.x, p.y)] = RGB8::new(light, light, light);
                ws2812.write(&data).await;
                Timer::after(Duration::from_micros(1000)).await;
            }
        }

        Timer::after(Duration::from_secs(2)).await;
    }
}
