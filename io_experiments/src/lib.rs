#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![feature(error_generic_member_access)]
#![allow(clippy::new_without_default)]

pub mod err;
pub mod varray;

// #[cfg(test)]
mod io_bench;

mod io;
pub mod io_uring;
mod io_uring_common;
// mod io_uring_file;
mod io_uring_file_copying;

use crate::io_bench::checksum_vec_u8;
use crate::io_uring::io_uring_main;
pub use io::{
    get_mebibytes_of_slice_usize, read_entire_file, read_entire_file_varray_mmap_lib,
    write_entire_file,
};
use num_format::{Locale, ToFormattedString};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info, Level};

pub const LOCALE: Locale = Locale::en;

pub fn io_experiment_main() {
    let tracing_format = tracing_subscriber::fmt::format().compact();

    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        .with_max_level(Level::INFO)
        .compact()
        .init();

    tracing::debug!("hello io_experiment");

    let file_path: PathBuf = match 3 {
        1 => "/xf-megafile/data/pages.db.index",
        2 => "/boot/initrd.img-6.1.0-9-amd64",
        3 => {
            "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step00-import/pixel-xy-indexes.dat"
        }
        4 => "/hugetemp/pixel-xy-indexes.dat",
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
    let output = read_entire_file(path, true).unwrap();
    let checksum = checksum_vec_u8(output);

    let time = Instant::now() - watch;
    info!(
        "InnerMain u8_unconverted {} in {}",
        path.display(),
        time.as_millis().to_formatted_string(&LOCALE)
    );
}
