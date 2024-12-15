use num_format::Locale;
use tracing::Level;

pub const LOCALE: Locale = Locale::en;

pub fn log_init(force_level: Option<Level>) {
    tracing_subscriber::fmt()
        .with_max_level(if let Some(level) = force_level {
            level
        } else {
            Level::INFO
        })
        .compact()
        .init();
}
