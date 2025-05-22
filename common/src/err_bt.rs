use std::backtrace::Backtrace;
use std::error::Error;
use tracing::error;

pub trait MyBacktrace {
    fn my_backtrace(&self) -> &Backtrace;
}

pub fn pretty_print_error(err: impl Error + MyBacktrace) {
    error!("⛔⛔⛔ FAIL: {err}\n{}", err.my_backtrace())
}
