#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![feature(error_generic_member_access)]
#![allow(clippy::new_without_default)]

pub mod err;
pub mod varray;

#[cfg(test)]
mod io_bench;

mod io;
pub mod io_uring;
mod io_uring_common;
mod io_uring_file;

pub use io::{
    get_mebibytes_of_slice_usize, read_entire_file, read_entire_file_varray_mmap_lib,
    write_entire_file,
};
use num_format::Locale;

pub const LOCALE: Locale = Locale::en;
