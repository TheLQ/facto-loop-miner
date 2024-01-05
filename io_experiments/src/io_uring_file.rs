use std::backtrace::Backtrace;
use std::fs::File;
use std::io::Result as IoResult;
use std::mem::ManuallyDrop;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::{io, ptr};

use num_format::ToFormattedString;
use tracing::{debug, info, trace};
use uring_sys2::{
    io_uring_cqe, io_uring_cqe_get_data64, io_uring_cqe_seen, io_uring_get_sqe, io_uring_prep_read,
    io_uring_sqe, io_uring_sqe_set_data64, io_uring_wait_cqe,
};

use crate::err::{VIoError, VIoResult};
use crate::io::USIZE_BYTES;
use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, PAGE_SIZE};
use crate::LOCALE;

pub const BUF_RING_COUNT: usize = 32 * 8;
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
type BackingBufEntry = [u8; BUF_RING_ENTRY_SIZE];
type BackingBufRing = [BackingBufEntry; BUF_RING_COUNT];

// #[repr(align(4096))]
// struct BackingBufRingStruct(BackingBufRingStructInner);
//
// #[repr(transparent)]
// struct BackingBufRingStructInner {
//     inner: BackingBufRing
// }

/// Handles read/write to a larger vec
// #[repr(C, align(4096))]
pub struct IoUringFile {
    file_handle: File,
    file_size: usize,
    backing_buf_ring: ManuallyDrop<Box<BackingBufRing>>,
    backing_buf_ring_data: [BackingBufData; BUF_RING_COUNT],
    result_buffer: ManuallyDrop<Vec<u8>>,
    result_buffer_done: Vec<bool>,
    result_buffer_done_fast_check_from: usize,
    result_cursor: usize,
}

