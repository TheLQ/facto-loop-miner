use crate::surfacev::err::VError;
use rcon_client::RCONError;
use std::backtrace::Backtrace;
use std::io;
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
    #[error("LuaCheckedEmpty")]
    LuaCheckedEmpty {
        command: String,
        backtrace: Backtrace,
    },
    #[error("LuaCheckedError {errors}")]
    LuaCheckedError {
        command: String,
        errors: String,
        backtrace: Backtrace,
    },
    #[error("LuaCheckedUnknown")]
    LuaCheckedUnknown {
        command: String,
        body: String,
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
    #[error("VError")]
    SurfaceError(#[from] VError),
}

impl AdmiralError {
    pub fn my_backtrace(&self) -> &Backtrace {
        match self {
            AdmiralError::Rcon { backtrace, .. }
            | AdmiralError::LuaBlankCommand { backtrace }
            | AdmiralError::LuaResultNotEmpty { backtrace, .. }
            | AdmiralError::LuaResultEmpty { backtrace, .. }
            | AdmiralError::LuaCheckedEmpty { backtrace, .. }
            | AdmiralError::LuaCheckedError { backtrace, .. }
            | AdmiralError::LuaCheckedUnknown { backtrace, .. }
            | AdmiralError::DestroyFailed { backtrace, .. }
            | AdmiralError::DefineFailed { backtrace, .. }
            | AdmiralError::TooLargeRequest { backtrace, .. }
            | AdmiralError::IoError { backtrace, .. } => backtrace,
            AdmiralError::SurfaceError(v) => v.my_backtrace(),
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

pub fn pretty_panic_admiral(err: AdmiralError) -> String {
    format!("{}", err)
}
