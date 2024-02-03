use itertools::Itertools;
use libc::iovec;
use num_format::ToFormattedString;
use std::io::Error as IoError;
use std::mem::transmute;
use std::path::Path;
use std::time::Instant;
use std::{io, mem};
use tracing::{error, trace};
use uring_sys2::{
    io_uring, io_uring_cqe, io_uring_cqe_seen, io_uring_peek_cqe, io_uring_queue_exit,
    io_uring_queue_init, io_uring_submit, io_uring_wait_cqe,
};

use crate::io_uring_common::IoUringEventData;
use crate::io_uring_file_copying::{IoUringFileCopying, BUF_RING_COUNT};
use crate::LOCALE;

/*
c file copy example: https://github.com/axboe/liburing/blob/master/examples/io_uring-cp.c
rust basic read example: https://github.com/Noah-Kennedy/liburing/blob/master/tests/test_read_file.rs
 */

pub(crate) fn io_uring_main(input_path: &Path) -> io::Result<()> {
    // init
    let mut ring = IoUring::new();

    let start = Instant::now();

    // fill queue
    let mut io_file = IoUringFileCopying::open(input_path, &mut ring)?;
    if let Err(e) = io_file.read_entire_file(&mut ring) {
        println!("err");
        error!("IOU failed! {}\n{}", e, e.my_backtrace());
        return Ok(());
    }
    let xy_array_usize = io_file.into_result_as_usize(&mut ring);
    let sum: usize = xy_array_usize.iter().sum1().unwrap();

    let read_watch = Instant::now() - start;
    println!(
        "processed in {} ms checksum {}",
        read_watch.as_millis().to_formatted_string(&LOCALE),
        sum.to_formatted_string(&LOCALE),
    );

    ring.exit();

    Ok(())
}

pub struct IoUring {
    pub ring: io_uring,
    // buf_ring: Box<io_uring_buf_ring>,
    // backing_buf_ring: Box<BackingBufRing>,
}

impl IoUring {
    pub fn new() -> Self {
        let ring = unsafe {
            let mut s = mem::MaybeUninit::<io_uring>::zeroed();
            // IORING_FEAT_ENTER_EXT_ARG so wait_cqes does not do submit() for us
            // IORING_FEAT_EXT_ARG
            let ret = io_uring_queue_init(BUF_RING_COUNT as u32, s.as_mut_ptr(), 0);
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

        IoUring {
            ring,
            // buf_ring,
            // backing_buf_ring,
        }
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

    pub fn submit(&mut self) -> bool {
        let submitted_entries = unsafe { io_uring_submit(&mut self.ring) };
        if submitted_entries < 0 {
            panic!(
                "io_uring_submit: {:?}",
                IoError::from_raw_os_error(submitted_entries)
            );
        }
        trace!("submit");
        submitted_entries != 0
    }

    pub fn assert_cq_has_no_overflow(&self) {
        let overflow = unsafe { *self.ring.cq.koverflow };
        assert_eq!(
            overflow, 0,
            "detected overflow of completion queue, too many submissions in flight"
        );
    }

    pub fn peek_cqe(&mut self) -> Option<*mut io_uring_cqe> {
        let mut cqe_ptr: *mut io_uring_cqe = unsafe { mem::zeroed() };
        let ret = unsafe { io_uring_peek_cqe(&mut self.ring, &mut cqe_ptr) };
        if ret == -libc::EAGAIN {
            None
        } else {
            Some(cqe_ptr)
        }
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

    #[cfg(nope)]
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

    // pub fn seen_cqe(&mut self, cqe: *mut io_uring_cqe) {
    //     unsafe { io_uring_cqe_seen(&mut self.ring, cqe) };
    // }

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
