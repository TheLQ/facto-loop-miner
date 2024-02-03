use std::backtrace::Backtrace;
use std::fs::{File, OpenOptions};
use std::io::Result as IoResult;
use std::mem::ManuallyDrop;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::{io, mem, ptr};

use num_format::ToFormattedString;
use tracing::{debug, info, trace, warn};
use uring_sys2::{
    io_uring_cqe, io_uring_cqe_get_data64, io_uring_cqe_seen, io_uring_get_sqe, io_uring_prep_read,
    io_uring_prep_read_fixed, io_uring_register_buffers, io_uring_register_files, io_uring_sqe,
    io_uring_sqe_set_data64, io_uring_sqe_set_flags, io_uring_unregister_buffers,
    io_uring_unregister_files, io_uring_wait_cqe, IOSQE_FIXED_FILE,
};

use crate::err::{VIoError, VIoResult};
use crate::io::USIZE_BYTES;
use crate::io_uring::IoUring;
use crate::io_uring_common::{allocate_page_size_aligned, PAGE_SIZE};
use crate::LOCALE;

pub const BUF_RING_COUNT: usize = 32;
// const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 256; // 1 MibiByte
const BUF_RING_ENTRY_SIZE: usize = PAGE_SIZE * 32; // 128 KiB
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
pub struct IoUringFileCopying {
    file_handle: File,
    file_registered_index: i32,
    backing_iovecs: Vec<libc::iovec>,
    backing_buf_ring: ManuallyDrop<Box<BackingBufRing>>,
    backing_buf_ring_data: [BackingBufData; BUF_RING_COUNT],
    result_buffer: ManuallyDrop<Vec<u8>>,
    result_cursor: usize,
}

impl IoUringFileCopying {
    pub fn open(path: &Path, ring: &mut IoUring) -> IoResult<Self> {
        // let file_handle = File::open(path)?;
        let file_handle = OpenOptions::new()
            .read(true)
            // .custom_flags(libc::O_DIRECT)
            .open(path)?;
        let file_size = file_handle.metadata()?.len() as usize;

        // Internal Fixed Files have less per-operation overhead than File Descriptors
        let fds = [file_handle.as_raw_fd()];
        let register_result = unsafe { io_uring_register_files(&mut ring.ring, fds.as_ptr(), 1) };
        assert_eq!(register_result, 0, "register files failed");

        let (backing_buf_ptr, _) = allocate_page_size_aligned::<BackingBufRing>();
        let mut backing_buf_ring = unsafe { ManuallyDrop::new(Box::from_raw(backing_buf_ptr)) };

        // Register
        let backing_iovecs: Vec<libc::iovec> = backing_buf_ring
            .iter_mut()
            .map(|backing_buf| libc::iovec {
                iov_len: backing_buf.len(),
                iov_base: backing_buf.as_mut_ptr() as *mut libc::c_void,
            })
            .collect();
        unsafe {
            io_uring_register_buffers(
                &mut ring.ring,
                backing_iovecs.as_ptr(),
                backing_iovecs.len() as libc::c_uint,
            )
        };

        Ok(IoUringFileCopying {
            file_handle,
            file_registered_index: 0,
            backing_iovecs,
            backing_buf_ring,
            backing_buf_ring_data: [BackingBufData::default(); BUF_RING_COUNT],
            result_buffer: ManuallyDrop::new(vec![0u8; file_size]),
            result_cursor: 0,
        })
    }

