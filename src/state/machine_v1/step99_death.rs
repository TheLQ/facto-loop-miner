use crate::state::machine::{Step, StepParams};
use std::process::exit;

pub struct Step99 {}

impl Step99 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step99 {})
    }
}

impl Step for Step99 {
    fn name(&self) -> String {
        "step99-death".to_string()
    }

    fn transformer(&self, _: StepParams) {
        exit(0);
    }
}
