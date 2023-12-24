use libc::{iovec, size_t};
use std::alloc::{alloc, Layout};
use std::ffi::c_void;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error as IoError, ErrorKind};
use std::mem::{transmute, MaybeUninit};
use std::os::fd::AsRawFd;
use std::ptr::addr_of;
use std::{io, mem, slice};

use uring_sys2::{
    __kernel_timespec, io_uring, io_uring_buf, io_uring_buf_reg, io_uring_buf_ring,
    io_uring_buf_ring_add, io_uring_buf_ring_advance, io_uring_buf_ring_init,
    io_uring_buf_ring_mask, io_uring_cqe, io_uring_cqe_get_data, io_uring_cqe_seen,
    io_uring_get_sqe, io_uring_prep_nop, io_uring_prep_read, io_uring_prep_read_fixed,
    io_uring_prep_readv, io_uring_queue_exit, io_uring_queue_init, io_uring_register_buf_ring,
    io_uring_register_buffers, io_uring_sqe, io_uring_sqe_set_data, io_uring_submit,
    io_uring_submit_and_wait, io_uring_wait_cqe, io_uring_wait_cqes, IORING_CQE_BUFFER_SHIFT,
    IORING_FEAT_EXT_ARG,
};

/*
c file copy example: https://github.com/axboe/liburing/blob/master/examples/io_uring-cp.c
rust basic read example: https://github.com/Noah-Kennedy/liburing/blob/master/tests/test_read_file.rs
 */

pub fn c_mode_main() -> io::Result<()> {
    // init
    let mut ring = IoUring::new();

    let file_path = match 3 {
        1 => "/xf-megafile/data/pages.db.index",
        2 => "/boot/initrd.img-6.1.0-9-amd64",
        3 => "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step10-base/pixel-xy-indexes.dat",
        _ => unimplemented!("fuck"),
    }
    .to_string();

    // fill queue
    let mut io_file = IoUringFile::open(file_path)?;
    let read = io_file
        .create_submission_queue_read_all(&mut ring)
        .expect("Failed to create read");

    // loop {
    //     let sqe = unsafe { io_uring_get_sqe(&mut ring) };
    //     if sqe == std::ptr::null_mut() {
    //         // TODO: throw something
    //         break;
    //     }
    //     unsafe { io_uring_prep_read_fixed(sqe) };
    // }

    io_file.drain_completion_queue(&mut ring);

    // submit requests
    ring.submit();

    // fetch completions
    // let mut my_cqes = ring.wait_cqes();
    // ring.seen_cqe(&mut my_cqes[4]);
    // for cqe in my_cqes {
    //
    // }

    ring.wait_cqe();

    ring.exit();

    Ok(())
}

/// TODO: New Macbook's are 16k?
/// But this is a runtime environment option. Usually.
const PAGE_SIZE: usize = 4096;
type PageSlice = [u8; PAGE_SIZE];
const BUF_RING_ID: u16 = 42;

const BUF_RING_COUNT: usize = 50;
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
type BufRing = [io_uring; BUF_RING_COUNT];
type BackingBufEntry = [u8; BUF_RING_ENTRY_SIZE as usize];
type BackingBufRing = [BackingBufEntry; BUF_RING_COUNT as usize];

struct IoUring<const QUEUE_DEPTH: u32 = 25> {
    ring: io_uring,
    // buf_ring: Box<io_uring_buf_ring>,
    // backing_buf_ring: Box<BackingBufRing>,
}

fn allocate_page_size_aligned<Container>() -> (*mut Container, usize) {
    // let layout = Layout::from_size_align(mem::size_of::<C>(), PAGE_SIZE).expect("allocate_layout");
    // let ptr = unsafe { alloc(layout) as *mut C };
    // if ptr.is_null() {
    //     panic!("allocate");
    // }
    // (ptr, ptr as usize)
    allocate_array_page_size_aligned::<Container, Container>(1)
}

fn allocate_array_page_size_aligned<Container, Entry>(count: usize) -> (*mut Container, usize) {
    let layout = Layout::from_size_align(count * mem::size_of::<Entry>(), PAGE_SIZE)
        .expect("allocate_layout");
    let ptr = unsafe { alloc(layout) as *mut Container };
    if ptr.is_null() {
        panic!("allocate");
    }
    (ptr, ptr as usize)
}

