#![feature(cell_update)]

use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use kiddo::KdTree;
use num_format::Locale;
use std::path::Path;

pub mod bin;
mod gamedata;
pub mod navigator;
mod opencv;
pub mod state;
pub mod surface;

pub type PixelKdTree = KdTree<f32, 2>;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;

pub fn inner_main() {
    println!("hello");
    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        4 => crate::bin::get_patch::get_patch_main(),
        _ => panic!("wtf"),
    }
}
