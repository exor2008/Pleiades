use crate::ws2812::Ws2812;
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
