#![feature(cell_update)]
#![feature(convert_float_to_int)]
#![feature(iter_array_chunks)]
#![feature(portable_simd)]
#![feature(exclusive_range_pattern)]
#![feature(slice_pattern)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(slice_flatten)]
#![feature(array_chunks)]
#![allow(unused)]

extern crate core;

use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use kiddo::KdTree;
use num_format::Locale;
use num_traits::PrimInt;
use std::fmt;
use std::ops::{Rem, Sub};
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

    match 5 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        4 => crate::self_bin::get_patch::get_patch_main(),
        5 => admiral::remote_game::admiral(),
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
