use num_format::ToFormattedString;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error as IoError, ErrorKind};
use std::os::fd::AsRawFd;
use std::{mem, ptr};
use tracing::{debug, trace};

use uring_sys2::{
    io_uring, io_uring_cqe, io_uring_cqe_get_data, io_uring_cqe_get_data64, io_uring_cqe_seen,
    io_uring_get_sqe, io_uring_peek_cqe, io_uring_prep_read, io_uring_sqe, io_uring_sqe_set_data,
    io_uring_sqe_set_data64, io_uring_wait_cqe,
};

use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, IoUringEventData, PAGE_SIZE};
use crate::LOCALE;

pub const BUF_RING_COUNT: usize = 32 * 4;
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
type BackingBufEntry = [u8; BUF_RING_ENTRY_SIZE];
type BackingBufRing = [BackingBufEntry; BUF_RING_COUNT];

/// Handles read/write to a larger vec
#[repr(C, align(4096))]
pub struct IoUringFile {
    file_handle: File,
    file_size: usize,
    backing_buf_ring: Box<BackingBufRing>,
    backing_buf_ring_data: [BackingBufData; BUF_RING_COUNT],
    result_buffer: Vec<u8>,
    result_buffer_done: Vec<bool>,
    result_buffer_done_fast_check_from: usize,
    result_cursor: usize,
}

