#![feature(convert_float_to_int)]

mod gamedata;
mod state;
mod surface;

use crate::state::machine_v1::new_v1_machine;
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

    new_v1_machine(root_dir).start(root_dir);
}
