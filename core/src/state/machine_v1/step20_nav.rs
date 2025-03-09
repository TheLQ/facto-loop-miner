use crate::navigator::mine_executor::{
    execute_route_batch, FailingMeta, MineRouteCombinationPathResult,
};
use crate::navigator::mine_permutate::{get_possible_routes_for_batch, PlannedBatch, PlannedRoute};
use crate::navigator::mine_selector::{select_mines_and_sources, MineSelectBatch};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MinePath;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, trace};

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

        let select_batches = select_mines_and_sources(&surface).into_success().unwrap();

        let mut num_mines_metrics = Metrics::new("mine_batch_size");
        for batch in &select_batches {
            let total = batch.mines.len();
            num_mines_metrics.increment(format!("{total}"));
        }
        num_mines_metrics.log_final();

        draw_no_touching_zone(&mut surface, &select_batches);

        // if 1 + 1 == 2 {
        //     debug_draw_base_sources(&mut surface, &select_batches);
        //
        //     let mut plans = Vec::new();
        //     for batch in select_batches {
        //         plans.append(&mut get_possible_routes_for_batch(&surface, batch));
        //     }
        //     debug_draw_planned_destinations(&mut surface, plans);
        //     surface.save(&params.step_out_dir)?;
        //     return Ok(());
        // }

        for (batch_index, batch) in select_batches.into_iter().enumerate() {
            let found = process_batch(&mut surface, batch, batch_index, &params.step_out_dir);
            if !found {
                error!("KILLING EARLY");
                break;
            }

            // if 1 + 1 == 2 {
            if batch_index > 10 {
                break;
            }
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
//     surface: &mut VSurface,
//     start: VPointDirectionQ,
//     end: VPointDirectionQ,
//     mine_base: MineLocation,
// ) -> VResult<()> {
//     let watch = BasicWatch::start();
//     match mori2_start(surface, start.clone(), end.clone()) {
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

fn draw_no_touching_zone(surface: &mut VSurface, batches: &[MineSelectBatch]) {
    for batch in batches {
        for mine in &batch.mines {
            surface.draw_square_area(&mine.area, Pixel::MineNoTouch);
        }
    }

    // stop routes going backwards right behind the start
    let radius = surface.get_radius_i32();
    let anti_backside_x = batches[0].base_sources[0].point().x() - 1;
    let anti_backside_points = (-(radius - 1)..radius)
        .map(|i| VPoint::new(anti_backside_x, i))
        .collect_vec();
    surface
        .set_pixels(Pixel::MineNoTouch, anti_backside_points)
        .unwrap()
}

fn debug_draw_base_sources(
    surface: &mut VSurface,
    batches: impl IntoIterator<Item = impl Borrow<MineSelectBatch>>,
) {
    let mut pixels = Vec::new();
    for batch in batches {
        let batch = batch.borrow();
        for source in &batch.base_sources {
            pixels.push(*source.point());
        }
    }
    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
}

fn debug_draw_planned_destinations(
    surface: &mut VSurface,
    plans: impl IntoIterator<Item = impl Borrow<PlannedBatch>>,
) {
    let mut pixels = Vec::new();
    for plan in plans {
        let plan = plan.borrow();
        // will dupe
        for route in &plan.routes {
            // pixels.push(*route.base_source.point());
            pixels.push(*route.destination.point());
        }
    }

    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
}

fn process_batch(
    surface: &mut VSurface,
    batch: MineSelectBatch,
    batch_index: usize,
    step_out_dir: &Path,
) -> bool {
    trace!("---");
    let num_mines = batch.mines.len();

    let mut planned_combinations = get_possible_routes_for_batch(surface, batch);

    let num_per_batch_routes_min = planned_combinations
        .iter()
        .map(|v| v.routes.len())
        .min()
        .unwrap();
    let num_per_batch_routes_max = planned_combinations
        .iter()
        .map(|v| v.routes.len())
        .max()
        .unwrap();
    let num_routes_total: usize = planned_combinations.iter().map(|v| v.routes.len()).sum();
    let num_batches = planned_combinations.len();
    info!(
        "batch #{batch_index} with {num_mines} mines created {num_batches} combinations \
                with total routes {num_routes_total} \
                each in range {num_per_batch_routes_min} {num_per_batch_routes_max}"
    );

    let planned_combinations = vec![planned_combinations.remove(0)];
    let res = execute_route_batch(surface, planned_combinations);
    match res {
        MineRouteCombinationPathResult::Success { paths } => {
            info!("pushing {} new mine paths", paths.len());
            assert!(!paths.is_empty(), "Success but no paths!!!!");
            for path in paths {
                surface.add_mine_path(path).unwrap();
            }
            true
        }
        MineRouteCombinationPathResult::Failure {
            meta:
                FailingMeta {
                    found_paths,
                    failing_routes,
                    failing_dump,
                    failing_all,
                },
        } => {
            error!("failed to pathfind");
            for path in found_paths {
                surface
                    .add_mine_path_with_pixel(path, Pixel::Highlighter)
                    .unwrap();
            }

            let (trigger_mine, rest) = failing_routes.split_first().unwrap();
            surface.draw_square_area_replacing(
                &trigger_mine.location.area,
                Pixel::MineNoTouch,
                Pixel::Highlighter,
            );
            for entry in rest {
                surface.draw_square_area_replacing(
                    &entry.location.area,
                    Pixel::MineNoTouch,
                    Pixel::EdgeWall,
                );
            }

            // // very busy dump
            // let mut pixels = Vec::new();
            // for dump_link in failing_dump {
            //     pixels.extend(dump_link.area());
            // }
            // surface.set_pixels(Pixel::Water, pixels).unwrap();
            // // then the targets
            // let mut pixels = Vec::new();
            // for locations in failing_routes {
            //     pixels.push(*locations.base_source.point());
            //     pixels.push(*locations.destination.point());
            // }
            // surface.set_pixels(Pixel::SteelChest, pixels).unwrap();

            // let mut all_pos = failing_all.iter().map(|v| v.next_straight_position()).collect_vec();
            if failing_all.is_empty() {
                warn!("gradient dump disabled");
            } else {
                trace!("dumping {}", failing_all.len());
                let mut compressed: HashMap<VPoint, usize> = HashMap::new();
                for each in failing_all {
                    let val = compressed.entry(each.pos_next()).or_default();
                    *val += 1;
                }
                surface
                    .save_pixel_img_colorized_grad(step_out_dir, compressed)
                    .unwrap();
            }

            false
        }
    }
}
