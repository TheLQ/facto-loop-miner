use crate::state::machine::{Step, StepParams};
use std::process::exit;

pub struct Step99_Death {}

impl Step99_Death {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step99_Death {})
    }
}

impl Step for Step99_Death {
    fn name(&self) -> String {
        "step99-death".to_string()
    }

    fn transformer(&self, _: StepParams) {
        exit(0);
    }
}
