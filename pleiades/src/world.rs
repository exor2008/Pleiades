use crate::{apds9960::Direction, ws2812::Ws2812};
use embassy_rp::pio::Instance;
use pleiades_macro_derive::enum_world;

pub mod empty;
pub mod fire;
pub mod matrix;
pub mod northen_light;
pub mod solid;
pub mod starry_night;
pub mod utils;
pub mod voronoi;

const WORLDS: usize = 6;

pub trait Tick {
    async fn tick(&mut self);
}

pub trait Flush {
    async fn flush(&mut self);
}

pub trait OnDirection {
    fn on_direction(&mut self, direction: Direction);
}

#[enum_world(Empty, Fire, NorthenLight, Matrix, Voronoi, StarryNight, Solid)]
pub enum World<
    'a,
    P,
    const S: usize,
    const L: usize,
    const C: usize,
    const N: usize,
    const N2: usize,
> where
    P: Instance,
{
    Empty(empty::Empty<'a, P, S, L, C, N>),
    Fire(fire::Fire<'a, P, S, L, C, N>),
    NorthenLight(northen_light::NorthenLight<'a, P, S, L, C, N>),
    Matrix(matrix::Matrix<'a, P, S, L, C, N, N2>),
    Voronoi(voronoi::Voronoi<'a, P, S, L, C, N>),
    StarryNight(starry_night::StarryNight<'a, P, S, L, C, N>),
    Solid(solid::Solid<'a, P, S, L, C, N>),
}

pub struct Switch {
    counter: usize,
    prev_counter: usize,
    is_on: bool,
}

impl Switch {
    pub fn new() -> Self {
        Switch {
            counter: 1,
            prev_counter: Default::default(),
            is_on: true,
        }
    }

    pub fn switch_world<
        'a,
        P: Instance,
        const S: usize,
        const L: usize,
        const C: usize,
        const N: usize,
        const N2: usize,
    >(
        &mut self,
        world: World<'a, P, S, L, C, N, N2>,
    ) -> World<'a, P, S, L, C, N, N2> {
        // Destroy old world and return peripherial resources
        self.counter += 1;
        self.counter = if self.counter > WORLDS {
            1
        } else {
            self.counter
        }; // TODO: edit
        self.get_world(world)
    }

    fn turn_off<
        'a,
        P: Instance,
        const S: usize,
        const L: usize,
        const C: usize,
        const N: usize,
        const N2: usize,
    >(
        &mut self,
        world: World<'a, P, S, L, C, N, N2>,
    ) -> World<'a, P, S, L, C, N, N2> {
        self.prev_counter = self.counter;
        self.counter = 0;
        self.get_world(world)
    }

    fn turn_on<
        'a,
        P: Instance,
        const S: usize,
        const L: usize,
        const C: usize,
        const N: usize,
        const N2: usize,
    >(
        &mut self,
        world: World<'a, P, S, L, C, N, N2>,
    ) -> World<'a, P, S, L, C, N, N2> {
        self.counter = self.prev_counter;
        self.get_world(world)
    }

    pub fn switch_power<
        'a,
        P: Instance,
        const S: usize,
        const L: usize,
        const C: usize,
        const N: usize,
        const N2: usize,
    >(
        &mut self,
        world: World<'a, P, S, L, C, N, N2>,
    ) -> World<'a, P, S, L, C, N, N2> {
        match self.is_on {
            true => {
                self.is_on = false;
                self.turn_off(world)
            }
            false => {
                self.is_on = true;
                self.turn_on(world)
            }
        }
    }

    fn get_world<
        'a,
        P: Instance,
        const S: usize,
        const L: usize,
        const C: usize,
        const N: usize,
        const N2: usize,
    >(
        &mut self,
        world: World<'a, P, S, L, C, N, N2>,
    ) -> World<'a, P, S, L, C, N, N2> {
        // Destroy old world and return peripherial resources
        let ws: Ws2812<'a, P, S, N> = world.into();
        match self.counter {
            0 => World::empty_from(ws),
            1 => World::fire_from(ws),
            2 => World::northen_light_from(ws),
            3 => World::matrix_from(ws),
            4 => World::voronoi_from(ws),
            5 => World::starry_night_from(ws),
            6 => World::solid_from(ws),
            _ => {
                defmt::panic!("World counter out of bounds")
            }
        }
    }
}
