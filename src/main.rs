#![feature(convert_float_to_int)]

mod gamedata;
mod opencv;
mod state;
mod surface;

use crate::state::machine_v1::new_v1_machine;
use crate::surface::pixel::generate_lookup_image;
use num_format::Locale;
use std::path::Path;

// Fix simd-json eating all my ram
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;

fn main() {
    println!("hello");
    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        _ => panic!("wtf"),
    }
}
