#![feature(convert_float_to_int)]

use facto_loop_miner::state::machine_v1::new_v1_machine;
use facto_loop_miner::surface::pixel::generate_lookup_image;
use num_format::Locale;
use std::path::Path;

// Fix simd-json eating all my ram
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    println!("hello");
    let root_dir = Path::new("work");

    match 1 {
        1 => new_v1_machine().start(root_dir),
        3 => generate_lookup_image(),
        _ => panic!("wtf"),
    }
}
