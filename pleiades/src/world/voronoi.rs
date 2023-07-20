use super::OnDirection;
use crate::apds9960::Direction;
use crate::color::{Color, ColorGradient};
use crate::led_matrix;
use crate::perlin;
use crate::world::{Flush, Tick};
use crate::ws2812::Ws2812;
use core::f32::consts::PI;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Ticker};
use heapless::Vec;
use micromath::F32Ext;
use pleiades_macro_derive::{Flush, From, Into};
use smart_leds::RGB8;

const POINTS: usize = 5;
const TIMES_OF_DAY: usize = 3;

#[derive(Flush, Into, From)]
pub struct Voronoi<'a, P: Instance, const S: usize, const L: usize, const C: usize, const N: usize>
{
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    buffer_new: [[RGB8; L]; C],
    buffer_old: [[RGB8; L]; C],
    model: Model<L, C>,
    ticker: Ticker,
    t: usize,
    time: f32,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize>
    Voronoi<'a, P, S, L, C, N>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let ticker = Ticker::every(Duration::from_millis(30));
        let time = PI / 2.0;
        let mut model: Model<L, C> = Model::new();
        let buffer_new = model.step(time);
        let buffer_old = buffer_new.clone();

        Self {
            led,
            model,
            ticker,
            buffer_new,
            buffer_old,
            t: 0,
            time: PI / 2.0,
        }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> Tick
    for Voronoi<'a, P, S, L, C, N>
where
    P: Instance,
{
    async fn tick(&mut self) {
        self.led.clear();

        self.time += 1e-3;
        if self.t == 0 {
            self.buffer_old = self.buffer_new;
            self.buffer_new = self.model.step(self.time);
        }

        let r = self.t as f32 / 10.0;

        for x in 0..C {
            for y in 0..L {
                let c1 = Color::new(0.0, self.buffer_old[x][y]);
                let c2 = Color::new(1.01, self.buffer_new[x][y]);

                let mut grad: ColorGradient<2> = ColorGradient::new();
                grad.add_color(c1);
                grad.add_color(c2);
                self.led.write(x, y, grad.get(r));
            }
        }

        self.t += 1;
        self.t = if self.t > 10 { 0 } else { self.t };
        self.ticker.next().await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize> OnDirection
    for Voronoi<'a, P, S, L, C, N>
where
    P: Instance,
{
    fn on_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                todo!("Implemnt UP for Voronoi")
            }
            Direction::Down => {
                todo!("Implemnt DOWN for Voronoi")
            }
        }
    }
}

struct Point<const L: usize, const C: usize> {
    x: isize,
    y: isize,
    x_shift: isize,
    y_shift: isize,
    colormap: ColorGradient<TIMES_OF_DAY>,
}

impl<const L: usize, const C: usize> Point<L, C> {
    fn new(x: isize, y: isize, colormap: ColorGradient<TIMES_OF_DAY>) -> Self {
        Point {
            x,
            y,
            x_shift: perlin::rand_int(-1, 2) as isize,
            y_shift: perlin::rand_int(-1, 2) as isize,
            colormap,
        }
    }

    fn go(&mut self) {
        self.x = Point::<L, C>::wrap_go(self.x, self.x_shift, C as isize);
        self.y = Point::<L, C>::wrap_go(self.y, self.y_shift, L as isize);
    }

    fn wrap_go(var: isize, shift: isize, border: isize) -> isize {
        let var = var + shift;
        let var = if var < 0 { border - 1 } else { var };
        let var = if var >= border - 1 { 0 } else { var };
        var
    }

    fn change_dir(&mut self) {
        self.x_shift = perlin::rand_int(-1, 2) as isize;
        self.y_shift = perlin::rand_int(-1, 2) as isize;
    }
}

struct Model<const L: usize, const C: usize> {
    points: Vec<Point<L, C>, POINTS>,
}

