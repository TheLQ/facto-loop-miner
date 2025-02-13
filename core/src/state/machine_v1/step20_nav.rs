use crate::navigator::mine_permutate::get_possible_routes_for_batch;
use crate::navigator::mine_selector::select_mines_and_sources;
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::pixel::Pixel;
use crate::surfacev::err::VResult;
use crate::surfacev::mine::{MineLocation, MinePath};
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
        // surface.validate();

        let select_batches = select_mines_and_sources(&mut surface)
            .into_success()
            .unwrap();
        // for mine_batch in mines {
        //     for mine in mine_batch.mines {
        //         surface.draw_square_area(&mine.area, Pixel::MineNoTouch);
        //     }
        // }
        for batch in select_batches {
            let plans = get_possible_routes_for_batch(&surface, batch);
            for plan in plans {
                // will dupe
                for route in &plan.routes {
                    surface
                        .set_pixels(Pixel::Highlighter, vec![*route.destination.point()])
                        .unwrap();
                }
            }
        }

        let base = MineLocation {
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
    mine_base: MineLocation,
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
