use num_format::ToFormattedString;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error as IoError, ErrorKind};
use std::mem;
use std::os::fd::AsRawFd;

use uring_sys2::{
    io_uring, io_uring_cqe, io_uring_cqe_get_data, io_uring_cqe_seen, io_uring_get_sqe,
    io_uring_peek_cqe, io_uring_prep_read, io_uring_sqe, io_uring_sqe_set_data, io_uring_wait_cqe,
};

use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, log_debug, IoUringEventData, PAGE_SIZE};
use crate::LOCALE;

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
    offset_max_read: usize,
    result_index: usize,
    result_buffer: Vec<u8>,
    file_size: usize,
}

impl IoUringFile {
    pub fn open(path: String) -> IoResult<Self> {
        let handle = File::open(path)?;
        let file_size = handle.metadata()?.len() as usize;
        let (ptr, _) = allocate_page_size_aligned::<BackingBufRing>();
        Ok(IoUringFile {
            handle,
            backing_buf_ring: unsafe { Box::from_raw(ptr) },
            backing_buf_ring_free: [true; BUF_RING_COUNT],
            backing_buf_ring_offset: [usize::MAX; BUF_RING_COUNT],
            result_buffer: vec![0u8; file_size],
            offset_max_read: 0,
            result_index: 0,
            file_size,
        })
    }

