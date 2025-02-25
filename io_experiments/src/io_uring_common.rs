use std::alloc::{alloc, Layout};
use std::mem;

/// TODO: New Macbook's are 16k?
/// But this is a runtime environment option. Usually.
pub const PAGE_SIZE: usize = 4096;
const BUF_RING_ID: u16 = 42;

pub fn allocate_page_size_aligned<Container>() -> (*mut Container, usize) {
    let layout = Layout::from_size_align(mem::size_of::<Container>(), PAGE_SIZE)
        .expect("allocate_layout")
        .pad_to_align();

    // let ptr = unsafe { alloc(layout) as *mut Container };
    // if ptr.is_null() {
    //     panic!("allocate");
    // }
    // (ptr, ptr as usize)
    let res = allocate_array_page_size_aligned::<Container>(1);
    (res.0 as *mut Container, res.1)

    // let mmap_ptr = unsafe {
    //     libc::mmap(
    //         ptr::null_mut(),
    //         layout.size(),
    //         // ACL required to use it
    //         libc::PROT_READ | libc::PROT_WRITE,
    //         // TODO: libc::MAP_HUGETLB | libc::MAP_HUGE_2MB
    //         // Required mode, Prepopulate with file content
    //         libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_HUGETLB | libc::MAP_HUGE_2MB,
    //         0,
    //         0,
    //     )
    // };
    // if mmap_ptr == libc::MAP_FAILED {
    //     panic!("mmap failed");
    // }
    // (mmap_ptr as *mut Container, mmap_ptr as usize)
}

pub fn allocate_array_page_size_aligned<Entry>(count: usize) -> (*mut Entry, usize) {
    let layout = Layout::from_size_align(count * mem::size_of::<Entry>(), PAGE_SIZE)
        .expect("allocate_layout");
    let ptr = unsafe { alloc(layout) as *mut Entry };
    if ptr.is_null() {
        panic!("allocate");
    }
    (ptr, ptr as usize)
}

/// Struct Kernel passes from submission queue to completion queue
/// We have u64 *total* to work with
#[derive(Default, Clone, Copy)]
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