impl<const QUEUE_DEPTH: u32> IoUring<QUEUE_DEPTH> {
    pub fn new() -> Self {
        let mut ring = unsafe {
            let mut s = mem::MaybeUninit::<io_uring>::uninit();
            // IORING_FEAT_ENTER_EXT_ARG so wait_cqes does not do submit() for us
            // IORING_FEAT_EXT_ARG
            let ret = io_uring_queue_init(QUEUE_DEPTH, s.as_mut_ptr(), 0);
            assert_eq!(
                ret,
                libc::EXIT_SUCCESS,
                "io_uring_queue_init: {:?}",
                IoError::from_raw_os_error(ret)
            );
            s.assume_init()
        };

        // let (buf_ring, io_uring_address) = unsafe {
        //     // Allocate a slice that starts on page aligned address
        //     // Unsure how to do this in normal rust
        //     let mut ring_head_ptr = mem::MaybeUninit::<*mut libc::c_void>::uninit();
        //     let ret = libc::posix_memalign(
        //         ring_head_ptr.as_mut_ptr(),
        //         PAGE_SIZE,
        //         // SAFETY save of the entries inside this wrapping array
        //         BUF_RING_COUNT * mem::size_of::<io_uring_buf>(),
        //     );
        //     assert_eq!(ret, libc::EXIT_SUCCESS, "posix_memalign {}", ret);
        //
        //     let void_ptr: *mut c_void = ring_head_ptr.assume_init();
        //     let ring_head: *mut io_uring_buf_ring = transmute(void_ptr);
        //     let address = ring_head as u64;
        //
        //     // let buf_ring: BufRing = transmute(ring_head_ptr.assume_init());
        //     // TODO: init before register?
        //     io_uring_buf_ring_init(ring_head);
        //     (ring_head, address)
        // };
        // println!("Asd");

        // : Box<BackingBufRing>
        // let (backing_buf_ring, backing_buf_ring_address) = unsafe {
        //     // Allocate a slice that starts on page aligned address
        //     // Unsure how to do this in normal rust
        //     let start_ptr_maybe: *mut *mut BackingBufRing = mem::zeroed();
        //     println!("Asd");
        //     // let ret = libc::posix_memalign(
        //     //     start_ptr_maybe as *mut *mut libc::c_void,
        //     //     PAGE_SIZE,
        //     //     // SAFETY save of the entries inside this wrapping array
        //     //     BUF_RING_COUNT * mem::size_of::<io_uring_buf>(),
        //     // );
        //     println!("Asd");
        //     assert_eq!(ret, libc::EXIT_SUCCESS, "posix_memalign {}", ret);
        //     // let test = Box::from_raw(start_ptr.assume_init());
        //     // let (_, test, _) = test.align_to::<BackingBufRing>();
        //     // Box::from(*test)
        //     // let start = start_ptr.assume_init();
        //     let start_ptr: *mut BackingBufRing = *start_ptr_maybe;
        //     let address = start_ptr as u64;
        //     (Box::from_raw(start_ptr), address)
        //     // Box::new(&*(start_ptr.assume_init() as *const BackingBufRing))
        // };
        // println!("Asd");

        // let (buf_ring_ptr, buf_ring_address) =
        //     allocate_array_page_size_aligned::<io_uring_buf_ring, io_uring_buf>(BUF_RING_COUNT);
        // let buf_ring = unsafe {
        //     io_uring_buf_ring_init(buf_ring_ptr);
        //     Box::from_raw(buf_ring_ptr)
        // };

        // let (backing_buf_ring_ptr, backing_buf_ring_address) =
        //     allocate_page_size_aligned::<BackingBufRing>();
        // let backing_buf_ring = unsafe { Box::from_raw(backing_buf_ring_ptr) };

        // // let bug_reg_addr = unsafe { addr_of!(*backing_buf_ring) as u64 };
        // let mut buf_reg: io_uring_buf_reg = unsafe { mem::zeroed() };
        // buf_reg.ring_entries = BUF_RING_COUNT as u32;
        // buf_reg.ring_addr = buf_ring_address as u64;
        // buf_reg.bgid = BUF_RING_ID;
        // unsafe {
        //     let ret = io_uring_register_buf_ring(&mut ring, &mut buf_reg, 0);
        //     assert_eq!(
        //         ret,
        //         0,
        //         "register {} {} {}",
        //         ret,
        //         IoError::from_raw_os_error(-ret),
        //         buf_ring_address
        //     );
        // }
        // println!("yay?");

        let mut result = IoUring {
            ring,
            // buf_ring,
            // backing_buf_ring,
        };
        // result.fill_buffer_ring();
        result
    }

    // fn get_buf_ring(&mut self) -> *mut io_uring_buf_ring {
    //     &mut *self.buf_ring
    // }

