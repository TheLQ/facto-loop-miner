use std::backtrace::Backtrace;
use std::path::Path;
use std::{io, result};
use thiserror::Error;

pub type VIoResult<V> = result::Result<V, VIoError>;

#[derive(Error, Debug)]
pub enum VIoError {
    #[error("IoError {path} {e}")]
    IoError {
        path: String,
        e: io::Error,
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
}
