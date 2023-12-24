use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, log_debug, IoUringEventData, PAGE_SIZE};
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error as IoError, ErrorKind};
use std::mem;
use std::os::fd::AsRawFd;
use uring_sys2::{
    io_uring, io_uring_cqe, io_uring_cqe_get_data, io_uring_cqe_seen, io_uring_get_sqe,
    io_uring_peek_cqe, io_uring_prep_read, io_uring_sqe, io_uring_sqe_set_data, io_uring_wait_cqe,
};

const BUF_RING_COUNT: usize = 50;
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
type BufRing = [io_uring; BUF_RING_COUNT];
type BackingBufEntry = [u8; BUF_RING_ENTRY_SIZE];
type BackingBufRing = [BackingBufEntry; BUF_RING_COUNT];

/// Handles read/write to a larger vec
pub struct IoUringFile {
    handle: File,
    backing_buf_ring: Box<BackingBufRing>,
    backing_buf_ring_free: [bool; BUF_RING_COUNT],
    backing_buf_ring_offset: [usize; BUF_RING_COUNT],
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
            backing_buf_ring_free: [true; BUF_RING_COUNT],
            backing_buf_ring_offset: [usize::MAX; BUF_RING_COUNT],
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

    pub fn read_entire_file(&mut self, ring: &mut IoUring) -> IoResult<()> {
        let file_size = self.output_buffer.len();
        let last_index = (file_size - (file_size % BUF_RING_ENTRY_SIZE)) / BUF_RING_ENTRY_SIZE;
        let mut force_next = false;
        while self.next_index < last_index {
            let changed = self.create_submission_queue_read_all(ring)?;
            let is_changed = changed != 0;
            if is_changed {
                ring.submit();
            }

            let mut drained = 0;
            loop {
                let is_drained = self.drain_completion_queue(ring, force_next);
                force_next = false;
                if !is_drained {
                    drained += 1;
                    break;
                }
                // println!("loop");
            }
            let is_drained = drained != 0;
            if !is_drained && !is_changed {
                log_debug("force next");
                force_next = true;
            }
        }

        Ok(())
    }

    pub fn create_submission_queue_read_all(&mut self, ring: &mut IoUring) -> IoResult<u8> {
        // let mut results = Vec::with_capacity(self.backing_buf_ring.len());
        let mut changed = 0;
        for buf_index in 0..self.backing_buf_ring.len() {
            if self.backing_buf_ring_free[buf_index] {
                let sqe =
                    self.create_submission_queue_read_for_buffer_index(ring, buf_index as u8)?;
                // results.push(sqe);
                changed += 1;
            }
        }
        // Ok(results)
        Ok(changed)
    }

    pub fn create_submission_queue_read_for_buffer_index(
        &mut self,
        ring: &mut IoUring,
        buf_index: u8,
    ) -> IoResult<*mut io_uring_sqe> {
        let sqe = unsafe { io_uring_get_sqe(&mut ring.ring) };
        if sqe.is_null() {
            return Err(IoError::new(ErrorKind::Other, "get_sqe, queue if full?"));
        }

        let event = Box::new(IoUringEventData::from_buf_index(buf_index));
        unsafe {
            io_uring_sqe_set_data(sqe, Box::into_raw(event) as *mut libc::c_void);
        }
        // TODO SAFETY Pretty sure we can't free this? Needs testing
        // mem::forget(event);

        let offset = (BUF_RING_ENTRY_SIZE * self.next_index) as u64;
        unsafe {
            io_uring_prep_read(
                sqe,
                self.handle.as_raw_fd(),
                self.backing_buf_ring[buf_index as usize].as_mut_ptr() as *mut libc::c_void,
                BUF_RING_ENTRY_SIZE as libc::c_uint,
                offset,
            )
        };
        self.backing_buf_ring_free[buf_index as usize] = false;
        self.backing_buf_ring_offset[buf_index as usize] = offset as usize;
        log_debug(&format!("sqe create {}", buf_index));
        Ok(sqe)
    }

    pub fn drain_completion_queue(&mut self, ring: &mut IoUring, wait: bool) -> bool {
        let mut cqe_ptr: *mut io_uring_cqe = unsafe { mem::zeroed() };
        if !wait {
            let ret = unsafe { io_uring_peek_cqe(&mut ring.ring, &mut cqe_ptr) };
            if ret == -libc::EAGAIN {
                return false;
            }
            assert_eq!(
                ret,
                0,
                "io_uring_peek_cqe: {:?}",
                IoError::from_raw_os_error(ret)
            );
        } else {
            log_debug("wait");
            let ret = unsafe { io_uring_wait_cqe(&mut ring.ring, &mut cqe_ptr) };
            assert_eq!(
                ret,
                0,
                "io_uring_wait_cqe: {:?}",
                IoError::from_raw_os_error(ret)
            );
        };
        if cqe_ptr.is_null() {
            panic!("asd")
        }

        let cqe_result = unsafe { (*cqe_ptr).res };
        assert!(cqe_result > 0, "(*cqe).res = {}", cqe_result);

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_data: *mut IoUringEventData =
            unsafe { io_uring_cqe_get_data(cqe_ptr) as *mut IoUringEventData };
        let cqe_data = unsafe { &*cqe_data };
        log_debug(&format!("read {}", cqe_data.buffer_index));
        let source = self.backing_buf_ring[cqe_data.buffer_index as usize];
        let target_offset = self.backing_buf_ring_offset[cqe_data.buffer_index as usize];
        let target = &mut self.output_buffer[target_offset..(target_offset + BUF_RING_ENTRY_SIZE)];
        target.copy_from_slice(&source);
        self.backing_buf_ring_free[cqe_data.buffer_index as usize] = true;

        unsafe { io_uring_cqe_seen(&mut ring.ring, cqe_ptr) };
        mem::forget(cqe_data);

        true
    }
}