    // fn fill_buffer_ring(&mut self) {
    //     for i in 0..BUF_RING_COUNT {
    //         unsafe {
    //             io_uring_buf_ring_add(
    //                 self.get_buf_ring(),
    //                 self.backing_buf_ring[i as usize].as_mut_ptr() as *mut libc::c_void,
    //                 BUF_RING_COUNT as libc::c_uint,
    //                 // TODO: why short??
    //                 i as libc::c_ushort,
    //                 io_uring_buf_ring_mask(BUF_RING_COUNT as u32),
    //                 // TODO: why i32??
    //                 i as i32,
    //             );
    //         }
    //     }
    //     unsafe {
    //         io_uring_buf_ring_advance(self.get_buf_ring(), BUF_RING_COUNT as libc::c_int);
    //     }
    // }

    pub fn submit(&mut self) {
        let ret = unsafe { io_uring_submit(&mut self.ring) };
        if ret < 0 {
            panic!("io_uring_submit: {:?}", IoError::from_raw_os_error(ret));
        }
        println!("submit {}", ret);
    }

    pub fn wait_cqe(&mut self) {
        let mut cqe_ptr: *mut io_uring_cqe = unsafe { mem::zeroed() };
        let ret = unsafe { io_uring_wait_cqe(&mut self.ring, &mut cqe_ptr) };
        assert_eq!(
            ret,
            0,
            "io_uring_wait_cqe: {:?}",
            IoError::from_raw_os_error(ret)
        );
        let cqe_result = unsafe { (*cqe_ptr).res };
        assert!(cqe_result > 0, "(*cqe).res = {}", cqe_result);

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_data: IoUringEventData = unsafe { transmute((*cqe_ptr).user_data) };

        unsafe { io_uring_cqe_seen(&mut self.ring, cqe_ptr) };
    }

    pub fn wait_cqes_broken(&mut self) -> Vec<io_uring_cqe> {
        let mut cqes: Vec<io_uring_cqe> = Vec::with_capacity(QUEUE_DEPTH as usize);
        unsafe {
            cqes.set_len(QUEUE_DEPTH as usize);
        }

        let wait_time = __kernel_timespec {
            tv_sec: 5,
            tv_nsec: 5,
        };

        unsafe {
            let res = io_uring_wait_cqes(
                &mut self.ring,
                &mut cqes.as_mut_ptr(),
                QUEUE_DEPTH,
                &wait_time,
                std::ptr::null(),
            );
            if res != 0 {
                panic!("io_uring_wait_cqes: {:?}", IoError::from_raw_os_error(res));
            }
        }
        for cqe in cqes.iter() {
            if cqe.res != 0 {
                panic!(
                    "io_uring_wait_cqes inner: {:?}",
                    IoError::from_raw_os_error(cqe.res)
                );
            }
        }

        cqes
    }

    pub fn seen_cqe(&mut self, cqe: *mut io_uring_cqe) {
        unsafe { io_uring_cqe_seen(&mut self.ring, cqe) };
    }

    pub fn exit(&mut self) {
        unsafe { io_uring_queue_exit(&mut self.ring) };
    }
}

fn create_iovecs(count: usize, read_size: usize) -> Vec<iovec> {
    let mut iovecs: Vec<libc::iovec> = vec![unsafe { mem::zeroed() }; count];
    for iov in iovecs.iter_mut() {
        let buf = unsafe {
            let mut s = mem::MaybeUninit::<*mut libc::c_void>::uninit();
            if libc::posix_memalign(s.as_mut_ptr(), 4096, read_size) != 0 {
                panic!("can't allocate");
            }
            s.assume_init()
        };
        iov.iov_base = buf;
        iov.iov_len = read_size;
    }
    // unsafe {
    //     io_uring_register_buffers(&mut ring, iovecs.as_mut_ptr(), QUEUE_DEPTH);
    // }
    iovecs
}

struct IoUringCompletion {
    result: i32,
}

/// Struct Kernel passes from submission queue to completion queue
/// We have u64 *total* to work with
#[derive(Default)]
#[repr(C)]
struct IoUringEventData {
    buffer_index: u8,
    b: u16,
    c: u16,
    d: u16,
}

impl IoUringEventData {
    pub fn from_buf_index(buffer_index: u8) -> Self {
        let mut new = Self::default();
        new.buffer_index = buffer_index;
        new
    }
}

