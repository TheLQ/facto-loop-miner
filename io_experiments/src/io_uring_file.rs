use num_format::ToFormattedString;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::{Error as IoError, ErrorKind};
use std::os::fd::AsRawFd;
use std::{mem, ptr};

use uring_sys2::{
    io_uring, io_uring_cqe, io_uring_cqe_get_data, io_uring_cqe_seen, io_uring_get_sqe,
    io_uring_peek_cqe, io_uring_prep_read, io_uring_sqe, io_uring_sqe_set_data, io_uring_wait_cqe,
};

use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, log_debug, IoUringEventData, PAGE_SIZE};
use crate::LOCALE;

pub const BUF_RING_COUNT: usize = 32 * 4;
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
type BackingBufEntry = [u8; BUF_RING_ENTRY_SIZE];
type BackingBufRing = [BackingBufEntry; BUF_RING_COUNT];

/// Handles read/write to a larger vec
pub struct IoUringFile {
    handle: File,
    backing_buf_ring: Box<BackingBufRing>,
    backing_buf_ring_data: [BackingBufData; BUF_RING_COUNT],
    offset_max_read: usize,
    next_result_index: usize,
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
            backing_buf_ring_data: [BackingBufData::default(); BUF_RING_COUNT],
            result_buffer: vec![0u8; file_size],
            offset_max_read: 0,
            next_result_index: 0,
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
        let mut blocked = false;
        let mut sqe_enabled = true;
        let mut total_cqe: usize = 0;
        let mut total_sqe: usize = 0;
        while self.offset_max_read < file_size {
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
                    self.pop_completion_queue(ring, wait_for_cqe || blocked)
                {
                    processed_completed += 1;
                    wait_for_cqe = false;
                    blocked = false;
                }
            }
            total_sqe += processed_completed;
            log_debug(&format!(
                "processed_submission {} processed_completed {} result_index {}",
                processed_submission, processed_completed, self.next_result_index
            ));
            ring.assert_cq_has_no_overflow();

            if sqe_enabled && processed_completed == 0 && processed_submission == 0 {
                // log_debug("blocked!");
                blocked = true;
            }
            if !sqe_enabled && total_cqe == total_sqe {
                log_debug("entire EOF");
                break;
            }
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
                        log_debug("SQ EOF");
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

        let event = Box::new(IoUringEventData::from_buf_index(buf_index));
        // SAFETY Box::new.into_raw() effectively leaks this?
        io_uring_sqe_set_data(sqe, Box::into_raw(event) as *mut libc::c_void);

        let offset = self.next_result_index * BUF_RING_ENTRY_SIZE;

        if offset > self.file_size {
            return Ok(CreateSqeResult::Eof);
        }
        let buf_index_usize = buf_index as usize;
        io_uring_prep_read(
            sqe,
            self.handle.as_raw_fd(),
            ptr::addr_of_mut!(self.backing_buf_ring_data[buf_index_usize].io_event_data)
                as *mut libc::c_void,
            BUF_RING_ENTRY_SIZE as libc::c_uint,
            offset as u64,
        );
        self.backing_buf_ring_data[buf_index_usize].is_free = false;
        self.backing_buf_ring_data[buf_index_usize].result_offset = offset;
        self.backing_buf_ring_data[buf_index_usize].result_abs_buf_index = self.next_result_index;
        // log_debug(&format!(
        //     "sqe create {}\tnext_result_index {}",
        //     buf_index, self.result_index
        // ));

        self.next_result_index += 1;
        Ok(CreateSqeResult::CreatedAt(sqe))
    }

    pub unsafe fn pop_completion_queue(&mut self, ring: &mut IoUring, wait: bool) -> PopCqeResult {
        let mut cqe_ptr: *mut io_uring_cqe = mem::zeroed();
        let ret = if !wait {
            let ret = io_uring_peek_cqe(&mut ring.ring, &mut cqe_ptr);
            if ret == -libc::EAGAIN {
                return PopCqeResult::EmptyQueue;
            } else if cqe_ptr.is_null() {
                panic!("!!! cqe nullptr")
                // return PopCqeResult::EmptyQueue;
            } else {
                ret
            }
        } else {
            log_debug("wait_cqe");
            io_uring_wait_cqe(&mut ring.ring, &mut cqe_ptr)
        };
        assert_eq!(
            ret,
            0,
            "io_uring_{}_cqe: {:?}",
            if wait { "wait" } else { "peek" },
            IoError::from_raw_os_error(ret)
        );
        // assert!(cqe_ptr.is_null(), "cqe nullptr");

        let cqe_result = (*cqe_ptr).res;
        assert!(cqe_result >= 0, "(*cqe).res = {}", cqe_result);

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_data = io_uring_cqe_get_data(cqe_ptr) as *mut IoUringEventData;
        let cqe_data = &*cqe_data;
        let cqe_buf_index_usize = cqe_data.buf_index as usize;
        let source: BackingBufEntry = self.backing_buf_ring[cqe_buf_index_usize];
        let mut source_ref: &[u8] = &source;

        let target_offset = self.backing_buf_ring_data[cqe_buf_index_usize].result_offset;
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

        // log_debug(&format!(
        //     "cqe buf {} range {} to {}",
        //     cqe_data.buf_index,
        //     target_offset.to_formatted_string(&LOCALE),
        //     target_offset_end.to_formatted_string(&LOCALE)
        // ));

        let target = &mut self.result_buffer[target_offset..target_offset_end];
        target.copy_from_slice(source_ref);
        self.backing_buf_ring_data[cqe_buf_index_usize].is_free = true;
        self.offset_max_read = self.offset_max_read.max(target_offset_end);

        io_uring_cqe_seen(&mut ring.ring, cqe_ptr);
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
