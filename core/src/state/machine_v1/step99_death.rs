use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams, DEATH_STEP_NAME};

pub struct Step99Death {}

impl Step99Death {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step99Death {})
    }
}

impl Step for Step99Death {
    fn name(&self) -> &'static str {
        DEATH_STEP_NAME
    }

    fn transformer(&self, _: StepParams) -> XMachineResult<()> {
        panic!("UNEXPECTED DEATH RUN");
    }
}
