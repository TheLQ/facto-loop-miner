use crate::state::machine::{Step, StepParams, DEATH_STEP_NAME};
use std::process::exit;

pub struct Step99Death {}

impl Step99Death {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step99Death {})
    }
}

impl Step for Step99Death {
    fn name(&self) -> String {
        DEATH_STEP_NAME.to_string()
    }

    fn transformer(&self, _: StepParams) {
        tracing::debug("UNEXPECTED DEATH RUN");
    }
}
