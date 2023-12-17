use crate::LOCALE;
use itertools::Itertools;
use num_format::ToFormattedString;
use std::collections::HashMap;

pub struct Metrics {
    new_item_log_prefix: String,
    entity_metrics: HashMap<String, u32>,
}

impl Metrics {
    pub fn new(new_item_log_prefix: &str) -> Self {
        Metrics {
            new_item_log_prefix: new_item_log_prefix.to_string(),
            entity_metrics: HashMap::new(),
        }
    }

    pub fn increment(&mut self, metric_name: String) {
        self.entity_metrics
            .entry(metric_name)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn increment_slow(&mut self, metric_name: &str) {
        self.entity_metrics
            .entry(metric_name.to_string())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn log_final(&self) {
        let max_key_length = self
            .entity_metrics
            .iter()
            .fold(0, |total, (key, _)| total.max(key.len()));

        for (name, count) in self
            .entity_metrics
            .iter()
            .sorted_by_key(|(name, _count)| (**name).clone())
        {
            tracing::debug!(
                "-- {} {:max_key_length$} {:>10} --",
                self.new_item_log_prefix,
                name,
                count.to_formatted_string(&LOCALE),
            );
        }
    }

    pub fn process<I>(new_item_prefix: &str, iter: I)
    where
        I: Iterator<Item = String>,
    {
        let mut metric = Metrics::new(new_item_prefix);
        for metric_name in iter {
            metric.increment_slow(&metric_name);
        }
        metric.log_final();
    }
}
