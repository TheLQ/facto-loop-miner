use rcon_client::RCONError;
use serde::__private::de::AdjacentlyTaggedEnumVariantSeed;
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
    #[error("DefineFailed {}", truncate_huge_lua(lua_text))]
    // #[error("DefineFailed")]
    DefineFailed {
        lua_text: String,
        backtrace: Backtrace,
    },
    #[error("TooLargeRequest {}", truncate_huge_lua(lua_text))]
    TooLargeRequest {
        lua_text: String,
        backtrace: Backtrace,
    },
    #[error("IoError {path} {e}")]
    IoError {
        path: String,
        e: io::Error,
        backtrace: Backtrace,
    },
}

impl AdmiralError {
    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            AdmiralError::Rcon { backtrace, source } => backtrace,
            AdmiralError::LuaBlankCommand { backtrace } => backtrace,
            AdmiralError::LuaResultNotEmpty {
                backtrace,
                body,
                command,
            } => backtrace,
            AdmiralError::LuaResultEmpty { backtrace, command } => backtrace,
            AdmiralError::DestroyFailed { backtrace } => backtrace,
            AdmiralError::DefineFailed {
                backtrace,
                lua_text,
            } => backtrace,
            AdmiralError::TooLargeRequest {
                backtrace,
                lua_text,
            } => backtrace,
            AdmiralError::IoError { backtrace, path, e } => backtrace,
        }
    }
}

fn truncate_huge_lua(input: &str) -> String {
    if input.len() < 100 {
        input.to_string()
    } else {
        format!(
            "{}...truncate {} chars....{}",
            &input[0..100],
            input.len() - 200,
            &input[(input.len() - 100)..]
        )
    }
}
