use crate::{apds9960::Direction, ws2812::Ws2812};
use core::usize;
use embassy_rp::pio::Instance;

pub mod fire;
pub mod matrix;
pub mod norhten_light;
pub mod voronoi;
// pub mod starry_night;
pub trait Tick {
    async fn tick(&mut self);
}

pub trait Flush {
    async fn flush(&mut self);
}

pub trait OnDirection {
    fn on_direction(&mut self, direction: Direction);
}

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
    Fire(fire::Fire<'a, P, S, L, C, N>),
    NorthenLight(norhten_light::NorthenLight<'a, P, S, L, C, N>),
    Matrix(matrix::Matrix<'a, P, S, L, C, N, N2>), // StarryNight(starry_night::StarryNight<'a, P, S, L, C, N>),
    Voronoi(voronoi::Voronoi<'a, P, S, L, C, N>),
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize>
    World<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    pub fn fire_from(ws: Ws2812<'a, P, S, N>) -> Self {
        let fire = fire::Fire::from(ws);
        World::Fire(fire)
    }

    pub fn northen_light_from(ws: Ws2812<'a, P, S, N>) -> Self {
        let northen_light = norhten_light::NorthenLight::from(ws);
        World::NorthenLight(northen_light)
    }

    pub fn matrix_from(ws: Ws2812<'a, P, S, N>) -> Self {
        let matrix = matrix::Matrix::from(ws);
        World::Matrix(matrix)
    }

    pub fn voronoi_from(ws: Ws2812<'a, P, S, N>) -> Self {
        let voronoi = voronoi::Voronoi::from(ws);
        World::Voronoi(voronoi)
    }

    // pub fn starry_night_from(ws: Ws2812<'a, P, S, N>) -> Self {
    //     let starry_night = starry_night::StarryNight::from(ws);
    //     World::StarryNight(starry_night)
    // }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize>
    Into<Ws2812<'a, P, S, N>> for World<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    fn into(self) -> Ws2812<'a, P, S, N> {
        match self {
            Self::Fire(fire) => fire.into(),
            Self::NorthenLight(nl) => nl.into(),
            Self::Matrix(m) => m.into(),
            Self::Voronoi(v) => v.into(),
            // Self::StarryNight(night) => night.into(),
        }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize>
    OnDirection for World<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    fn on_direction(&mut self, direction: Direction) {
        match self {
            Self::Fire(fire) => fire.on_direction(direction),
            Self::NorthenLight(nl) => nl.on_direction(direction),
            Self::Matrix(m) => m.on_direction(direction),
            Self::Voronoi(v) => v.on_direction(direction),
            // Self::StarryNight(night) => night.on_direction(direction),
        }
    }
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
        self.counter = if self.counter > 4 { 1 } else { self.counter }; // TODO: edit
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
            0 => {
                todo!("Power ON/OFF")
            }
            1 => World::fire_from(ws),
            2 => World::northen_light_from(ws),
            3 => World::matrix_from(ws),
            4 => World::voronoi_from(ws),
            _ => {
                defmt::panic!("World counter out of bounds")
            }
        }
    }
}
