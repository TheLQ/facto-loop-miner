use crate::surfacev::vpoint::VPoint;
use itertools::Itertools;
use opencv::core::Point2f;
use std::backtrace::Backtrace;
use std::io;
use thiserror::Error;
use tracing::error;

pub type VResult<R> = Result<R, VError>;

#[derive(Error, Debug)]
pub enum VError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(positions))]
    XYOutOfBounds { positions: Vec<VPoint> },
    #[error("XYNotInteger point {}", position_to_strings_f32(position))]
    XYNotInteger { position: Point2f },
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

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}

fn position_to_strings(position: &VPoint) -> String {
    format!("{:?}", position)
}

fn position_to_strings_f32(position: &Point2f) -> String {
    format!("{:?}", position)
}
