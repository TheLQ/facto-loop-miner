// Fix simd-json eating all my ram
#[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    facto_loop_miner::inner_main();
}
