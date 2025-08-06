use std::backtrace::Backtrace;
use std::error::Error;
use std::process::exit;
use tracing::error;

pub trait MyBacktrace {
    fn my_backtrace(&self) -> &Backtrace;
}

pub fn pretty_print_error(err: impl Error + MyBacktrace) {
    error!("⛔⛔⛔ FAIL: {err}\n{}", err.my_backtrace())
}

pub trait PrettyUnwrapMyBacktrace<T> {
    fn pretty_unwrap(self) -> T;
}

impl<T, E> PrettyUnwrapMyBacktrace<T> for Result<T, E>
where
    E: Error + MyBacktrace,
{
    fn pretty_unwrap(self) -> T {
        self.unwrap_or_else(|e| {
            pretty_print_error(e);
            exit(102);
        })
    }
}