    /// Superfast io_uring
    pub fn read_entire_file(&mut self, ring: &mut IoUring) -> VIoResult<()> {
        let mut total_submissions: usize = 0;
        let mut total_completions: usize = 0;

        // pre-fill Completion Queue
        for buf_index in 0..self.backing_buf_ring_data.len() {
            let sqe =
                unsafe { self.refresh_submission_queue_read_for_buffer_index(ring, buf_index)? };
            if let CreateSqeResult::CreatedAt(_) = sqe {
                total_submissions += 1;
            } else {
                panic!("file too small");
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
                "total_submissions {} total_completions {}",
                total_submissions, total_completions
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
        eprintln!("INTO RESULT");

        unsafe {
            io_uring_unregister_files(&mut ring.ring);
            io_uring_unregister_buffers(&mut ring.ring);
            libc::munmap(
                self.backing_buf_ring.as_mut_ptr() as *mut libc::c_void,
                mem::size_of::<BackingBufRing>(),
            );
        }

        let (_, xy_vec_aligned, _) = unsafe { self.result_buffer.align_to_mut::<usize>() };
        assert_eq!(xy_vec_aligned.len(), result_len_usize, "invalid size");
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
        if offset > self.file_size() {
            info!("EOF at result_cursor {}", self.result_cursor);
            return Ok(CreateSqeResult::Eof);
        }
        let sqe_ptr = io_uring_get_sqe(&mut ring.ring);
        if sqe_ptr.is_null() {
            return Err(VIoError::IoUring_SqeNullPointer {
                backtrace: Backtrace::capture(),
            });
        }

        io_uring_prep_read_fixed(
            sqe_ptr,
            self.file_registered_index,
            self.backing_buf_ring[buf_index].as_mut_ptr() as *mut libc::c_void,
            BUF_RING_ENTRY_SIZE as libc::c_uint,
            offset as u64,
            buf_index as libc::c_int,
        );
        io_uring_sqe_set_flags(sqe_ptr, IOSQE_FIXED_FILE);
        io_uring_sqe_set_data64(sqe_ptr, buf_index as u64);
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
        } else if cqe_result != BUF_RING_ENTRY_SIZE as i32 {
            // return Err(VIoError::IoUring_CqeReadIncomplete {
            //     expected_size: BUF_RING_ENTRY_SIZE,
            //     actual_size: cqe_result as usize,
            //     backtrace: Backtrace::capture(),
            // });
            warn!(
                "expected {} got {}",
                BUF_RING_ENTRY_SIZE.to_formatted_string(&LOCALE),
                cqe_result.to_formatted_string(&LOCALE)
            );
        }

        // let buffer_id = unsafe {
        //     (*cqe_ptr).flags >> IORING_CQE_BUFFER_SHIFT;
        // };
        let cqe_buf_index = io_uring_cqe_get_data64(cqe_ptr) as usize;
        let target_offset_start = self.backing_buf_ring_data[cqe_buf_index].result_offset;
        if target_offset_start > self.file_size() {
            return Err(VIoError::IoUring_CqeOffsetTooBig {
                file_size: self.file_size(),
                target_offset: target_offset_start,
                backtrace: Backtrace::capture(),
            });
        }

        let source: BackingBufEntry = self.backing_buf_ring[cqe_buf_index];
        let source_ref: &[u8];
        let mut target_offset_end = target_offset_start + source.len();
        if target_offset_end < self.file_size() {
            source_ref = &source;
        } else {
            target_offset_end = self.file_size();
            source_ref = &source[0..(target_offset_end - target_offset_start)];
        }

        trace!(
            "cqe buf {} range {} to {}",
            cqe_buf_index,
            target_offset_start.to_formatted_string(&LOCALE),
            target_offset_end.to_formatted_string(&LOCALE)
        );

        let target = &mut self.result_buffer[target_offset_start..target_offset_end];
        target.copy_from_slice(source_ref);

        io_uring_cqe_seen(&mut ring.ring, cqe_ptr);
        Ok(cqe_buf_index)
    }

    fn file_size(&self) -> usize {
        self.result_buffer.len()
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
    result_offset: usize,
    backing_result_cursor: usize,
}

impl Default for BackingBufData {
    fn default() -> Self {
        BackingBufData {
            result_offset: usize::MAX,
            backing_result_cursor: usize::MAX,
        }
    }
}