    pub fn into_result(self) -> Vec<u8> {
        self.result_buffer
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

    /// Superfast io_uring
    pub fn read_entire_file(&mut self, ring: &mut IoUring) -> IoResult<()> {
        let file_size = self.result_buffer.len();
        let mut sqe_enabled = true;
        let mut total_cqe: usize = 0;
        let mut total_sqe: usize = 0;
        while self.offset_max_read < file_size {
            let submit;
            let mut wait_for_cqe: bool;
            let processed_submission: u8;
            if sqe_enabled {
                match self.create_submission_queue_read_all(ring)? {
                    CreateAllSqeResult::RefreshZero => {
                        submit = false;
                        wait_for_cqe = true;
                        processed_submission = 0;
                    }
                    CreateAllSqeResult::RefreshTotal(total) => {
                        submit = true;
                        wait_for_cqe = false;
                        processed_submission = total;
                    }
                    CreateAllSqeResult::EofWithTotal(total) => {
                        submit = true;
                        wait_for_cqe = false;
                        processed_submission = total;
                        sqe_enabled = false;
                    }
                }
            } else {
                submit = false;
                wait_for_cqe = true;
                processed_submission = 0;
            }
            total_cqe += processed_submission as usize;
            log_debug(&format!("processed_submission {}", processed_submission));

            if submit {
                ring.submit();
            }

            let mut processed_completed = 0;
            while let PopCqeResult::PopOneEvent = self.pop_completion_queue(ring, wait_for_cqe) {
                processed_completed += 1;
                wait_for_cqe = false;
            }
            total_sqe += processed_completed;
            log_debug(&format!("processed_completed {}", processed_completed));

            if !sqe_enabled && total_cqe == total_sqe {
                log_debug("entire EOF");
                break;
            }
        }

        Ok(())
    }

    pub fn create_submission_queue_read_all(
        &mut self,
        ring: &mut IoUring,
    ) -> IoResult<CreateAllSqeResult> {
        // let mut results = Vec::with_capacity(self.backing_buf_ring.len());
        let mut changed = 0;
        for buf_index in 0..self.backing_buf_ring.len() {
            if self.backing_buf_ring_free[buf_index] {
                let sqe =
                    self.create_submission_queue_read_for_buffer_index(ring, buf_index as u8)?;
                // results.push(sqe);
                match sqe {
                    CreateSqeResult::Eof => {
                        log_debug("sqe EOF");
                        return Ok(CreateAllSqeResult::EofWithTotal(changed));
                    }
                    CreateSqeResult::CreatedAt(_) => changed += 1,
                }
            }
        }
        // Ok(results)
        return Ok(CreateAllSqeResult::RefreshTotal(changed));
    }

    pub fn create_submission_queue_read_for_buffer_index(
        &mut self,
        ring: &mut IoUring,
        buf_index: u8,
    ) -> IoResult<CreateSqeResult> {
        let sqe = unsafe { io_uring_get_sqe(&mut ring.ring) };
        if sqe.is_null() {
            return Err(IoError::new(ErrorKind::Other, "get_sqe, queue is full?"));
        }

        let event = Box::new(IoUringEventData::from_buf_index(buf_index));
        unsafe {
            io_uring_sqe_set_data(sqe, Box::into_raw(event) as *mut libc::c_void);
        }
        // TODO SAFETY Pretty sure we can't free this? Needs testing
        // mem::forget(event);

        let offset = self.result_index * BUF_RING_ENTRY_SIZE;
        self.result_index += 1;
        log_debug(&format!("result_index {}", self.result_index));
        if offset > self.file_size {
            return Ok(CreateSqeResult::Eof);
        }
        unsafe {
            io_uring_prep_read(
                sqe,
                self.handle.as_raw_fd(),
                self.backing_buf_ring[buf_index as usize].as_mut_ptr() as *mut libc::c_void,
                BUF_RING_ENTRY_SIZE as libc::c_uint,
                offset as u64,
            )
        };
        self.backing_buf_ring_free[buf_index as usize] = false;
        self.backing_buf_ring_offset[buf_index as usize] = offset;
        log_debug(&format!("sqe create {}", buf_index));
        Ok(CreateSqeResult::CreatedAt(sqe))
    }

    pub fn pop_completion_queue(&mut self, ring: &mut IoUring, wait: bool) -> PopCqeResult {
        let mut cqe_ptr: *mut io_uring_cqe = unsafe { mem::zeroed() };
        if !wait {
            let ret = unsafe { io_uring_peek_cqe(&mut ring.ring, &mut cqe_ptr) };
            if ret == -libc::EAGAIN {
                return PopCqeResult::EmptyQueue;
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
        assert!(cqe_result >= 0, "(*cqe).res = {}", cqe_result);

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_data: *mut IoUringEventData =
            unsafe { io_uring_cqe_get_data(cqe_ptr) as *mut IoUringEventData };
        let cqe_data = unsafe { &*cqe_data };
        log_debug(&format!("cqe read {}", cqe_data.buf_index));
        let source: BackingBufEntry = self.backing_buf_ring[cqe_data.buf_index as usize];
        let mut source_ref: &[u8] = &source;

        let target_offset = self.backing_buf_ring_offset[cqe_data.buf_index as usize];
        assert!(
            target_offset < self.file_size,
            "target offset too big {}",
            target_offset.to_formatted_string(&LOCALE)
        );
        let mut target_offset_end = target_offset + BUF_RING_ENTRY_SIZE;
        if target_offset_end > self.file_size {
            target_offset_end = self.file_size;
            source_ref = &source[0..(target_offset_end - target_offset)];
        }

        log_debug(&format!(
            "cqe from {} to {}",
            target_offset.to_formatted_string(&LOCALE),
            target_offset_end.to_formatted_string(&LOCALE)
        ));

        let target = &mut self.result_buffer[target_offset..target_offset_end];
        target.copy_from_slice(source_ref);
        self.backing_buf_ring_free[cqe_data.buf_index as usize] = true;
        self.offset_max_read = self.offset_max_read.max(target_offset_end);
        log_debug(&format!("cqe buf_index {}", cqe_data.buf_index));

        unsafe { io_uring_cqe_seen(&mut ring.ring, cqe_ptr) };
        PopCqeResult::PopOneEvent
    }
}

pub enum CreateSqeResult {
    CreatedAt(*mut io_uring_sqe),
    Eof,
}

pub enum CreateAllSqeResult {
    RefreshZero,
    RefreshTotal(u8),
    EofWithTotal(u8),
}

pub enum PopCqeResult {
    PopOneEvent,
    EmptyQueue,
}
