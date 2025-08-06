use facto_loop_miner_common::err_bt::MyBacktrace;
use rcon_client::RCONError;
use std::backtrace::Backtrace;
use std::io;
use thiserror::Error;
use tracing::error;

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
    #[error("LuaCheckedUnknown {body}")]
    LuaCheckedUnknown {
        command: String,
        body: String,
        backtrace: Backtrace,
    },
    #[error("DestroyFailed")]
    DestroyFailed { backtrace: Backtrace },
    #[error("DefineFailed {}", truncate_huge_lua(command))]
    // #[error("DefineFailed")]
    DefineFailed {
        command: String,
        backtrace: Backtrace,
    },
    #[error("TooLargeRequest {}", truncate_huge_lua(command))]
    TooLargeRequest {
        command: String,
        backtrace: Backtrace,
    },
    #[error("IoError {path} {e}")]
    IoError {
        path: String,
        e: io::Error,
        backtrace: Backtrace,
    },
    // #[error("VError {0}")]
    // SurfaceError(#[from] VError),
}

impl MyBacktrace for AdmiralError {
    fn my_backtrace(&self) -> &Backtrace {
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
            // AdmiralError::SurfaceError(v) => v.my_backtrace(),
        }
    }
}

impl AdmiralError {
    pub fn my_command(&self) -> Option<&String> {
        match self {
            AdmiralError::Rcon { .. }
            // | AdmiralError::SurfaceError { .. }
            | AdmiralError::DestroyFailed { .. }
            | AdmiralError::IoError { .. }
            | AdmiralError::LuaBlankCommand { .. } => None,
            AdmiralError::LuaResultNotEmpty { command, .. }
            | AdmiralError::LuaResultEmpty { command, .. }
            | AdmiralError::LuaCheckedEmpty { command, .. }
            | AdmiralError::LuaCheckedError { command, .. }
            | AdmiralError::LuaCheckedUnknown { command, .. }
            | AdmiralError::DefineFailed { command, .. }
            | AdmiralError::TooLargeRequest { command, .. } => Some(command),
        }
    }
}

pub fn truncate_huge_lua(input: &str) -> String {
    if input.len() < 3000 {
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

pub fn pretty_panic_admiral(err: AdmiralError) {
    if let Some(cmd) = err.my_command() {
        error!("raw command -- {}", cmd);
    }
    error!("⛔⛔⛔ DEAD: {err}\n{}", err.my_backtrace());
}
