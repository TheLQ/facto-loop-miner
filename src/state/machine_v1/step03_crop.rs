use crate::state::machine::{Step, StepParams};
use crate::surface::surface::Surface;

pub struct Step03 {}

impl Step03 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step03 {})
    }
}

const CROP_RADIUS: i32 = 4000;

impl Step for Step03 {
    fn name(&self) -> String {
        "step03-crop".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let surface_dir = params.step_history_out_dirs.last().unwrap();
        let surface = Surface::load(&surface_dir);

        let cropped_surface = surface.crop(CROP_RADIUS);

        cropped_surface.save(&params.step_out_dir);
    }
}
