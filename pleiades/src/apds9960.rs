use core::option::Option;
use embassy_rp::i2c::Instance;
use embassy_rp::i2c::{self, Error, Mode};
use embedded_hal_async::i2c::I2c;

pub const DEV_ADDR: u8 = 0x39;

pub struct Register;
impl Register {
    pub const ENABLE: u8 = 0x80;
    pub const ATIME: u8 = 0x81;
    pub const WTIME: u8 = 0x83;
    pub const AILTL: u8 = 0x84;
    pub const AIHTL: u8 = 0x86;
    pub const PILT: u8 = 0x89;
    pub const PIHT: u8 = 0x8B;
    pub const CONFIG1: u8 = 0x8D;
    const CONTROL: u8 = 0x8F;
    pub const CONFIG2: u8 = 0x90;
    pub const ID: u8 = 0x92;
    pub const STATUS: u8 = 0x93;
    pub const CDATAL: u8 = 0x94;
    pub const RDATAL: u8 = 0x96;
    pub const GDATAL: u8 = 0x98;
    pub const BDATAL: u8 = 0x9A;
    pub const PDATA: u8 = 0x9C;
    pub const POFFSET_UR: u8 = 0x9D;
    pub const POFFSET_DL: u8 = 0x9E;
    pub const GPENTH: u8 = 0xA0;
    pub const GPEXTH: u8 = 0xA1;
    pub const GCONFIG1: u8 = 0xA2;
    pub const GOFFSET_U: u8 = 0xA4;
    pub const GOFFSET_D: u8 = 0xA5;
    pub const GOFFSET_L: u8 = 0xA6;
    pub const GOFFSET_R: u8 = 0xA7;
    pub const GCONFIG4: u8 = 0xAB;
    pub const GFLVL: u8 = 0xAE;
    pub const GSTATUS: u8 = 0xAF;
    pub const IFORCE: u8 = 0xE4;
    pub const PICLEAR: u8 = 0xE5;
    pub const CICLEAR: u8 = 0xE6;
    pub const AICLEAR: u8 = 0xE7;
    pub const GFIFO_U: u8 = 0xFC;
}

pub struct Enable(u8);
impl Enable {
    const ALL: u8 = 0b1111_1111;
    const PON: u8 = 0b0000_0001;
    const AEN: u8 = 0b0000_0010;
    const PEN: u8 = 0b0000_0100;
    const WEN: u8 = 0b0000_1000;
    const AIEN: u8 = 0b0001_0000;
    const PIEN: u8 = 0b0010_0000;
    const GEN: u8 = 0b0100_0000;
}

pub struct Status(u8);
impl Status {
    pub const AVALID: u8 = 0b0000_0001;
    pub const PVALID: u8 = 0b0000_0010;
}

pub struct Apds9960<'d, T, M>
where
    T: Instance,
    M: Mode,
{
    i2c: i2c::I2c<'d, T, M>,
    sm: StateMashine,
}

impl<'d, T: Instance> Apds9960<'d, T, i2c::Async> {
    pub fn new(i2c: i2c::I2c<'d, T, i2c::Async>) -> Self {
        let sm = StateMashine::default();
        Apds9960 { i2c, sm }
    }

    pub async fn enable(&mut self) -> Result<(), Error> {
        self.i2c
            .write(DEV_ADDR, &[Register::ENABLE, Enable::PON | Enable::PEN])
            .await?;
        Ok(())
    }

    pub async fn powerup(&mut self) -> Result<(), Error> {
        self.i2c
            .write(DEV_ADDR, &[Register::CONTROL, 0b0000_0000])
            .await?; //LED DRIVE

        self.i2c
            .write(DEV_ADDR, &[Register::CONFIG2, 0b0011_0000])
            .await?; // LED BOOST
        Ok(())
    }

