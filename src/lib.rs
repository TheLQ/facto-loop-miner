// #![feature(cell_update)]
// #![feature(convert_float_to_int)]
#![feature(iter_array_chunks)]
#![feature(portable_simd)]
#![feature(exclusive_range_pattern)]
// #![feature(slice_pattern)]
#![feature(iterator_try_collect)]
// #![feature(iterator_try_reduce)]
// #![feature(slice_flatten)]
#![feature(array_chunks)]
#![feature(error_generic_member_access)]
#![feature(int_roundings)]
// #![feature()]
// #![feature()]
// #![feature()]
// #![feature()]
// #![feature()]

// lints
#![allow(unused_variables)]
#![allow(dead_code)]
// todo: This is for something() { Ok(()) } , only testing
#![allow(clippy::unnecessary_wraps)]
// styling
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
use kiddo::KdTree;
use num_format::Locale;
use num_traits::PrimInt;
use std::path::Path;
use tracing::Level;

mod admiral;
mod gamedata;
pub mod navigator;
mod opencv;
pub mod self_bin;
pub mod simd;
pub mod simd_diff;
pub mod state;
pub mod surface;
pub mod surfacev;
pub mod util;

pub type PixelKdTree = KdTree<f32, 2>;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;
pub fn inner_main() {
    let tracing_format = tracing_subscriber::fmt::format().compact();

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .compact()
        .init();

    tracing::debug!("hello");
    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        4 => self_bin::get_patch::get_patch_main(),
        5 => admiral::client::admiral(),
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
