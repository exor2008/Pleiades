use core::marker::PhantomData;
use cyw43::Runner;
use cyw43::{Control, NetDriver};
use cyw43_pio::PioSpi;
use embassy_net::{self, Config, Stack, StackResources};
use embassy_net::{Ipv4Address, Ipv4Cidr};
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Output, Pin};
use embassy_rp::pio::{Common, Instance, Irq, PioPin, StateMachine};
use embassy_rp::Peripheral;
use embedded_hal_1::digital::OutputPin;
use heapless::Vec;
use static_cell::make_static;

pub struct Wifi<'a, WifiType> {
    control: Control<'a>,
    kind: PhantomData<WifiType>,
}

impl<'a, WifiType> Wifi<'a, WifiType>
where
    WifiType: IpConfig,
{
    pub async fn pre_init<CS, PIO, DMA, DIO, CLK, PWR, const SM: usize>(
        mut common: Common<'a, PIO>,
        sm: StateMachine<'a, PIO, SM>,
        irq: Irq<'a, PIO, 0>,
        cs: Output<'a, CS>,
        pwr: PWR,
        dio: DIO,
        clk: CLK,
        dma: impl Peripheral<P = DMA> + 'a,
    ) -> (
        NetDriver<'a>,
        Control<'a>,
        Runner<'a, PWR, PioSpi<'a, CS, PIO, SM, DMA>>,
    )
    where
        CS: Pin,
        PIO: Instance,
        DMA: Channel,
        PWR: OutputPin,
        DIO: PioPin,
        CLK: PioPin,
        WifiType: IpConfig,
    {
        // load wifi drivers
        let fw = include_bytes!("../cyw43-firmware/43439A0.bin");

        let spi: PioSpi<'_, CS, PIO, SM, DMA> =
            PioSpi::new(&mut common, sm, irq, cs, dio, clk, dma);
        let state = make_static!(cyw43::State::new());
        let (net_device, control, runner) = cyw43::new(state, pwr, spi, fw).await;

        return (net_device, control, runner);
    }

    pub async fn init(
        net_device: NetDriver<'static>,
        mut control: Control<'a>,
    ) -> (Wifi<'a, WifiType>, &'static mut Stack<NetDriver<'static>>) {
        let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

        control.init(clm).await;
        control
            .set_power_management(cyw43::PowerManagementMode::PowerSave)
            .await;
        let config = WifiType::config();
        let seed: u64 = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

        // Init network stack
        let stack = make_static!(Stack::new(
            net_device,
            config,
            make_static!(StackResources::<2>::new()),
            seed,
        ));

        (
            Wifi {
                control,
                kind: PhantomData::<WifiType>,
            },
            stack,
        )
    }
}

pub struct Create;

pub struct Join;

impl<'a> Wifi<'a, Create> {
    pub async fn create(&mut self, ssid: &str, passphrase: &str) {
        self.control.start_ap_wpa2(ssid, passphrase, 5).await;
    }
}

impl<'a> Wifi<'a, Join> {
    pub async fn join(&mut self) {
        self.control
            .join_wpa2("pleiades", "pleiades")
            .await
            .unwrap();
    }
}

pub trait IpConfig {
    fn config() -> Config;
}

impl IpConfig for Create {
    fn config() -> Config {
        embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
            dns_servers: Vec::new(),
            gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
        })
    }
}

impl IpConfig for Join {
    fn config() -> Config {
        Config::dhcpv4(Default::default())
    }
}