impl<const L: usize, const C: usize> Model<L, C> {
    fn new() -> Self {
        let mut points: Vec<Point<L, C>, POINTS> = Vec::new();

        let mut cm1 = ColorGradient::new();

        cm1.add_color(Color::new(0.0, RGB8::new(1, 52, 89)));
        cm1.add_color(Color::new(0.5, RGB8::new(122, 39, 1)));
        cm1.add_color(Color::new(1.01, RGB8::new(108, 194, 189)));

        let mut cm2 = ColorGradient::new();
        cm2.add_color(Color::new(0.0, RGB8::new(3, 32, 52)));
        cm2.add_color(Color::new(0.5, RGB8::new(227, 81, 0)));
        cm2.add_color(Color::new(1.01, RGB8::new(90, 129, 158)));

        let mut cm3 = ColorGradient::new();
        cm3.add_color(Color::new(0.0, RGB8::new(7, 115, 167)));
        cm3.add_color(Color::new(0.5, RGB8::new(254, 83, 0)));
        cm3.add_color(Color::new(1.01, RGB8::new(125, 122, 162)));

        let mut cm4 = ColorGradient::new();
        cm4.add_color(Color::new(0.0, RGB8::new(1, 1, 1)));
        cm4.add_color(Color::new(0.5, RGB8::new(254, 164, 1)));
        cm4.add_color(Color::new(1.01, RGB8::new(246, 126, 125)));

        let mut cm5 = ColorGradient::new();
        cm5.add_color(Color::new(0.0, RGB8::new(0, 12, 12)));
        cm5.add_color(Color::new(0.5, RGB8::new(254, 218, 121)));
        cm5.add_color(Color::new(1.01, RGB8::new(255, 193, 167)));

        let mut colormaps: Vec<ColorGradient<TIMES_OF_DAY>, POINTS> = Vec::new();
        unsafe {
            colormaps.push_unchecked(cm1);
            colormaps.push_unchecked(cm2);
            colormaps.push_unchecked(cm3);
            colormaps.push_unchecked(cm4);
            colormaps.push_unchecked(cm5);
        }

        for cm in colormaps.into_iter() {
            let x = perlin::rand_uint(0, C as u32) as isize;
            let y = perlin::rand_uint(0, L as u32) as isize;
            unsafe {
                points.push_unchecked(Point::new(x, y, cm));
            }
        }
        Model { points }
    }

    fn step(&mut self, time: f32) -> [[RGB8; L]; C] {
        let mut index_matrix = [[0usize; L]; C];
        let mut buffer = [[RGB8::default(); L]; C];
        let sin = (time.sin() + 1.0) / 2.0; // [0..1]

        for x in 0..C {
            for y in 0..L {
                // Vector of distances from every LED to every Point
                let dist: Vec<isize, POINTS> = self
                    .points
                    .iter()
                    .map(|p| {
                        let x_diff = x as isize - p.x;
                        let y_diff = y as isize - p.y;
                        x_diff * x_diff + y_diff * y_diff
                    })
                    .collect();

                // Find index of closest Point
                if let Some(index) = dist
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.cmp(b))
                    .map(|(index, _)| index)
                {
                    index_matrix[x][y] = index;
                }
            }
        }

        for x in 0..C {
            for y in 0..L {
                let idx1 = index_matrix[x][y];
                let point = &self.points[idx1];

                if x == 0 || y == 0 || x == C - 1 || y == L - 1 {
                    buffer[x][y] = point.colormap.get(sin);
                } else {
                    for x_shift in -1..=1 {
                        for y_shift in -1..=1 {
                            if x_shift != 0 && y_shift != 0 {
                                let x_idx = (x as isize + x_shift) as usize;
                                let y_idx = (y as isize + y_shift) as usize;
                                let idx2 = index_matrix[x_idx][y_idx];
                                if idx1 != idx2 {
                                    buffer[x][y] = point.colormap.get(sin);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.points.iter_mut().for_each(|p| {
            if perlin::rand_float(0.0, 1.0) > 0.6 {
                p.change_dir();
            }
            p.go();
        });

        buffer
    }
}
