use std::time::Instant;

pub struct BasicWatch {
    start_time: Instant,
}

impl BasicWatch {
    pub fn new() -> Self {
        BasicWatch {
            start_time: Instant::now(),
        }
    }

    pub fn get_duration_seconds(&self) -> u64 {
        (Instant::now() - self.start_time).as_secs()
    }
}
