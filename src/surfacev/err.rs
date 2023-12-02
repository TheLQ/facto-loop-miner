use crate::surfacev::vpoint::VPoint;
use itertools::Itertools;
use opencv::core::Point2f;
use std::backtrace::Backtrace;
use std::io;
use std::path::Path;
use thiserror::Error;
use tracing::error;

pub type VResult<R> = Result<R, VError>;

#[derive(Error, Debug)]
pub enum VError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(positions))]
    XYOutOfBounds {
        positions: Vec<VPoint>,
        backtrace: Backtrace,
    },
    #[error("XYNotInteger point {}", position_to_strings_f32(position))]
    XYNotInteger {
        position: Point2f,
        backtrace: Backtrace,
    },
    #[error("IoError {path} {e}")]
    IoError {
        path: String,
        e: io::Error,
        backtrace: Backtrace,
    },
    #[error("UnknownName {name}")]
    UnknownName { name: String, backtrace: Backtrace },
    #[error("SimdJsonFail {e}")]
    SimdJsonFail {
        e: simd_json::Error,
        backtrace: Backtrace,
    },
    #[error("NotADirectory {path}")]
    NotADirectory { path: String, backtrace: Backtrace },
}

impl VError {
    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            VError::XYOutOfBounds { backtrace, .. }
            | VError::XYNotInteger { backtrace, .. }
            | VError::IoError { backtrace, .. }
            | VError::UnknownName { backtrace, .. }
            | VError::SimdJsonFail { backtrace, .. }
            | VError::NotADirectory { backtrace, .. } => backtrace,
        }
    }

    /// IoError factory. Use like `read().map_err(VError::io_error)`
    pub fn io_error(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        |e| VError::IoError {
            e,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}

fn position_to_strings(position: &VPoint) -> String {
    format!("{:?}", position)
}

fn position_to_strings_f32(position: &Point2f) -> String {
    format!("{:?}", position)
}
