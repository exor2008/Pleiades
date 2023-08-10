use core::cell::RefCell;
use defmt::info;
use defmt_rtt as _;
use embassy_boot_rp::*;
use embassy_rp::flash::Flash;
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_storage::nor_flash::NorFlash;
use heapless::Vec;

const FLASH_SIZE: usize = 2 * 1024 * 1024;

// embassy_sync::channel::Channel<ThreadModeRawMutex, Vec<u8, 4096>, 1>
pub async fn update_firmware<'a>(
    firmware: &'static Channel<ThreadModeRawMutex, Vec<u8, 4096>, 1>,
    watchdog: WATCHDOG,
    flash: FLASH,
) {
    // let mut led = Output::new(pin, Level::Low);

    // Override bootloader watchdog
    let mut watchdog = Watchdog::new(watchdog);
    watchdog.start(Duration::from_secs(8));

    let flash: Flash<_, FLASH_SIZE> = Flash::new(flash);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash);
    let mut updater = BlockingFirmwareUpdater::new(config);

    // Timer::after(Duration::from_secs(5)).await;
    watchdog.feed();
    // led.set_high();
    let mut offset = 0;
    let mut buf: AlignedBuffer<4096> = AlignedBuffer([0; 4096]);
    let mut buf2: AlignedBuffer<2> = AlignedBuffer([0; 2]);
    defmt::info!("preparing update");

    let state = updater
        .get_state(&mut buf.0[..1])
        .map_err(|e| defmt::warn!("E: {:?}", defmt::Debug2Format(&e)))
        .unwrap();

    match state {
        State::Boot => info!("State is Boot!"),
        State::Swap => info!("State is Swap!"),
    }

    let writer = updater
        .prepare_update(&mut buf.0[..1])
        .map_err(|e| defmt::warn!("E: {:?}", defmt::Debug2Format(&e)))
        .unwrap();
    defmt::info!("writer created, starting write");

    // let mut chunk = firmware.recv().await;

    // while !chunk.is_empty() {
    //     buf.0[..chunk.len()].copy_from_slice(&chunk.as_slice());
    //     // defmt::info!("writing block at offset {}", offset);
    //     writer.write(offset, &buf.0[..chunk.len()]).unwrap();
    //     offset += chunk.len() as u32;
    //     // Timer::after(Duration::from_millis(500)).await;
    //     watchdog.feed();

    //     chunk = firmware.recv().await;
    // }

    // let app = include_bytes!("D:\\matrix.bin");

    // for chunk in app.chunks(4096) {
    //     buf.0[..chunk.len()].copy_from_slice(&chunk[..chunk.len()]);
    //     defmt::info!("writing block at offset {}", offset);
    //     writer.write(offset, &buf.0[..chunk.len()]).unwrap();
    //     offset += chunk.len() as u32;
    //     watchdog.feed();
    // }

    // for chunk in app.chunks(4096) {
    //     buf.0[..chunk.len()].copy_from_slice(&chunk[..chunk.len()]);
    //     defmt::info!("writing block at offset {}", offset);
    //     writer.write(offset, &buf.0[..chunk.len()]).unwrap();
    //     // updater
    //     //     .write_firmware(&mut buf2.0[..1], offset, &buf.0[..chunk.len()])
    //     //     .unwrap();
    //     offset += chunk.len() as u32;
    //     watchdog.feed();
    // }

    watchdog.feed();
    defmt::info!("firmware written ({} bytes), marking update", offset);
    updater.mark_updated(&mut buf.0[..1]).unwrap();
    Timer::after(Duration::from_secs(2)).await;
    // led.set_low();
    defmt::info!("update marked, resetting");
    // Timer::after(Duration::from_secs(1)).await;
    cortex_m::peripheral::SCB::sys_reset();
    // loop {}
}

pub fn mark_booted(flash: &FLASH) {
    let flash: Flash<_, FLASH_SIZE> = Flash::new(flash);
    let flash = Mutex::new(RefCell::new(flash));

    let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash);
    let mut updater = BlockingFirmwareUpdater::new(config);

    let mut buf: AlignedBuffer<1> = AlignedBuffer([0; 1]);
    defmt::info!("preparing update");

    let state = updater
        .get_state(&mut buf.0[..1])
        .map_err(|e| defmt::warn!("E: {:?}", defmt::Debug2Format(&e)))
        .unwrap();

    match state {
        State::Boot => defmt::info!("State is Boot"),
        State::Swap => {
            defmt::info!("Firmware loaded successfully. Mrking as booted");
            updater.mark_booted(&mut buf.0[..1]).unwrap()
        }
    };
}
