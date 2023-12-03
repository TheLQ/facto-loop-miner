use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::vsurface::VSurface;

pub struct Step03 {}

impl Step03 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step03 {})
    }
}

const CROP_RADIUS: u32 = 4000;

impl Step for Step03 {
    fn name(&self) -> String {
        "step03-crop".to_string()
    }

    /// Temporarily reduce surface size for easier development
    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        surface.crop(CROP_RADIUS);

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}
