extern crate test;

use std::path::PathBuf;

use crate::io::{
    map_u8_to_usize_iter_ref, map_u8_to_usize_slice, read_entire_file,
    read_entire_file_usize_aligned_vec, read_entire_file_usize_mmap_custom,
    read_entire_file_usize_read_then_iter, read_entire_file_usize_transmute_broken,
    read_entire_file_varray_mmap_lib, USIZE_BYTES,
};

fn input_path() -> PathBuf {
    PathBuf::from(BENCH_XY_PATH)
}

const BENCH_RAW_XY_BUFFER: &[u8] =
    include_bytes!("../../work/out0/step10-base/pixel-xy-indexes.dat");
const BENCH_XY_PATH: &str = "work/out0/step10-base/pixel-xy-indexes.dat";

#[bench]
fn bench_included_minimum_test(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("start");
        let total: usize = BENCH_RAW_XY_BUFFER.iter().map(|v| *v as usize).sum();
        println!("total {}", total);
        total
    })
}

#[bench]
fn bench_included_map_slice(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("start");
        let mut output: Vec<usize> = vec![0; BENCH_RAW_XY_BUFFER.len() / USIZE_BYTES];
        map_u8_to_usize_slice(BENCH_RAW_XY_BUFFER, &mut output);
        injest_value(output)
    })
}

// slow lol
#[bench]
fn bench_included_map_iter(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("start");
        let output: Vec<usize> = map_u8_to_usize_iter_ref(BENCH_RAW_XY_BUFFER.iter())
            .into_iter()
            .collect();
        injest_value(output)
    })
}

#[bench]
fn bench_read_minimum_unconverted(bencher: &mut test::Bencher) {
    println!("init");
    bencher.iter(|| {
        println!("interation");
        let output = read_entire_file(&input_path()).unwrap();
        println!("output");
        injest_value_TESTING_u8(output)
    })
}

#[bench]
fn bench_read_aligned_vec(bencher: &mut test::Bencher) {
    println!("init");
    bencher.iter(|| {
        println!("interation");
        let output = read_entire_file_usize_aligned_vec(&input_path()).unwrap();
        println!("output");
        injest_value(output)
    })
}

#[bench]
fn bench_read_transmute_broken(bencher: &mut test::Bencher) {
    if env::var("BROKEN_TEST").is_err() {
        println!("not doing broking test transmute_broken");
        return;
    }
    bencher.iter(|| {
        let output = read_entire_file_usize_transmute_broken(&input_path()).unwrap();
        injest_value(output)
    });
}

#[bench]
fn bench_read_mmap_lib(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("interation");
        let output = read_entire_file_varray_mmap_lib(&input_path()).unwrap();
        let bench_result: usize = output.as_slice().iter().sum1().unwrap();
        bench_result
    });
}

#[bench]
fn bench_read_mmap_custom(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("interation");
        let output = read_entire_file_usize_mmap_custom(&input_path()).unwrap();
        let bench_result: usize = output.iter().sum1().unwrap();
        drop_mmap_vec(output);
        bench_result
    });
}

#[bench]
fn bench_read_iter(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("interation");
        let output = read_entire_file_usize_read_then_iter(&input_path()).unwrap();
        injest_value(output)
    });
}

fn injest_value(output: Vec<usize>) -> usize {
    let total: usize = output.iter().sum();
    println!("total {}", total);
    assert_eq!(total, 224321692961);
    total
}

fn injest_value_testing_u8(output: Vec<u8>) -> u8 {
    let total: u8 = output.iter().sum();
    println!("total {}", total);
    // assert_eq!(total, 224321692961);
    total
}
