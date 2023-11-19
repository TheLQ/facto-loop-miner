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
    LuaResultNotEmpty { body: String, backtrace: Backtrace },
    #[error("LuaResultEmpty")]
    LuaResultEmpty { backtrace: Backtrace },
}