impl IoUringFile {
    pub fn open(path: String) -> IoResult<Self> {
        let file_handle = File::open(path)?;
        let file_size = file_handle.metadata()?.len() as usize;
        println!("file size {}", file_size);
        let (ptr, _) = allocate_page_size_aligned::<BackingBufRing>();
        Ok(IoUringFile {
            file_handle,
            file_size,
            backing_buf_ring: unsafe { Box::from_raw(ptr) },
            backing_buf_ring_data: [BackingBufData::default(); BUF_RING_COUNT],
            result_buffer: vec![0u8; file_size],
            result_buffer_done: vec![false; file_size],
            result_buffer_done_fast_check_from: 0,
            result_cursor: 0,
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
        let mut blocked = false;
        let mut sqe_enabled = true;
        let mut total_cqe: usize = 0;
        let mut total_sqe: usize = 0;
        while self.result_buffer_done_fast_check_from < self.result_buffer_done.len() {
            let submit;
            let mut wait_for_cqe: bool;
            let processed_submission: u8;
            if sqe_enabled {
                match self.refresh_submission_queue_read_all(ring)? {
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
                        submit = false;
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

            if submit && !blocked {
                ring.submit();
            }

            let mut processed_completed = 0;
            unsafe {
                while let PopCqeResult::PopOneEvent =
                    self.pop_completion_queue(ring, PopCqeType::Wait)
                {
                    processed_completed += 1;
                    wait_for_cqe = false;
                    blocked = false;
                }
            }
            total_sqe += processed_completed;
            debug!(
                "processed_submission {} processed_completed {} minimum_done {}",
                processed_submission, processed_completed, self.result_buffer_done_fast_check_from
            );
            ring.assert_cq_has_no_overflow();

            if sqe_enabled && processed_completed == 0 && processed_submission == 0 {
                trace!("blocked!");
                blocked = true;
            }
            if !sqe_enabled && total_cqe == total_sqe {
                debug!("entire EOF");
                break;
            }
            self.check_and_advance();
        }

        Ok(())
    }

    pub fn refresh_submission_queue_read_all(
        &mut self,
        ring: &mut IoUring,
    ) -> IoResult<CreateAllSqeResult> {
        let mut changed = 0;
        for buf_index in 0..self.backing_buf_ring.len() {
            if self.backing_buf_ring_data[buf_index].is_free {
                let sqe = unsafe {
                    self.refresh_submission_queue_read_for_buffer_index(ring, buf_index as u8)?
                };
                match sqe {
                    CreateSqeResult::Eof => {
                        debug!("SQ EOF");
                        return Ok(CreateAllSqeResult::EofWithTotal(changed));
                    }
                    CreateSqeResult::CreatedAt(_) => changed += 1,
                }
            }
        }
        Ok(CreateAllSqeResult::RefreshTotal(changed))
    }

    pub unsafe fn refresh_submission_queue_read_for_buffer_index(
        &mut self,
        ring: &mut IoUring,
        buf_index: u8,
    ) -> IoResult<CreateSqeResult> {
        let sqe = io_uring_get_sqe(&mut ring.ring);
        assert!(!sqe.is_null(), "get_sqe, queue is full?");

        io_uring_sqe_set_data64(sqe, buf_index as u64);

        let offset = self.result_cursor * BUF_RING_ENTRY_SIZE;
        if offset > self.file_size {
            return Ok(CreateSqeResult::Eof);
        }

        let buf_index_usize = buf_index as usize;
        io_uring_prep_read(
            sqe,
            self.file_handle.as_raw_fd(),
            ptr::addr_of_mut!(self.backing_buf_ring_data[buf_index_usize].io_event_data)
                as *mut libc::c_void,
            BUF_RING_ENTRY_SIZE as libc::c_uint,
            offset as u64,
        );
        self.backing_buf_ring_data[buf_index_usize].is_free = false;
        self.backing_buf_ring_data[buf_index_usize].result_offset = offset;
        self.backing_buf_ring_data[buf_index_usize].result_abs_buf_index = self.result_cursor;
        trace!(
            "sqe create {}\tminimum_done {}",
            buf_index,
            self.result_cursor
        );

        self.result_cursor += 1;
        Ok(CreateSqeResult::CreatedAt(sqe))
    }

    pub unsafe fn pop_completion_queue(
        &mut self,
        ring: &mut IoUring,
        pop_type: PopCqeType,
    ) -> PopCqeResult {
        let mut cqe_ptr: *mut io_uring_cqe = mem::zeroed();
        debug!("wait_cqe");
        let ret = io_uring_wait_cqe(&mut ring.ring, &mut cqe_ptr);
        assert_eq!(
            ret,
            0,
            "io_uring_wait_cqe: {:?}",
            IoError::from_raw_os_error(ret)
        );
        assert!(!cqe_ptr.is_null(), "cqe nullptr");
        println!("fetched");

        let cqe_result = (*cqe_ptr).res;
        assert!(cqe_result >= 0, "(*cqe).res = {}", cqe_result);
        println!("res");

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_buf_index = io_uring_cqe_get_data64(cqe_ptr) as usize;
        println!("source");
        let source: BackingBufEntry = self.backing_buf_ring[cqe_buf_index];
        println!("source");
        let mut source_ref: &[u8] = &source;
        println!("source");

        let target_offset = self.backing_buf_ring_data[cqe_buf_index].result_offset;
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

        trace!(
            "cqe buf {} range {} to {}",
            cqe_buf_index,
            target_offset.to_formatted_string(&LOCALE),
            target_offset_end.to_formatted_string(&LOCALE)
        );

        let target = &mut self.result_buffer[target_offset..target_offset_end];
        target.copy_from_slice(source_ref);
        self.backing_buf_ring_data[cqe_buf_index].is_free = true;
        // self.offset_max_read = self.offset_max_read.max(target_offset_end);

        io_uring_cqe_seen(&mut ring.ring, cqe_ptr);
        PopCqeResult::PopOneEvent
    }

    fn check_and_advance(&mut self) {
        let start = self.result_buffer_done_fast_check_from;
        let mut cursor = start;
        for i in start..(self.result_buffer_done.len()) {
            if !self.result_buffer_done[i] {
                break;
            }
            cursor += 1;
        }
        self.result_buffer_done_fast_check_from = cursor;
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

pub enum PopCqeType {
    Wait,
    SubmitAndWait,
}

pub enum PopCqeResult {
    PopOneEvent,
    EmptyQueue,
}

#[derive(Clone, Copy)]
struct BackingBufData {
    is_free: bool,
    result_offset: usize,
    result_abs_buf_index: usize,
    io_event_data: IoUringEventData,
}

impl Default for BackingBufData {
    fn default() -> Self {
        BackingBufData {
            is_free: true,
            result_offset: usize::MAX,
            result_abs_buf_index: usize::MAX,
            io_event_data: IoUringEventData::from_buf_index(0),
        }
    }
}
