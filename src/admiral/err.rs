use rcon_client::RCONError;
use std::backtrace::Backtrace;
use std::io;
use std::marker::ConstParamTy;
use thiserror::Error;

pub type AdmiralResult<T> = Result<T, AdmiralError>;

#[derive(Error, Debug)]
pub enum AdmiralError {
    #[error("RCON {source}")]
    Rcon {
        #[from]
        source: RCONError,
        backtrace: Backtrace,
    },
    #[error("LuaBlankCommand")]
    LuaBlankCommand { backtrace: Backtrace },
    #[error("LuaResultNotEmpty {body}")]
    LuaResultNotEmpty {
        command: String,
        body: String,
        backtrace: Backtrace,
    },
    #[error("LuaResultEmpty")]
    LuaResultEmpty {
        command: String,
        backtrace: Backtrace,
    },
    #[error("DestroyFailed")]
    DestroyFailed { backtrace: Backtrace },
}
