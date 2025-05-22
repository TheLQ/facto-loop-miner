use crate::navigator::planners::altare::start_altare_planner;
// use crate::navigator::planners::altare::start_altare_planner;
use crate::navigator::planners::debugplan::start_debug_planner;
use crate::navigator::planners::ruze::start_ruze_planner;
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
        // surface.validate();

        match 2 {
            1 => start_ruze_planner(&mut surface, &params),
            2 => start_altare_planner(&mut surface),
            9 => start_debug_planner(&mut surface)?,
            _ => unimplemented!(),
        }

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

// fn test_basic() {
//     let base = MineLocation {
//         patch_indexes: Vec::new(),
//         area: VArea::from_arbitrary_points([VPOINT_ZERO, VPOINT_TEN]),
//     };
//     let start = VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::North);
//     let end = VPointDirectionQ(VPoint::new(200, 200), FacDirectionQuarter::North);
//     run_mori(&mut surface, start, end, base)?;
// }

// fn run_mori(
//     surface: &mut VSurface
// ) -> VResult<()> {
//     let start = VPointDirectionQ(VPoint::new(100,100), FacDirectionQuarter::East);
//     let end = VPointDirectionQ(VPoint::new(-900, -900), FacDirectionQuarter::East);
//
//     match mori2_start(surface, start, end) {
//         MoriResult::Route { path, cost } => {
//             info!(
//                 "found {} path cost {} from {} to {} ",
//                 path.len(),
//                 cost,
//                 start,
//                 end
//             );
//             surface.add_mine_path(vec![MinePath {
//                 cost,
//                 links: path,
//                 mine_base,
//             }])?;
//         }
//         MoriResult::FailingDebug(stuff) => {
//             todo!("pathfinding failed")
//         }
//     }
//     info!("Mori execution {watch}");
//     Ok(())
// }
