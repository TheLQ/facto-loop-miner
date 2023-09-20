use crate::state::machine::{Step, StepParams};
use crate::surface::pixel::Pixel;
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

    /// Temporarily reduce surface size for easier development
    fn transformer(&self, params: StepParams) {
        let surface_dir = params.step_history_out_dirs.last().unwrap();
        let mut surface = Surface::load(&surface_dir);

        surface = surface.crop(CROP_RADIUS);

        let mut count: u32 = 0;
        for val in &mut surface.buffer {
            if val == &Pixel::Water {
                *val = Pixel::Empty;
                count = count + 1;
            }
        }
        tracing::debug!("wiped out {} water", count);

        surface.save(&params.step_out_dir);
    }
}
