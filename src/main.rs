#![feature(convert_float_to_int)]

use facto_loop_miner::state::machine_v1::new_v1_machine;
use facto_loop_miner::surface::pixel::generate_lookup_image;
use std::path::Path;

// Fix simd-json eating all my ram
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    facto_loop_miner::innser_main();
}
