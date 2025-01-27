#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(portable_simd)]
#![feature(error_generic_member_access)]
//
// lints
//
#![allow(unused_variables)]
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
use num_format::Locale;
use num_traits::PrimInt;
use std::path::Path;

mod gamedata;
pub mod navigator;
mod opencv;
pub mod simd;
pub mod simd_diff;
pub mod state;
pub mod surface;
pub mod surfacev;
pub mod util;

// pub type PixelKdTree = KdTree<f32, 2usize>;
pub type PixelKdTree = float::kdtree::KdTree<f32, usize, 2usize, 32, u32>;

// TODO: Remove now duplicated
pub const LOCALE: Locale = Locale::en;
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

/// what is this called?
pub fn bucket_div<N>(value: N, bucket_size: N) -> N
where
    N: PrimInt,
{
    (value - (value % bucket_size)) / bucket_size
}
