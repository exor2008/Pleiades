use crate::led_matrix;
use crate::perlin;

use crate::color::{Color, ColorGradient};
use crate::ws2812::Ws2812;
use core::marker::PhantomData;
use embassy_rp::pio::Instance;
use embassy_time::{Duration, Ticker};
use heapless::Vec;
use pleiades_macro_derive::{Flush, From, Into};
use smart_leds::RGB8;

use crate::world::{Flush, Tick};

#[derive(Flush, Into, From)]
pub struct Matrix<
    'a,
    P: Instance,
    const S: usize,
    const L: usize,
    const C: usize,
    const N: usize,
    const N2: usize,
> {
    led: led_matrix::LedMatrix<'a, P, S, L, C, N>,
    colormap: ColorGradient<C>,
    letters: Vec<Letters, N2>,
    ticker: Ticker,
    rnd_col: Vec<usize, C>,
    t: usize,
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize>
    Matrix<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    pub fn new(ws: Ws2812<'a, P, S, N>) -> Self {
        let led = led_matrix::LedMatrix::new(ws);
        let ticker = Ticker::every(Duration::from_millis(30));
        let mut colormap = ColorGradient::new();
        let letters: Vec<Letters, N2> = Vec::new();
        let rnd_col: Vec<usize, C> = Vec::new();

        colormap.add_color(Color::new(0.0, RGB8::new(0, 0, 0)));
        colormap.add_color(Color::new(0.8, RGB8::new(5, 50, 5)));
        colormap.add_color(Color::new(1.01, RGB8::new(50, 150, 50)));

        Self {
            led,
            colormap,
            letters,
            ticker,
            rnd_col,
            t: 0,
        }
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize> Tick
    for Matrix<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    async fn tick(&mut self) {
        self.led.clear();

        self.spawn_letters();
        self.process_letters();
        self.remove_letters();

        self.letters.iter_mut().for_each(|letter| match letter {
            Letters::Falling(ref mut l) => {
                let color = self.colormap.get(l.temperature);
                self.led.write(l.x, l.y, color);
            }
            Letters::Stationary(ref mut l) => {
                let color = self.colormap.get(l.temperature);
                self.led.write(l.x, l.y, color);
            }
        });

        self.t = self.t.wrapping_add(1);
        self.ticker.next().await;
    }
}

impl<'a, P, const S: usize, const L: usize, const C: usize, const N: usize, const N2: usize>
    Matrix<'a, P, S, L, C, N, N2>
where
    P: Instance,
{
    fn spawn_letters(&mut self) {
        let chance = perlin::rand_float(0.0, 1.0);

        if !self.letters.is_full() && chance >= 0.7 {
            let x: usize = self.next_rnd_column();

            let cool_rate = perlin::rand_float(0.005, 0.015);
            let temperature = perlin::rand_float(0.8, 1.0);
            let move_after = perlin::rand_uint(1, 12) as usize;

            let letter = Letters::new_falling(x, 0, move_after, temperature, cool_rate);
            if let Err(_) = self.letters.push(letter) {
                defmt::error!("Pushing letter in full vector while spawning.");
            }
        }
    }

    fn process_letters(&mut self) {
        let mut tmp_letters: Vec<Letters, N> = Vec::new();

        self.letters.iter_mut().for_each(|letter| match letter {
            Letters::Falling(ref mut l) => {
                if l.down() {
                    let letter =
                        Letters::new_stationary(l.x, l.y - 1, l.temperature - 0.2, l.cool_rate);
                    if let Err(_) = tmp_letters.push(letter) {
                        defmt::error!("Pushing letter in full tmp vector.")
                    }
                }
            }

            Letters::Stationary(ref mut l) => {
                l.cool();
            }
        });

        if N2 - self.letters.len() >= tmp_letters.len() {
            self.letters.extend(tmp_letters);
        } else {
            defmt::error!(
                "Pushing letter in full vector. Vector len: {}, new letters len: {}",
                self.letters.len(),
                tmp_letters.len()
            )
        }
    }

    fn remove_letters(&mut self) {
        self.letters.retain(|letter| match letter {
            Letters::Falling(l) => l.y < L,
            Letters::Stationary(l) => l.temperature > 0.0,
        });
    }

    fn next_rnd_column(&mut self) -> usize {
        if self.rnd_col.is_empty() {
            self.rnd_col = (0..C).collect();
            perlin::shuffle(&mut self.rnd_col);
        }
        self.rnd_col.remove(self.rnd_col.len() - 1)
    }
}

struct Falling;
struct Stationary;

enum Letters {
    Falling(Letter<Falling>),
    Stationary(Letter<Stationary>),
}

impl Letters {
    fn new_stationary(x: usize, y: usize, temperature: f32, cool_rate: f32) -> Self {
        Self::Stationary(Letter {
            x,
            y,
            move_after: Default::default(),
            move_after_init: Default::default(),
            temperature,
            cool_rate,
            star_type: Default::default(),
        })
    }

    fn new_falling(
        x: usize,
        y: usize,
        move_after: usize,
        temperature: f32,
        cool_rate: f32,
    ) -> Self {
        Self::Falling(Letter {
            x,
            y,
            move_after,
            move_after_init: move_after,
            temperature,
            cool_rate,
            star_type: Default::default(),
        })
    }
}

#[derive(Debug)]
struct Letter<LetterType> {
    x: usize,
    y: usize,
    move_after: usize,
    move_after_init: usize,
    temperature: f32,
    cool_rate: f32,
    star_type: PhantomData<LetterType>,
}

impl Letter<Falling> {
    fn down(&mut self) -> bool {
        match self.move_after == 0 {
            true => {
                self.y += 1;
                self.move_after = self.move_after_init;
                self.temperature += perlin::rand_float(-0.2, 0.2);
                self.temperature = self.temperature.clamp(0.8, 1.0);
                true
            }
            false => {
                self.move_after -= 1;
                false
            }
        }
    }
}

impl Letter<Stationary> {
    fn cool(&mut self) {
        match self.move_after == 0 {
            true => {
                self.temperature -= self.cool_rate;
                self.temperature = self.temperature.max(0.0);

                self.move_after = self.move_after_init;
            }
            false => {
                self.move_after -= 1;
            }
        }
    }
}
