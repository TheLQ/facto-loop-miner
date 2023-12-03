use crate::LOCALE;
use num_format::ToFormattedString;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

pub struct BasicWatch {
    start_time: Instant,
    end_time: Option<Instant>
}

impl BasicWatch {
    pub fn start() -> Self {
        BasicWatch {
            start_time: Instant::now(),
            end_time: None
        }
    }
    
    pub fn stop(&mut self) {
        self.end_time = Some(Instant::now())
    }

    fn duration(&self) -> Duration {
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
        write!(
            f,
            "{} ms",
            self.duration().as_millis().to_formatted_string(&LOCALE)
        )
    }
}
