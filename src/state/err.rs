use crate::surfacev::err::VError;
use thiserror::Error;

pub type XMachineResult<R> = Result<R, XMachineError>;

#[derive(Error, Debug)]
pub enum XMachineError {
    #[error("asdf")]
    SurfaceFailure { e: VError },
}

impl From<VError> for XMachineError {
    fn from(e: VError) -> Self {
        XMachineError::SurfaceFailure { e }
    }
}
