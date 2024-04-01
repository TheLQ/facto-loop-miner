use crate::surface::pixel::Pixel;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use enum_map::EnumMap;
use num_format::ToFormattedString;
use std::fmt::Display;

/// Strings are slow apparently at billions of entries. Solution: Enums!
#[derive(Default)]
pub struct FastMetrics {
    log_prefix: String,
    entity_metrics: EnumMap<FastMetric, u32>,
    start: BasicWatch,
}

impl FastMetrics {
    pub fn new(log_prefix: String) -> Self {
        FastMetrics {
            log_prefix,
            ..Default::default()
        }
    }

    pub fn increment(&mut self, metric_name: FastMetric) {
        self.entity_metrics[metric_name] += 1
    }

    pub fn log_final(self) {
        let mut used_entries: Vec<(FastMetric, u32)> = self
            .entity_metrics
            .into_iter()
            .filter(|(_, size)| *size != 0)
            .collect();
        used_entries.sort_by_key(|(key, _)| key.clone());

        let max_key_length = used_entries
            .iter()
            .fold(0, |total, (metric, _)| total.max(metric.to_string().len()));

        for (metric, count) in used_entries {
            tracing::debug!(
                "-- {:max_key_length$} {:>10} --",
                metric.to_string(),
                count.to_formatted_string(&LOCALE),
            );
        }

        tracing::debug!(
            "-- Metrics {} in {} since creation --",
            self.log_prefix,
            self.start
        );
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
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, enum_map::Enum)]
pub enum FastMetric {
    PixelCvMapper_Empty,
    PixelCvMapper_NotEmpty,
    PixelCvMapper_FilterEmpty,
    PixelCvMapper_Filter(Pixel),
    VSurface_Pixel(Pixel),
}

impl Display for FastMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
