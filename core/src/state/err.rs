use crate::surfacev::err::VError;
use std::backtrace::Backtrace;
use thiserror::Error;

pub type XMachineResult<R> = Result<R, XMachineError>;

#[derive(Error, Debug)]
pub enum XMachineError {
    #[error("SurfaceFailure {}", e)]
    SurfaceFailure { e: VError },
}

impl XMachineError {
    pub fn my_backtrace(&self) -> &Backtrace {
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