impl IoUringFile {
    pub fn open(path: &Path, ring: &mut IoUring) -> IoResult<Self> {
        let file_handle = File::open(path)?;
        let file_size = file_handle.metadata()?.len() as usize;
        println!("file size {}", file_size);

        // let register_result = unsafe {
        //     io_uring_register_files(&mut ring.ring, [file_handle.as_raw_fd()].as_ptr(), 1)
        // };
        // assert_eq!(register_result, 0, "register failed");

        let (backing_buf_ptr, _) = allocate_page_size_aligned::<BackingBufRing>();
        Ok(IoUringFile {
            file_handle,
            file_size,
            backing_buf_ring: unsafe { ManuallyDrop::new(Box::from_raw(backing_buf_ptr)) },
            backing_buf_ring_data: [BackingBufData::default(); BUF_RING_COUNT],
            result_buffer: ManuallyDrop::new(vec![0u8; file_size]),
            result_buffer_done: vec![false; file_size],
            result_buffer_done_fast_check_from: 0,
            result_cursor: 0,
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

    /// Superfast io_uring
    pub fn read_entire_file(&mut self, ring: &mut IoUring) -> VIoResult<()> {
        let mut total_submissions: usize = 0;
        let mut total_completions: usize = 0;

        // pre-fill Completion Queue
        for buf_index in 0..self.backing_buf_ring_data.len() {
            if self.backing_buf_ring_data[buf_index].is_free {
                let sqe = unsafe {
                    self.refresh_submission_queue_read_for_buffer_index(ring, buf_index)?
                };
                if let CreateSqeResult::CreatedAt(_) = sqe {
                    total_submissions += 1;
                } else {
                    panic!("file too small");
                }
            }
        }
        ring.submit();

        let mut end_of_file = false;
        loop {
            let free_buffer_index = unsafe { self.pop_completion_queue(ring)? };
            total_completions += 1;

            if !end_of_file {
                let create_cqe = unsafe {
                    self.refresh_submission_queue_read_for_buffer_index(ring, free_buffer_index)?
                };
                match create_cqe {
                    CreateSqeResult::CreatedAt(_) => {
                        total_submissions += 1;
                    }
                    CreateSqeResult::Eof => {
                        end_of_file = true;
                    }
                }
                ring.submit();
            }

            debug!(
                "total_submissions {} total_completions {} minimum_done {}",
                total_submissions, total_completions, self.result_buffer_done_fast_check_from
            );
            ring.assert_cq_has_no_overflow();

            if end_of_file && total_submissions == total_completions {
                debug!("entire EOF");
                break;
            }
        }

        Ok(())
    }

    pub fn into_result_as_usize(mut self, ring: &mut IoUring) -> Vec<usize> {
        let result_len_u8 = self.result_buffer.len();
        let result_len_usize = result_len_u8 / USIZE_BYTES;

        // unsafe {
        //     io_uring_unregister_files(&mut ring.ring);
        // }

        // Build u8 Vec viewing the same memory with proper aligned access
        // Docs state the outer slices should be empty in real world environments
        let (xy_vec_prefix, xy_vec_aligned, xy_vec_suffix) =
            unsafe { self.result_buffer.align_to_mut::<usize>() };
        assert_eq!(xy_vec_prefix.len(), 0, "prefix big");
        assert_eq!(xy_vec_suffix.len(), 0, "suffix big");
        unsafe {
            // SAFETY result_buffer is ManuallyDrop so we can own it's data now
            Vec::from_raw_parts(
                xy_vec_aligned.as_mut_ptr(),
                result_len_usize,
                result_len_usize,
            )
        }
    }

    pub unsafe fn refresh_submission_queue_read_for_buffer_index(
        &mut self,
        ring: &mut IoUring,
        buf_index: usize,
    ) -> VIoResult<CreateSqeResult> {
        let offset = self.result_cursor * BUF_RING_ENTRY_SIZE;
        if offset > self.file_size {
            info!("EOF at result_cursor {}", self.result_cursor);
            return Ok(CreateSqeResult::Eof);
        }
        let sqe_ptr = io_uring_get_sqe(&mut ring.ring);
        if sqe_ptr.is_null() {
            return Err(VIoError::IoUring_SqeNullPointer {
                backtrace: Backtrace::capture(),
            });
        }

        io_uring_sqe_set_data64(sqe_ptr, buf_index as u64);
        // io_uring_sqe_set_flags(sqe_ptr, IOSQE_FIXED_FILE);
        io_uring_prep_read(
            sqe_ptr,
            // should only register this file
            // 0,
            self.file_handle.as_raw_fd(),
            ptr::addr_of_mut!(self.backing_buf_ring[buf_index]) as *mut libc::c_void,
            BUF_RING_ENTRY_SIZE as libc::c_uint,
            offset as u64,
        );
        self.backing_buf_ring_data[buf_index].is_free = false;
        self.backing_buf_ring_data[buf_index].result_offset = offset;
        self.backing_buf_ring_data[buf_index].backing_result_cursor = self.result_cursor;
        trace!(
            "sqe create {}\tcursor_status {}",
            buf_index,
            self.result_cursor
        );

        self.result_cursor += 1;
        Ok(CreateSqeResult::CreatedAt(sqe_ptr))
    }

    pub unsafe fn pop_completion_queue(&mut self, ring: &mut IoUring) -> VIoResult<usize> {
        let mut cqe_ptr: *mut io_uring_cqe = ptr::null_mut();
        let ret = io_uring_wait_cqe(&mut ring.ring, &mut cqe_ptr);
        if ret != 0 {
            return Err(VIoError::IoUring_CqeWaitReturn {
                e: io::Error::from_raw_os_error(-ret),
                backtrace: Backtrace::capture(),
            });
        }
        if cqe_ptr.is_null() {
            return Err(VIoError::IoUring_CqeNullPointer {
                backtrace: Backtrace::capture(),
            });
        }

        let cqe_result = (*cqe_ptr).res;
        if cqe_result < 0 {
            return Err(VIoError::IoUring_CqeResultReturn {
                e: io::Error::from_raw_os_error(-cqe_result),
                backtrace: Backtrace::capture(),
            });
        }

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_buf_index = io_uring_cqe_get_data64(cqe_ptr) as usize;
        let cqe_buf_data = &mut self.backing_buf_ring_data[cqe_buf_index];
        let source: BackingBufEntry = self.backing_buf_ring[cqe_buf_index];
        let mut source_ref: &[u8] = &source;

        let target_offset = cqe_buf_data.result_offset;
        if target_offset > self.file_size {
            return Err(VIoError::IoUring_CqeOffsetTooBig {
                file_size: self.file_size,
                target_offset,
                backtrace: Backtrace::capture(),
            });
        }
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
        cqe_buf_data.is_free = true;
        self.result_buffer_done[cqe_buf_data.backing_result_cursor] = true;

        io_uring_cqe_seen(&mut ring.ring, cqe_ptr);
        Ok(cqe_buf_index)
    }

    // fn check_is_remaining_result_buffer(&mut self) -> bool {
    //     let mut cursor = self.result_buffer_done_fast_check_from;
    //     cursor += 1;
    //     while cursor != self.result_buffer_done.len() && self.result_buffer_done[cursor] {
    //         cursor += 1;
    //     }
    //     self.result_buffer_done_fast_check_from = cursor - 1;
    //     trace!(
    //         "reamining {} to {}",
    //         cursor,
    //         self.result_buffer_done.len() - 1
    //     );
    //     cursor != self.result_buffer_done.len() - 1
    // }
}

#[derive(PartialEq)]
pub enum CreateSqeResult {
    CreatedAt(*mut io_uring_sqe),
    Eof,
}

#[derive(Debug)]
pub enum CreateAllSqeResult {
    RefreshZero,
    RefreshTotal(usize),
    EofWithTotal(usize),
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
    backing_result_cursor: usize,
}

impl Default for BackingBufData {
    fn default() -> Self {
        BackingBufData {
            is_free: true,
            result_offset: usize::MAX,
            backing_result_cursor: usize::MAX,
        }
    }
}
