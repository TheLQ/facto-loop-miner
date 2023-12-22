#![allow(unused_variables)]
#![allow(dead_code)]

#![feature(array_chunks)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![feature(error_generic_member_access)]

pub mod err;
mod io_bench;
pub mod varray;

mod io;
pub use io::{
    get_mebibytes_of_slice_usize, read_entire_file, read_entire_file_varray_mmap_lib,
    write_entire_file,
};
