use std::alloc::{alloc, Layout};
use std::mem;

/// TODO: New Macbook's are 16k?
/// But this is a runtime environment option. Usually.
pub const PAGE_SIZE: usize = 4096;
type PageSlice = [u8; PAGE_SIZE];
const BUF_RING_ID: u16 = 42;

pub fn allocate_page_size_aligned<Container>() -> (*mut Container, usize) {
    // let layout = Layout::from_size_align(mem::size_of::<C>(), PAGE_SIZE).expect("allocate_layout");
    // let ptr = unsafe { alloc(layout) as *mut C };
    // if ptr.is_null() {
    //     panic!("allocate");
    // }
    // (ptr, ptr as usize)
    allocate_array_page_size_aligned::<Container, Container>(1)
}

pub fn allocate_array_page_size_aligned<Container, Entry>(count: usize) -> (*mut Container, usize) {
    let layout = Layout::from_size_align(count * mem::size_of::<Entry>(), PAGE_SIZE)
        .expect("allocate_layout");
    let ptr = unsafe { alloc(layout) as *mut Container };
    if ptr.is_null() {
        panic!("allocate");
    }
    (ptr, ptr as usize)
}

pub fn log_debug(value: &str) {
    println!("{}", value);
}

/// Struct Kernel passes from submission queue to completion queue
/// We have u64 *total* to work with
#[derive(Default)]
#[repr(C)]
pub struct IoUringEventData {
    b: u16,
    c: u16,
    d: u16,
    // TODO: Only works here, we are overwriting something...
    pub buf_index: u8,
}

impl IoUringEventData {
    pub fn from_buf_index(buffer_index: u8) -> Self {
        Self {
            buf_index: buffer_index,
            ..Self::default()
        }
    }
}