    pub async fn read(&mut self) -> Result<u8, Error> {
        let mut is_prox = [0u8];
        self.i2c
            .write_read(DEV_ADDR, &[Register::STATUS], &mut is_prox)
            .await?;

        let mut prox = [0u8];
        if is_prox[0] & Status::PVALID != 0 {
            self.i2c
                .write_read(DEV_ADDR, &[Register::PDATA], &mut prox)
                .await?;

            return Ok(prox[0]);
        }
        Err(Error::Abort(i2c::AbortReason::Other(42)))
    }

    pub async fn gesture(&mut self) -> Option<Command> {
        if let Ok(dist) = self.read().await {
            self.sm.next(dist);
            return Some(Command::Swing);
        }
        None
    }
}

impl<'d, T: Instance, M: Mode> Into<i2c::I2c<'d, T, M>> for Apds9960<'d, T, M> {
    fn into(self) -> i2c::I2c<'d, T, M> {
        self.i2c
    }
}

#[derive(Default)]
struct StateMashine {
    state: State,
    succ_checks: u32,
    power_checks: u32,
    updown_checks: u32,
    recorded: u32,
    init_dist: u8,
}

impl StateMashine {
    fn next(&mut self, dist: u8) {
        self.state = self.process(dist);
    }

    fn reset(&mut self) {
        self.succ_checks = 0;
        self.power_checks = 0;
        self.updown_checks = 0;
        self.recorded = 0;
        self.init_dist = 0;
    }

    fn process(&mut self, dist: u8) -> State {
        match self.state {
            State::Check => match dist {
                // dist > 0
                dist if dist > 1 => match self.succ_checks > 3 {
                    true => {
                        self.succ_checks += 1;
                        self.recorded = self.succ_checks;
                        State::Swing
                    }
                    false => {
                        self.succ_checks += 1;
                        State::Check
                    }
                },
                // dist == 0
                dist => {
                    self.reset();
                    State::Check
                }
            },

            State::Swing => match self.recorded <= 25 {
                // Gesture is fast...
                true => match dist == 0 {
                    // ... and now finished
                    true => {
                        // Swing
                        defmt::info!("Swing");
                        self.reset();
                        State::Check
                    }
                    // ... and continuing
                    false => {
                        self.recorded += 1;
                        State::Swing
                    }
                },
                // Gesture is slow, not just swing
                false => match dist == 0 {
                    true => {
                        // Swing
                        defmt::info!("Swing");
                        self.reset();
                        State::Check
                    }
                    false => {
                        self.init_dist = dist;
                        State::Record
                    }
                },
            },

            State::Record => match dist {
                // Hand close to sensor...
                dist if dist >= 200 => match self.power_checks {
                    // ... for a short time
                    checks if checks < 20 => {
                        self.power_checks += 1;
                        State::Record
                    }
                    // ... switch the power
                    checks if checks == 20 => {
                        // Power Switch
                        defmt::info!("Switch Power");
                        self.power_checks += 1;
                        State::Record
                    }
                    // ... for a long time
                    checks => State::Record,
                },
                // Gesture is over
                dist if dist == 0 => {
                    self.reset();
                    State::Check
                }

                // Hand at middle distance from sensor
                dist => match self.updown_checks > 5 {
                    true => {
                        // UP DOWN
                        match self.init_dist < dist {
                            true => {
                                //DOWN
                                defmt::info!("Down");
                                self.updown_checks = 0;
                                self.init_dist = dist;
                                State::Record
                            }
                            false => {
                                // UP
                                defmt::info!("Up");
                                self.updown_checks = 0;
                                self.init_dist = dist;
                                State::Record
                            }
                        }
                    }
                    false => {
                        self.updown_checks += 1;
                        State::Record
                    }
                },
            },
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Swing,
    SwitchPower,
    Level(u8),
}

#[derive(Debug)]
enum State {
    Check,
    Swing,
    Record,
}

impl Default for State {
    fn default() -> Self {
        State::Check
    }
}
