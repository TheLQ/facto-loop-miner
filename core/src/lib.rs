#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(portable_simd)]
#![feature(error_generic_member_access)]
//
// lints
//
// #![allow(dead_code)]
// todo: This is for something() { Ok(()) } , only testing
#![allow(clippy::unnecessary_wraps)]
//
// styling
//
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::missing_errors_doc)]
// TODO: <<<< SHOULD REVIEW
#![allow(clippy::cast_precision_loss)]
// #![deny(clippy::pedantic)]
// #![deny(clippy::all)]
#![allow(clippy::let_and_return)]
#![allow(clippy::result_large_err)]

// TODO #![deny(future-incompatible)]
// TODO #![deny(let-underscore)]
// TODO #![deny(nonstandard-style)]

extern crate core;

use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use facto_loop_miner_common::log_init_trace;
use kiddo::float;
use std::path::Path;

mod gamedata;
mod navigator;
mod opencv;
mod simd;
// mod simd_diff;
mod state;
mod surface;
mod surfacev;
mod util;

// pub type PixelKdTree = KdTree<f32, 2usize>;
type PixelKdTree = float::kdtree::KdTree<f32, usize, 2usize, 32, u32>;

// TODO: REmove now duplicated
pub const TILES_PER_CHUNK: usize = 32;
pub fn inner_main() {
    log_init_trace();

    tracing::debug!("hello");
    // let mut data = String::new();
    // stdin().read_line(&mut data).expect("asd");

    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        _ => panic!("wtf"),
    }
}
