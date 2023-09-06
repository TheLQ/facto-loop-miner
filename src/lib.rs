#![feature(cell_update)]
#![feature(convert_float_to_int)]

use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use kiddo::KdTree;
use num_format::Locale;
use std::path::Path;

mod gamedata;
pub mod navigator;
mod opencv;
pub mod self_bin;
pub mod state;
pub mod surface;

pub type PixelKdTree = KdTree<f32, 2>;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;

// Fix simd-json eating all my ram
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn inner_main() {
    println!("hello");
    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        4 => crate::self_bin::get_patch::get_patch_main(),
        _ => panic!("wtf"),
    }
}
