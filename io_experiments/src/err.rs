use facto_loop_miner_common::err_utils::xbt;
use std::backtrace::Backtrace;
use std::fmt::Debug;
use std::io::Error;
use std::path::Path;
use std::{io, result};
use thiserror::Error;
use tracing::error;

pub type VIoResult<V> = result::Result<V, VIoError>;

#[derive(Error, Debug)]
#[allow(non_camel_case_types)]
pub enum VIoError {
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
    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            VIoError::IoUring_SqeNullPointer { backtrace, .. }
            | VIoError::IoUring_CqeWaitReturn { backtrace, .. }
            | VIoError::IoUring_CqeNullPointer { backtrace, .. }
            | VIoError::IoUring_CqeResultReturn { backtrace, .. }
            | VIoError::IoUring_CqeReadIncomplete { backtrace, .. }
            | VIoError::IoUring_CqeOffsetTooBig { backtrace, .. }
            | VIoError::IoUring_CqeCopyFailed { backtrace, .. } => backtrace,
        }
    }
}

pub type VStdIoResult<V> = Result<V, VStdIoError>;

pub struct VStdIoError {
    e: io::Error,
    backtrace: Backtrace,
}

impl From<io::Error> for VStdIoError {
    fn from(e: Error) -> Self {
        Self {
            backtrace: xbt(),
            e,
        }
    }
}

pub trait VPathUnwrapper<T> {
    fn unwrap_path(self, path: impl AsRef<Path>) -> T;
}

impl<T> VPathUnwrapper<T> for Result<T, VStdIoError> {
    fn unwrap_path(self, path: impl AsRef<Path>) -> T {
        match self {
            Ok(t) => t,
            Err(VStdIoError { e, backtrace }) => {
                error!("backtrace {}", backtrace);
                panic!("FAILED {} at {}", e, path.as_ref().display())
            }
        }
    }
}
