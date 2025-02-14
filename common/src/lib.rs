use num_format::Locale;

pub mod err_utils;
mod logger;

pub use logger::log_init_debug;
pub use logger::log_init_trace;

pub const LOCALE: Locale = Locale::en;
