// Fix simd-json eating all my ram
// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// use std::time::{SystemTime, UNIX_EPOCH};
//
// struct DummyThing {
//     actual: u128,
//     outer: Vec<u128>,
// }
//
// impl Default for DummyThing {
//     fn default() -> Self {
//         DummyThing {
//             actual: SystemTime::now()
//                 .duration_since(UNIX_EPOCH)
//                 .unwrap()
//                 .as_nanos(),
//             outer: vec![
//                 SystemTime::now()
//                     .duration_since(UNIX_EPOCH)
//                     .unwrap()
//                     .as_nanos(),
//                 SystemTime::now()
//                     .duration_since(UNIX_EPOCH)
//                     .unwrap()
//                     .as_nanos(),
//                 SystemTime::now()
//                     .duration_since(UNIX_EPOCH)
//                     .unwrap()
//                     .as_nanos(),
//             ],
//         }
//     }
// }
//
// #[no_mangle]
// fn work() {}
//
// #[no_mangle]
// fn process_slow() {}
//
// #[no_mangle]
// fn process_fast() {
//     let res = take_by_generic()
//     println!("fast {}", )
// }
//
// fn take_by_generic<'a>(test: impl IntoIterator<Item = &'a DummyThing>) -> u128 {
//     let mut result = 0;
//     for item in test {
//         result += item.actual / item.outer.iter().count() as u128
//     }
//     result
// }
//
// fn take_by_slice<'a>(test: &[DummyThing]) -> u128 {
//     let mut result = 0;
//     for item in test {
//         result += item.actual / item.outer.iter().count() as u128
//     }
//     result
// }

fn main() {
    facto_loop_miner::inner_main();
}
