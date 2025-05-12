use std::backtrace::Backtrace;

use crate::{blueprint::bpfac::position::FacBpPosition, common::vpoint::VPoint};
use itertools::Itertools;
use thiserror::Error;

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
        position: FacBpPosition,
        backtrace: Backtrace,
    },
    #[error("Serde {}", err)]
    Serde {
        #[from]
        err: serde_json::Error,
        backtrace: Backtrace,
    },
}

impl FError {
    pub fn from_serde(err: serde_json::Error) -> Self {
        Self::Serde {
            err,
            backtrace: Backtrace::capture(),
        }
    }
}

fn positions_to_strings(positions: &[VPoint]) -> String {
    positions.iter().map(|e| format!("{:?}", e)).join(",")
}
