use facto_loop_miner_common::err_bt::MyBacktrace;
use facto_loop_miner_common::err_utils::xbt;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_io::err::{UringError, VStdIoError};
use image::ImageError;
use itertools::Itertools;
use simd_json::prelude::ArrayTrait;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::path::Path;
use thiserror::Error;
use tracing::error;

pub type VResult<R> = Result<R, VError>;

#[derive(Error, Debug)]
pub enum VError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(pos))]
    MiniXYOutOfBounds {
        pos: Vec<VPoint>,
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
    // #[error("NotADirectory {path}")]
    // NotADirectory { path: String, backtrace: Backtrace },
    #[error("Image {path}")]
    Image {
        err: ImageError,
        path: String,
        backtrace: Backtrace,
    },
    #[error("UringError {0}")]
    UringError(#[from] UringError),
}

impl MyBacktrace for VError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            VError::MiniXYOutOfBounds { backtrace, .. }
            | VError::IoError { backtrace, .. }
            | VError::UnknownName { backtrace, .. }
            | VError::SimdJsonFail { backtrace, .. }
            // | VError::NotADirectory { backtrace, .. }
            | VError::Image { backtrace, .. } => backtrace,
            VError::UringError(e) => e.my_backtrace(),
        }
    }
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(VPoint::to_string).join(",")
}

pub trait CoreConvertPathResult<T, E> {
    fn convert(self, path: impl AsRef<Path>) -> Result<T, E>;
}

impl<T> CoreConvertPathResult<T, VError> for Result<T, VStdIoError> {
    fn convert(self, path: impl AsRef<Path>) -> Result<T, VError> {
        self.map_err(|VStdIoError { e, backtrace }| VError::IoError {
            err: e,
            backtrace,
            path: path.as_ref().to_string_lossy().to_string(),
        })
    }
}

impl<T> CoreConvertPathResult<T, VError> for Result<T, io::Error> {
    fn convert(self, path: impl AsRef<Path>) -> Result<T, VError> {
        self.map_err(|e| VError::IoError {
            err: e,
            backtrace: xbt(),
            path: path.as_ref().to_string_lossy().to_string(),
        })
    }
}

impl<T> CoreConvertPathResult<T, VError> for Result<T, simd_json::Error> {
    fn convert(self, path: impl AsRef<Path>) -> Result<T, VError> {
        self.map_err(|e| VError::SimdJsonFail {
            err: e,
            backtrace: xbt(),
            path: path.as_ref().to_string_lossy().to_string(),
        })
    }
}

impl<T> CoreConvertPathResult<T, VError> for Result<T, ImageError> {
    fn convert(self, path: impl AsRef<Path>) -> Result<T, VError> {
        self.map_err(|e| VError::Image {
            err: e,
            backtrace: xbt(),
            path: path.as_ref().to_string_lossy().to_string(),
        })
    }
}

pub type XYOutOfBoundsResult<V> = Result<V, XYOutOfBoundsError>;

pub struct XYOutOfBoundsError {
    pos: Vec<VPoint>,
    backtrace: Backtrace,
}

impl XYOutOfBoundsError {
    pub fn new(pos: Vec<VPoint>) -> Self {
        XYOutOfBoundsError {
            pos,
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<XYOutOfBoundsError> for VError {
    fn from(XYOutOfBoundsError { pos, backtrace }: XYOutOfBoundsError) -> Self {
        Self::MiniXYOutOfBounds { pos, backtrace }
    }
}

impl Debug for XYOutOfBoundsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "XYOutOfBoundsError {}", self.pos.iter().join(","))
    }
}

impl Display for XYOutOfBoundsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
