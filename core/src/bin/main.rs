use facto_loop_miner_io::force_affinity;

// Fix simd-json eating all my ram
// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
// static GLOBAL: tcmalloc_better::TCMalloc = tcmalloc_better::TCMalloc;

fn main() {
    force_affinity();
    facto_loop_miner::inner_main();
}
