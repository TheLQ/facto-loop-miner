#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![feature(error_generic_member_access)]
#![allow(clippy::new_without_default)]

extern crate core;

pub mod err;
pub mod varray;

// #[cfg(test)]
mod io_bench;

mod io;
pub mod io_uring;
mod io_uring_common;
// mod io_uring_file;
mod io_uring_file_copying;

use crate::err::VIoError;
use crate::io::{read_entire_file_mmap_copy, read_entire_file_usize_mmap_custom};
use crate::io_bench::{checksum_vec_u8, checksum_vec_usize};
use crate::io_uring::io_uring_main;
pub use io::{
    get_mebibytes_of_slice_usize, read_entire_file, read_entire_file_varray_mmap_lib,
    write_entire_file,
};
use memmap2::MmapOptions;
use num_format::{Locale, ToFormattedString};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{info, Level};

pub const LOCALE: Locale = Locale::en;

pub fn io_experiment_main() {
    let tracing_format = tracing_subscriber::fmt::format().compact();

    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        .with_max_level(Level::TRACE)
        .compact()
        .init();

    tracing::debug!("hello io_experiment");

    let file_path: PathBuf = match 3 {
        1 => "/xf-megafile/data/pages.db.index",
        2 => "/boot/initrd.img-6.1.0-9-amd64",
        3 => {
            "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step00-import/pixel-xy-indexes.dat"
        }
        4 => "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step10-base/pixel-xy-indexes.dat",
        5 => "/hugetemp/pixel-xy-indexes.dat",
        _ => unimplemented!("fuck"),
    }
    .into();
    info!("io_experiments processing {}", file_path.display());

    match 1 {
        1 => test_u8(&file_path),
        2 => io_uring_main(&file_path).expect("Asd"),
        _ => panic!("nope"),
    }
}

fn test_u8(path: &Path) {
    let watch = Instant::now();
    // let output = read_entire_file(path, true).unwrap();
    // let checksum = checksum_vec_u8(output);

    info!("asdf");
    let stopwatch = Instant::now();
    let output = read_entire_file_usize_mmap_custom(path, true, true, true).unwrap();
    // let output = read_entire_file_mmap_copy(path).unwrap();
    info!(
        "file read in {}",
        (Instant::now() - stopwatch)
            .as_secs()
            .to_formatted_string(&LOCALE)
    );

    // let tester = MmapOptions::new()
    //     .huge(None)
    //     .len(1024 ^ 3)
    //     .map_anon()
    //     .map_err(VIoError::io_error(path))
    //     .unwrap();
    // let checksum = checksum_vec_u8(&tester[..]);
    // info!(
    //     "tester checksum in {}",
    //     (Instant::now() - stopwatch)
    //         .as_secs()
    //         .to_formatted_string(&LOCALE)
    // );

    sleep(Duration::new(999999, 0));

    let stopwatch = Instant::now();
    let checksum = checksum_vec_usize(&output);
    info!(
        "checksum in {}",
        (Instant::now() - stopwatch)
            .as_secs()
            .to_formatted_string(&LOCALE)
    );
    println!("checksum {}", checksum);

    let time = Instant::now() - watch;
    info!(
        "InnerMain u8_unconverted {} in {}",
        path.display(),
        time.as_millis().to_formatted_string(&LOCALE)
    );
}
