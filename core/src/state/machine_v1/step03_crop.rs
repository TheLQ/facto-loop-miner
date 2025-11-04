use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::vsurface::VSurface;

pub struct Step03 {}

/// Temporarily reduce surface size for easier development
impl Step03 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step03 {})
    }
}

pub const CROP_RADIUS: u32 = 3000;

impl Step for Step03 {
    fn name(&self) -> &'static str {
        "step03-crop"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        surface.pixels_mut().crop(CROP_RADIUS);

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}
