use std::backtrace::Backtrace;
use std::path::Path;
use std::{io, result};
use thiserror::Error;

pub type VIoResult<V> = result::Result<V, VIoError>;

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum VIoError {
    #[error("IoError {path} {e}")]
    IoError {
        path: String,
        e: io::Error,
        backtrace: Backtrace,
    },
    #[error("IoUring_SqeNullPointer")]
    IoUring_SqeNullPointer { backtrace: Backtrace },
    #[error("IoUring_CqeWaitReturn {e}")]
    IoUring_CqeWaitReturn { e: io::Error, backtrace: Backtrace },
    #[error("IoUring_CqeNullPointer")]
    IoUring_CqeNullPointer { backtrace: Backtrace },
    #[error("IoUring_CqeResultReturn {e}")]
    IoUring_CqeResultReturn { e: io::Error, backtrace: Backtrace },
    #[error("IoUring_CqeReadIncomplete expected {expected_size} got {actual_size}")]
    IoUring_CqeReadIncomplete {
        expected_size: usize,
        actual_size: usize,
        backtrace: Backtrace,
    },
    #[error("IoUring_CqeOffsetTooBig {file_size} {target_offset}")]
    IoUring_CqeOffsetTooBig {
        file_size: usize,
        target_offset: usize,
        backtrace: Backtrace,
    },
    #[error("IoUring_CqeCopyFailed {source_size} {target_size}")]
    IoUring_CqeCopyFailed {
        source_size: usize,
        target_size: usize,
        backtrace: Backtrace,
    },
}

impl VIoError {
    pub fn io_error(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        |e| VIoError::IoError {
            e,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            VIoError::IoError { backtrace, .. }
            | VIoError::IoUring_SqeNullPointer { backtrace, .. }
            | VIoError::IoUring_CqeWaitReturn { backtrace, .. }
            | VIoError::IoUring_CqeNullPointer { backtrace, .. }
            | VIoError::IoUring_CqeResultReturn { backtrace, .. }
            | VIoError::IoUring_CqeReadIncomplete { backtrace, .. }
            | VIoError::IoUring_CqeOffsetTooBig { backtrace, .. }
            | VIoError::IoUring_CqeCopyFailed { backtrace, .. } => backtrace,
        }
    }
}
