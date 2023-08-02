#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]

pub mod apds9960;
pub mod color;
pub mod led_matrix;
pub mod perlin;
pub mod wifi;
pub mod world;
pub mod ws2812;
