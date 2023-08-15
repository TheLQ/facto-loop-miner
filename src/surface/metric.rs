use crate::LOCALE;
use num_format::ToFormattedString;
use std::collections::HashMap;

#[derive(Default)]
pub struct Metrics {
    entity_metrics: HashMap<String, u32>,
}

impl Metrics {
    pub fn increment(&mut self, metric_name: &str) {
        self.entity_metrics
            .entry(metric_name.to_string())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn log_final(self) {
        for (name, count) in self.entity_metrics {
            println!(
                "-- Added {}\t\t{} ",
                name,
                count.to_formatted_string(&LOCALE)
            );
        }
    }
}
