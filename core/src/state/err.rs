use crate::surfacev::err::VError;
use facto_loop_miner_common::err_bt::MyBacktrace;
use std::backtrace::Backtrace;
use thiserror::Error;

pub type XMachineResult<R> = Result<R, XMachineError>;

#[derive(Error, Debug)]
pub enum XMachineError {
    #[error("SurfaceFailure {}", e)]
    SurfaceFailure { e: VError },
}

impl MyBacktrace for XMachineError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            XMachineError::SurfaceFailure { e } => e.my_backtrace(),
        }
    }
}

impl From<VError> for XMachineError {
    fn from(e: VError) -> Self {
        XMachineError::SurfaceFailure { e }
    }
}
