use crate::state::machine::{Step, StepParams};
use crate::surface::surface::Surface;

struct Step04 {}

impl Step for Step04 {
    fn name(&self) -> String {
        "step04-contours".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let previous_step_dir = params.step_history_out_dirs.last().unwrap();

        // Surface::load(params.)
    }
}
