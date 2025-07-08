use num_format::Locale;

pub mod duration;
pub mod err_bt;
pub mod err_utils;
mod logger;

pub use logger::log_init_debug;
pub use logger::log_init_trace;

pub const LOCALE: Locale = Locale::en;
pub use logger::EXECUTOR_TAG;
