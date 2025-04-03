use facto_loop_miner_common::LOCALE;
use num_format::ToFormattedString;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

/// Usage: format!("Task completed in {}")`
pub struct BasicWatch {
    start_time: Instant,
    end_time: Option<Instant>,
}

impl BasicWatch {
    pub fn start() -> Self {
        BasicWatch {
            start_time: Instant::now(),
            end_time: None,
        }
    }

    pub fn stop(&mut self) {
        self.end_time = Some(Instant::now())
    }

    pub fn duration(self) -> Duration {
        self.as_duration()
    }

    /// Publicly should consume, but internally Display can't consume
    fn as_duration(&self) -> Duration {
        let end = if let Some(v) = self.end_time {
            v
        } else {
            Instant::now()
        };
        end - self.start_time
    }
}

impl Display for BasicWatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        BasicWatchResult(self.as_duration()).fmt(f)
    }
}

pub struct BasicWatchResult(pub Duration);

impl Display for BasicWatchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!(
            "{}ms",
            self.0.as_millis().to_formatted_string(&LOCALE)
        ))
    }
}
