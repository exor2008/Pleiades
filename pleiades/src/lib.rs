#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]
#![feature(impl_trait_projections)]

pub mod apds9960;
pub mod color;
pub mod firmware;
pub mod http;
pub mod led_matrix;
pub mod perlin;
pub mod wifi;
pub mod world;
pub mod ws2812;
