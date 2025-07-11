#![allow(unused_imports)]

extern crate test;

use crate::LOCALE;
use crate::err::VPathUnwrapper;
use crate::io::{
    USIZE_BYTES, drop_mmap_vec, map_u8_to_usize_iter, map_u8_to_usize_iter_ref,
    map_u8_to_usize_slice, read_entire_file, read_entire_file_usize_aligned_vec,
    read_entire_file_usize_mmap_custom, read_entire_file_usize_transmute_broken,
    read_entire_file_varray_mmap_lib,
};
use crate::io_uring::IoUring;
use crate::io_uring_file_copying::IoUringFileCopying;
use num_format::ToFormattedString;
use std::env;
use std::path::PathBuf;

fn input_path() -> PathBuf {
    PathBuf::from(BENCH_XY_PATH)
}

// step00-import
// step10-base
const BENCH_RAW_XY_BUFFER: &[u8] = &[0u8; 1]; //include_bytes!("../../work/out0/step10-base/pixel-xy-indexes.dat");
const BENCH_XY_PATH: &str = "work/out0/step00-import/pixel-xy-indexes.dat";
// const BENCH_XY_PATH: &str = "work/out0/step10-base/pixel-xy-indexes.dat";

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
        checksum_vec_usize(&output)
    })
}

// slow lol
#[bench]
fn bench_included_map_iter(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("start");
        let output: Vec<usize> = map_u8_to_usize_iter_ref(BENCH_RAW_XY_BUFFER.iter()).collect();
        checksum_vec_usize(&output)
    })
}

#[bench]
fn bench_read_minimum_unconverted(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let output = read_entire_file(&input_path(), true).unwrap_path(&input_path());
        checksum_vec_u8(&output)
    })
}

#[bench]
fn bench_read_aligned_vec(bencher: &mut test::Bencher) {
    println!("init");
    bencher.iter(|| {
        println!("interation");
        let path = input_path();
        let output = read_entire_file_usize_aligned_vec(&input_path()).unwrap_path(&input_path());
        println!("output");
        checksum_vec_usize(&output);
    })
}

#[bench]
fn bench_read_transmute_broken(bencher: &mut test::Bencher) {
    if env::var("BROKEN_TEST").is_err() {
        println!("not doing broking test transmute_broken");
        return;
    }
    bencher.iter(|| {
        let path = input_path();
        let output =
            read_entire_file_usize_transmute_broken(&input_path()).unwrap_path(&input_path());
        checksum_vec_usize(&output)
    });
}

#[bench]
fn bench_read_mmap_lib(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let output = read_entire_file_varray_mmap_lib(&input_path()).unwrap_path(&input_path());
        let bench_result = output.as_slice().iter().sum::<usize>();
        bench_result
    });
}

#[bench]
fn bench_read_mmap_custom(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        println!("interation {}", env::current_dir().unwrap().display());
        let output = read_entire_file_usize_mmap_custom(&input_path(), true, true, true)
            .unwrap_path(&input_path());
        let bench_result: usize = output.iter().sum::<usize>();
        drop_mmap_vec(output);
        bench_result
    });
}

#[bench]
fn bench_read_iter(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let data = read_entire_file(&input_path(), true).unwrap_path(&input_path());
        let output: Vec<usize> = map_u8_to_usize_iter(data).collect();
        checksum_vec_usize(&output)
    });
}

#[bench]
fn bench_read_slice(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let data = read_entire_file(&input_path(), true).unwrap_path(&input_path());
        let mut usize_data = vec![0; data.len() / USIZE_BYTES];
        map_u8_to_usize_slice(&data, &mut usize_data);
        checksum_vec_usize(&usize_data)
    });
}

#[bench]
fn bench_read_io_uring(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        let mut io_uring = IoUring::new();
        let mut io_uring_file = IoUringFileCopying::open(&input_path(), &mut io_uring).unwrap();
        io_uring_file.read_entire_file(&mut io_uring).unwrap();
        let usize_data = io_uring_file.into_result_as_usize(&mut io_uring);
        checksum_vec_usize(&usize_data)
    });
}

const EXPECTED_SUM: usize = 224321692961;

pub fn checksum_vec_usize(output: &[usize]) -> usize {
    let total: usize = output.iter().sum();
    println!("total {}", total);

    // assert_eq!(total, EXPECTED_SUM);
    if total != EXPECTED_SUM {
        eprintln!(
            "WARN: Expected {} got {}",
            total,
            EXPECTED_SUM.to_formatted_string(&LOCALE)
        );
    }
    total
}

pub fn checksum_vec_u8(output: &[u8]) -> usize {
    let total: usize = output.iter().map(|v| *v as usize).sum();
    println!("total {}", total);
    // assert_eq!(total, EXPECTED_SUM * USIZE_BYTES);
    total
}
