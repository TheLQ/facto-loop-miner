use facto_loop_miner_io::force_affinity;
use std::hint::black_box;
use std::ptr;

// Fix simd-json eating all my ram
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
// static GLOBAL: tcmalloc_better::TCMalloc = tcmalloc_better::TCMalloc;

fn main() {
    ptr::swap()
    force_affinity();
    facto_loop_miner::inner_main();
}
