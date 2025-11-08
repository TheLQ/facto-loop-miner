#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(iter_array_chunks)]
#![feature(test)]
#![feature(error_generic_member_access)]
#![allow(clippy::new_without_default)]

pub mod err;
pub mod io;
mod io_uring_common;
pub mod varray;

#[cfg(feature = "uring")]
mod io_bench;
#[cfg(feature = "uring")]
pub mod io_uring;
#[cfg(feature = "uring")]
mod io_uring_file_copying;

pub use io::{
    get_mebibytes_of_slice_usize, read_entire_file, read_entire_file_varray_mmap_lib,
    write_entire_file,
};
#[cfg(feature = "uring")]
pub use io_bench::checksum_vec_u8;

use libc::{CPU_COUNT, CPU_ISSET, CPU_SET, cpu_set_t};
use std::mem;
use std::ops::BitOr;

const NUM_CPUS: usize = 32;

pub fn force_affinity() {
    unsafe {
        get_affinity();

        let mut cpuset: cpu_set_t = mem::zeroed();
        for cpu in 0..NUM_CPUS {
            CPU_SET(cpu, &mut cpuset)
        }
        let res = libc::sched_setaffinity(0, size_of::<cpu_set_t>(), &cpuset);
        assert_eq!(res, 0);

        get_affinity();
    }
}

fn get_affinity() {
    unsafe {
        let mut existing_cpuset: cpu_set_t = mem::zeroed();
        let res = libc::sched_getaffinity(0, size_of::<cpu_set_t>(), &mut existing_cpuset);
        println!("get affinity res {res}");
        let enabled_cpus = CPU_COUNT(&existing_cpuset);

        let mut setsize = 0usize;
        for i in 0..NUM_CPUS {
            if CPU_ISSET(i, &existing_cpuset) {
                let mutator = 1usize.rotate_left(i as u32);
                // println!("applying {mutator:b}");
                setsize = setsize.bitor(mutator);
            }
        }
        println!("CPUs enabled {enabled_cpus:>2} encoded {setsize:b}");
    }
}
