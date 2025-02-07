use crate::navigator::mori::{mori2_start, MoriResult};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surfacev::err::VResult;
use crate::surfacev::mine::{MineBase, MinePath};
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_TEN, VPOINT_ZERO};
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use tracing::info;

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

        let base = MineBase {
            patch_indexes: Vec::new(),
            area: VArea::from_arbitrary_points([VPOINT_ZERO, VPOINT_TEN]),
        };
        let start = VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::North);
        let end = VPointDirectionQ(VPoint::new(200, 200), FacDirectionQuarter::North);
        run_mori(&mut surface, start, end, base)?;

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

fn run_mori(
    surface: &mut VSurface,
    start: VPointDirectionQ,
    end: VPointDirectionQ,
    mine_base: MineBase,
) -> VResult<()> {
    let watch = BasicWatch::start();
    match mori2_start(surface, start.clone(), end.clone()) {
        MoriResult::Route { path, cost } => {
            info!(
                "found {} path cost {} from {} to {} ",
                path.len(),
                cost,
                start,
                end
            );
            surface.add_mine_path(vec![MinePath {
                cost,
                links: path,
                mine_base,
            }])?;
        }
        MoriResult::FailingDebug(stuff) => {
            todo!("pathfinding failed")
        }
    }
    info!("Mori execution {watch}");
    Ok(())
}
