use crate::surface::pixel::Pixel;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use itertools::Itertools;
use num_format::ToFormattedString;
use std::collections::HashMap;
use strum::AsRefStr;

/// Strings are slow apparently at billions of entries. Solution: Enums!
#[derive(Default)]
pub struct FastMetrics {
    entity_metrics: HashMap<FastMetric, u32>,
    start: BasicWatch,
}

impl FastMetrics {
    pub fn new() -> Self {
        FastMetrics::default()
    }

    pub fn increment(&mut self, metric_name: FastMetric) {
        self.entity_metrics
            .entry(metric_name)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn log_final(&self) {
        let max_key_length = self
            .entity_metrics
            .keys()
            .fold(0, |total, key| total.max(key.to_str().len()));

        for (name, count) in self.entity_metrics.iter().sorted_by_key(|(name, _)| *name) {
            tracing::debug!(
                "-- {:max_key_length$} {:>10} --",
                name.to_str(),
                count.to_formatted_string(&LOCALE),
            );
        }
        tracing::debug!("-- Metrics in {} since creation --", self.start);
    }

    // pub fn process<I>(new_item_prefix: &str, iter: I)
    // where
    //     I: Iterator<Item = String>,
    // {
    //     let mut metric = crate::surface::metric::Metrics::new(new_item_prefix);
    //     for metric_name in iter {
    //         metric.increment_slow(&metric_name);
    //     }
    //     metric.log_final();
    // }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord, AsRefStr)]
pub enum FastMetric {
    PixelCvMapper_Empty,
    PixelCvMapper_NotEmpty,
    PixelCvMapper_FilterEmpty,
    PixelCvMapper_Filter(Pixel),
    VSurface_Pixel(Pixel),
}

impl FastMetric {
    pub fn to_str(&self) -> &str {
        self.as_ref()
    }
}
