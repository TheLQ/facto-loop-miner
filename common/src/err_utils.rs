use std::backtrace::Backtrace;
use std::marker::PhantomData;
use std::path::PathBuf;

pub fn xbt() -> Backtrace {
    Backtrace::capture()
}

/// IO Error Context to always bring path along
/// Integrates well with child error types
pub struct IOEC<E> {
    path: PathBuf,
    _p: PhantomData<E>,
}

pub struct IOECStd {
    pub path: PathBuf,
    pub err: std::io::Error,
}

pub struct IOECSerdeSimd {
    pub path: PathBuf,
    pub err: simd_json::Error,
}

impl<E> IOEC<E> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _p: PhantomData,
        }
    }

    pub fn io(&self) -> impl Fn(std::io::Error) -> E
    where
        E: From<IOECStd>,
    {
        |err: std::io::Error| {
            IOECStd {
                path: self.path.clone(),
                err,
            }
            .into()
        }
    }

    pub fn simd(&self) -> impl Fn(simd_json::Error) -> E
    where
        E: From<IOECSerdeSimd>,
    {
        |err: simd_json::Error| {
            IOECSerdeSimd {
                path: self.path.clone(),
                err,
            }
            .into()
        }
    }
}