/// Handles read/write to a larger vec
struct IoUringFile {
    handle: File,
    backing_buf_ring: Box<BackingBufRing>,
    backing_buf_ring_available: [bool; BUF_RING_COUNT],
    next_index: usize,
    output_buffer: Vec<u8>,
}

impl IoUringFile {
    pub fn open(path: String) -> IoResult<Self> {
        let handle = File::open(path)?;
        let file_size = handle.metadata()?.len();
        let (ptr, _) = allocate_page_size_aligned::<BackingBufRing>();
        Ok(IoUringFile {
            handle,
            backing_buf_ring: unsafe { Box::from_raw(ptr) },
            backing_buf_ring_available: [true; BUF_RING_COUNT],
            output_buffer: vec![0u8; file_size as usize],
            next_index: 0,
        })
    }

    // pub fn create_read_iovec(
    //     &self,
    //     ring: &mut IoUring,
    // ) -> IoResult<(*mut io_uring_sqe, Vec<iovec>)> {
    //     let mut iovecs = create_iovecs(5, 128);
    //
    //     let sqe = unsafe { io_uring_get_sqe(&mut ring.ring) };
    //     if sqe == std::ptr::null_mut() {
    //         return Err(IoError::new(ErrorKind::Other, "fuck"));
    //     }
    //     unsafe { io_uring_prep_readv(sqe, self.handle.as_raw_fd(), iovecs.as_mut_ptr(), 5, 0) };
    //     Ok((sqe, iovecs))
    // }

    pub fn create_submission_queue_read_all(
        &mut self,
        ring: &mut IoUring,
    ) -> IoResult<Vec<*mut io_uring_sqe>> {
        let mut results = Vec::with_capacity(self.backing_buf_ring.len());
        for buf_index in 0..self.backing_buf_ring.len() {
            if self.backing_buf_ring_available[buf_index] {
                let sqe =
                    self.create_submission_queue_read_for_buffer_index(ring, buf_index as u8)?;
                results.push(sqe);
            }
        }
        Ok(results)
    }

    pub fn create_submission_queue_read_for_buffer_index(
        &mut self,
        ring: &mut IoUring,
        buf_index: u8,
    ) -> IoResult<*mut io_uring_sqe> {
        let sqe = unsafe { io_uring_get_sqe(&mut ring.ring) };
        if sqe == std::ptr::null_mut() {
            return Err(IoError::new(ErrorKind::Other, "get_sqe"));
        }

        let mut event = IoUringEventData::from_buf_index(buf_index);
        let event_ptr: *mut IoUringEventData = &mut event;
        unsafe {
            io_uring_sqe_set_data(sqe, event_ptr as *mut libc::c_void);
        }
        // TODO SAFETY Pretty sure we can't free this? Needs testing
        mem::forget(event);

        unsafe {
            io_uring_prep_read(
                sqe,
                self.handle.as_raw_fd(),
                self.backing_buf_ring[buf_index as usize].as_mut_ptr() as *mut libc::c_void,
                BUF_RING_ENTRY_SIZE as libc::c_uint,
                (BUF_RING_ENTRY_SIZE * buf_index as usize) as u64,
            )
        };
        Ok(sqe)
    }

    pub fn drain_completion_queue(&mut self, ring: &mut IoUring) {
        let mut cqe_ptr: *mut io_uring_cqe = unsafe { mem::zeroed() };
        let ret = unsafe { io_uring_wait_cqe(&mut ring.ring, &mut cqe_ptr) };
        assert_eq!(
            ret,
            0,
            "io_uring_wait_cqe: {:?}",
            IoError::from_raw_os_error(ret)
        );
        let cqe_result = unsafe { (*cqe_ptr).res };
        assert!(cqe_result > 0, "(*cqe).res = {}", cqe_result);

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_data: IoUringEventData = unsafe { transmute((*cqe_ptr).user_data) };
        cqe_data.buffer_index;

        unsafe { io_uring_cqe_seen(&mut ring.ring, cqe_ptr) };
    }
}

// #[repr(C, align(4096))]
// struct PageAlignedMemory<const ENTRY_SIZE: usize, const ENTRY_COUNT: usize> {
//     inner: [[u8; ENTRY_SIZE]; ENTRY_COUNT],
// }
//
// impl<const ENTRY_SIZE: usize, const ENTRY_COUNT: usize> PageAlignedMemory<ENTRY_SIZE, ENTRY_COUNT> {
//     unsafe fn as_fake_slice<T>(&mut self, size: usize) -> &mut [T] {
//         let (_, slice, _) = self.inner.align_to_mut::<T>();
//         &slice[0..size]
//     }
// }
