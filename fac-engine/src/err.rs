use std::backtrace::Backtrace;

use itertools::Itertools;
use thiserror::Error;

use crate::common::{cvpoint::Point2f, vpoint::VPoint};

pub type FResult<T> = Result<T, FError>;

#[derive(Error, Debug)]
pub enum FError {
    #[error("XYOutOfBounds positions {}", positions_to_strings(positions))]
    XYOutOfBounds {
        positions: Vec<VPoint>,
        backtrace: Backtrace,
    },
    #[error("XYNotInteger point {:?}", position)]
    XYNotInteger {
        position: Point2f,
        backtrace: Backtrace,
    },
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}
