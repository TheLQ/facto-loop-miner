use facto_loop_miner_common::err_utils::{xbt, IOECSerdeSimd, IOECStd, IOEC};
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_io::err::VIoError;
use image::ImageError;
use itertools::Itertools;
use std::backtrace::Backtrace;
use std::io;
use std::path::{Path, PathBuf};
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
    #[error("IoError {path} {err}")]
    IoError {
        path: String,
        err: io::Error,
        backtrace: Backtrace,
    },
    #[error("UnknownName {name}")]
    UnknownName { name: String, backtrace: Backtrace },
    #[error("SimdJsonFail {err} for {path}")]
    SimdJsonFail {
        err: simd_json::Error,
        path: String,
        backtrace: Backtrace,
    },
    #[error("NotADirectory {path}")]
    NotADirectory { path: String, backtrace: Backtrace },
    #[error("Image {path}")]
    Image {
        err: ImageError,
        path: String,
        backtrace: Backtrace,
    },
    #[error("VIoError {0}")]
    VIoError(#[from] VIoError),
}

impl VError {
    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            VError::XYOutOfBounds { backtrace, .. }
            | VError::IoError { backtrace, .. }
            | VError::UnknownName { backtrace, .. }
            | VError::SimdJsonFail { backtrace, .. }
            | VError::NotADirectory { backtrace, .. }
            | VError::Image { backtrace, .. } => backtrace,
            VError::VIoError(e) => e.my_backtrace(),
        }
    }

    pub fn ioec(path: impl Into<PathBuf>) -> IOEC<Self> {
        IOEC::new(path.into())
    }

    pub fn simd_json(path: &Path) -> impl FnOnce(simd_json::Error) -> Self + '_ {
        |err| VError::SimdJsonFail {
            err,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    // Use like `read().map_err(VError::io_error)`
    fn io_error(path: &Path) -> impl FnOnce(io::Error) -> Self + '_ {
        |err| VError::IoError {
            err,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn image(path: &Path) -> impl FnOnce(ImageError) -> Self + '_ {
        |err| Self::Image {
            path: path.to_string_lossy().to_string(),
            err,
            backtrace: xbt(),
        }
    }
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(VPoint::display).join(",")
}

impl From<IOECStd> for VError {
    fn from(IOECStd { path, err }: IOECStd) -> Self {
        Self::IoError {
            path: path.to_string_lossy().to_string(),
            err,
            backtrace: xbt(),
        }
    }
}

impl From<IOECSerdeSimd> for VError {
    fn from(IOECSerdeSimd { path, err }: IOECSerdeSimd) -> Self {
        Self::SimdJsonFail {
            path: path.to_string_lossy().to_string(),
            err,
            backtrace: xbt(),
        }
    }
}
