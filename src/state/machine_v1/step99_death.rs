use crate::state::machine::{Step, StepParams};
use std::process::exit;

pub struct Step99Death {}

impl Step99Death {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step99Death {})
    }
}

impl Step for Step99Death {
    fn name(&self) -> String {
        "step99-death".to_string()
    }

    fn transformer(&self, _: StepParams) {
        exit(0);
    }
}
