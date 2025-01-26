use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::vsurface::VSurface;

pub(crate) struct Step20;

impl Step20 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step20 {})
    }
}

impl Step for Step20 {
    fn name(&self) -> &'static str {
        "step20-nav"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        Ok(())
    }
}
