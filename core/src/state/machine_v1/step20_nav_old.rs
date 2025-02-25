use crate::navigator::path_executor::{execute_route_batch, MineRouteCombinationPathResult};
use crate::navigator::path_grouper::{
    base_bottom_right_corner, get_mine_bases_by_batch, MineBaseBatch, MineBaseBatchResult,
};
use crate::navigator::path_planner::{
    expanded_mine_no_touching_zone, get_possible_routes_for_batch, MineChoices,
};
use crate::navigator::path_side::BaseSource;
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::patch::{DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::opencv_re::core::Point;
use num_format::ToFormattedString;
use std::borrow::BorrowMut;
use std::sync::Mutex;
use tracing::{error, info, warn};

pub struct Step20 {}

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

        // let mut counter: usize = 0;
        // for item in surface.buffer {
        //     if item == Pixel::IronOre {
        //         counter = counter + 1;
        //     }
        // }
        // panic!("found {} iron", counter.to_formatted_string(&LOCALE));

        match 1 {
            1 => navigate_patches_to_base2(&mut surface),
            2 => navigate_patches_to_base_single(&mut surface),
            3 => navigate_patches_to_base_dump_rails(&mut surface),
            _ => panic!("adf"),
        };

        // navigate_patches_to_base(&mut surface, &mut params)?;

        // for dest in main_base_destinations() {
        //     surface.draw_square(&Pixel::Stone, 20, &dest);
        // }

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

const PATH_LIMIT: Option<u8> = Some(10);
// const PATH_LIMIT: Option<u8> = None;

fn navigate_patches_to_base_single(surface: &mut VSurface) {
    let base_source = BaseSource::new();

    info!("Loading mine bases");
    let mut mine_batches = get_mine_bases_by_batch(surface, &base_source)
        .into_success()
        .unwrap();
    info!("Loaded {} batches", mine_batches.len());

    let mine_batch = mine_batches.remove(1);
    if 1 + 1 == 2 {
        surface.draw_square_area(&mine_batch.batch_search_area, Pixel::EdgeWall, None);
        return;
    }

    // info!("removed {:?}", mine_batch);
    info!("removed 1");
    let mut route_combination_batch = get_possible_routes_for_batch(surface, mine_batch);
    info!(
        "possible batches {}",
        route_combination_batch.combinations.len()
    );
    // if 1 + 1 == 2 {
    //     trace!(
    //         "combinations ++= {}",
    //         route_combination_batch.combinations.len()
    //     );
    //     return;
    // }

    let combination = route_combination_batch.combinations.remove(1);
    for route in combination.routes {
        let res = mori_start(
            surface,
            route.base_rail,
            route.entry_rail,
            &route_combination_batch.planned_search_area,
        );
        if res.is_route() {
            info!("next!");
            continue;
        }
        match res {
            PathingResult::Route { path, cost } => {
                info!("success!");
                write_rail(surface, &path).unwrap();
            }
            PathingResult::FailingDebug(path) => {
                info!("fail!");
                write_rail(surface, &path).unwrap();
            }
        }
    }
    // combination.routes.pop();
}

fn navigate_patches_to_base_dump_rails(surface: &mut VSurface) {
    let base_source = BaseSource::new();

    info!("Loading mine bases");
    let mine_batches = get_mine_bases_by_batch(surface, &base_source)
        .into_success()
        .unwrap();
    info!("Loaded {} batches", mine_batches.len());

    draw_no_touching_zone(surface, &mine_batches);

    let mut rails: Vec<Rail> = Vec::new();
    let mut combinations = 0;
    for mine_batch in mine_batches {
        let route_combination_batch = get_possible_routes_for_batch(surface, mine_batch);
        for combination in route_combination_batch.combinations {
            for route in combination.routes {
                rails.push(route.base_rail);
                rails.push(route.entry_rail);
                combinations += 1;
            }
        }
    }
    write_rail(surface, &rails).unwrap();
    info!(
        "Would have tested {} combinations",
        combinations.to_formatted_string(&LOCALE)
    )
}

fn navigate_patches_to_base2(surface: &mut VSurface) {
    let base_source = BaseSource::new();

    info!("Loading mine bases");
    let mine_batches = get_mine_bases_by_batch(surface, &base_source);
    let mine_batches = match mine_batches {
        MineBaseBatchResult::Success { batches } => batches,
        MineBaseBatchResult::EmptyBatch { batch } => {
            error!("empty batch in area???");
            surface.draw_square_area(
                &batch.batch_search_area,
                Pixel::Highlighter,
                Some(Pixel::IronOre),
            );
            return;
        }
    };

    info!("Loaded {} batches", mine_batches.len());

    draw_no_touching_zone(surface, &mine_batches);

    // if 1 + 1 == 2 {
    //     return;
    // }
    let mut failing_count = 0;

    for (batch_index, mine_batch) in mine_batches.into_iter().skip(0).enumerate() {
        let watch = BasicWatch::start();

        // for mine in &mine_batch.mines {
        //     trace!("area {:?}", mine.area);
        //     surface.draw_square_area(&mine.area, Pixel::Highlighter, None);
        // }
        // if 1 + 1 == 2 {
        //     break;
        // }

        let batch_side = mine_batch.base_source_eighth.clone();
        info!("Processing batch with {} mines", mine_batch.mines.len());
        let route_combination_batch = get_possible_routes_for_batch(surface, mine_batch);
        info!(
            "Batch created {} combinations",
            route_combination_batch.combinations.len()
        );
        // TODO???
        // 2024-05-31T06:40:36.603091Z  INFO facto_loop_miner::state::machine_v1::step20_nav: Processing batch with 3 mines
        // 2024-05-31T06:40:36.603108Z  INFO facto_loop_miner::navigator::path_planner: Expanded 3 mines with 0 destinations to...
        // 2024-05-31T06:40:36.603111Z  INFO facto_loop_miner::navigator::path_planner: generated 0 combinations
        // 2024-05-31T06:40:36.603113Z  INFO facto_loop_miner::navigator::path_planner: generated 0 permutations
        // 2024-05-31T06:40:36.603115Z  INFO facto_loop_miner::navigator::path_planner: Expanded 3 mines with 0 destinations to 0 combinations then 0 permutated
        // 2024-05-31T06:40:36.603116Z  INFO facto_loop_miner::state::machine_v1::step20_nav: Batch created 0 combinations
        if route_combination_batch.combinations.is_empty() {
            error!("empty combinations!");
            continue;
        }

        // if 1 + 1 == 2 {
        //     break;
        // }

        let mut debug_areas = Vec::new();
        let mut debug_rails = Vec::new();
        for mine_combination in &route_combination_batch.combinations {
            for mine_route in &mine_combination.routes {
                debug_areas.push(mine_route.mine.area.clone());
                debug_rails.push(mine_route.base_rail.clone());
                debug_rails.push(mine_route.entry_rail.clone());
            }
        }

        let planned_area_clone = route_combination_batch.planned_search_area.clone();
        let res = execute_route_batch(surface, route_combination_batch);
        info!("execution took {}", watch);

        match res {
            MineRouteCombinationPathResult::Success {
                paths,
                route_combination,
            } => {
                for path in &paths {
                    info!(
                        "Writing path {:?} rail for base {:?}",
                        path.rail.len(),
                        path.mine_base
                    );
                    write_rail(surface, &path.rail).unwrap();
                }

                let mut side = Mutex::lock(&batch_side).unwrap();
                let side_before = side.pos();
                for _ in 0..route_combination.routes.len() {
                    // let weak_count = Rc::weak_count(&batch_side);
                    // let strong_count = Rc::strong_count(&batch_side);
                    // info!("weak count {} strong {}", weak_count, strong_count);
                    // Rc::get_mut(&mut batch_side).unwrap().next();
                    side.next();
                    // batch_side.get_mut().unwrap().next();
                }
                info!("upgraded side from {} to {:?}", side_before, side);

                surface.add_rail_path_drain(paths);
            }
            MineRouteCombinationPathResult::Failure { .. } => {
                let side = Mutex::lock(&batch_side).unwrap();
                // error!("side is {:?}", side);
                failing_count += 1;
                error!("Failed to find {}", failing_count);
                if failing_count == 20 {
                    error!("too many fails");
                    for area in debug_areas {
                        surface.draw_square_area(&area, Pixel::Highlighter, None);
                    }
                    write_rail_with_pixel(surface, &debug_rails, Pixel::Highlighter).unwrap();
                    break;
                }
                // surface.draw_square_area(
                //     &planned_area_clone,
                //     Pixel::Highlighter,
                //     Some(Pixel::IronOre),
                // );
            }
        }

        // if batch_index == 0 {
        //     info!("asfsdfv");
        //     break;
        // }
    }
}

fn draw_no_touching_zone<'a>(
    surface: &mut VSurface,
    mine_batches: impl IntoIterator<Item = &'a MineBaseBatch>,
) {
    // Wrap patches in a no touching zone, so rail doesn't drive between start and the patch
    for mine_batch in mine_batches {
        for mine in &mine_batch.mines {
            // let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(patch);

            // let padding = 6;
            // surface.draw_square(
            //     patch_top_left.x() + padding,
            //     patch_bottom_right.x() - padding,
            //     patch_top_left.y() + padding,
            //     patch_bottom_right.y() - padding,
            //     Pixel::SteelChest,
            //     Some(patch.resource),
            // )
            // let mine_choice = MineChoices::from_mine(surface, mine.clone());
            // let choice_area: VArea =
            //     VArea::from_arbitrary_points(mine_choice.destinations.iter().map(|v| v.endpoint));

            // get patches
            // let choice_area = VArea::from_arbitrary_points(
            //     mine.get_vpatches(surface)
            //         .into_iter()
            //         .flat_map(|patch| &patch.pixel_indexes),
            // );

            // warn!(
            //     "Destinations for {:?}\n{}",
            //     choice_area,
            //     mine_choice
            //         .destinations
            //         .iter()
            //         .map(|v| format!("{:?}", v))
            //         .join("\n")
            // );

            let mine_area = expanded_mine_no_touching_zone(mine);
            let patch_top_left = &mine_area.start;
            let patch_bottom_right = mine_area.point_bottom_left();
            surface.draw_square_area(
                &mine_area,
                Pixel::SteelChest,
                Some(surface.get_patches_slice()[mine.patch_indexes[0]].resource),
            );
            // surface
            //     .set_pixel(choice_area.start, Pixel::Highlighter)
            //     .unwrap();
            // surface
            //     .set_pixel(choice_area.point_center(), Pixel::EdgeWall)
            //     .unwrap();
        }
    }
}

// fn navigate_patches_to_base(surface: &mut VSurface, params: &mut StepParams) -> VResult<()> {
//     // if 1 + 2 == 34 {
//     //     let x_start = -REMOVE_RESOURCE_BASE_TILES;
//     //     let x_end = REMOVE_RESOURCE_BASE_TILES;
//     //     let y_start = -REMOVE_RESOURCE_BASE_TILES;
//     //     let y_end = REMOVE_RESOURCE_BASE_TILES;
//     //     for set_x in x_start..x_end {
//     //         for set_y in x_start..x_end {
//     //             surface.set_pixel(VPoint::new(set_x, set_y), Pixel::Highlighter)?;
//     //         }
//     //     }
//     //     return Ok(());
//     // }
//
//     //     write_rail(surface, &Vec::from([start.clone(), end.clone()]))?;
//     //     // surface.draw_debug_square(&start.endpoint);
//     //
//     // for base in main_base_destinations() {
//     //     // surface.set_pixel(base, Pixel::Rail).unwrap();
//     //     write_rail(
//     //         surface,
//     //         &Vec::from([Rail::new_straight(base, RailDirection::Left)]),
//     //     )?;
//     // }
//     // if 1 + 1 == 2 {
//     //     return Ok(());
//     // }
//
//     // let mut destinations = main_base_destinations_base_corner().into_iter();
//     let mut destinations_positive = main_base_destinations_positive_side()
//         .into_iter()
//         .peekable();
//     let mut destinations_negative = main_base_destinations_negative_side()
//         .into_iter()
//         .peekable();
//
//     let base_corner = base_bottom_right_corner();
//     let mut made_paths: u8 = 0;
//
//     let ordered_patches: Vec<_> = get_mine_bases_by_batch(surface)
//         .into_iter()
//         .cloned()
//         // .skip(10)
//         .collect();
//
//     // Wrap patches in a no touching zone, so rail doesn't drive between start and the patch
//     for patch in &ordered_patches {
//         let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(patch);
//
//         let padding = 6;
//         surface.draw_square(
//             patch_top_left.x() + padding,
//             patch_bottom_right.x() - padding,
//             patch_top_left.y() + padding,
//             patch_bottom_right.y() - padding,
//             Pixel::SteelChest,
//             Some(patch.resource),
//         )
//     }
//
//     // for end in &ordered_patches {
//     //     for super_x in 0..100 {
//     //         for super_y in 0..100 {
//     //             let hpoint = end.area.start.move_xy(super_x, super_y);
//     //             if !surface.is_point_out_of_bounds(&hpoint) {
//     //                 surface.set_pixel(hpoint, Pixel::Highlighter).unwrap();
//     //             }
//     //         }
//     //     }
//     // }
//     // if true {
//     //     info!("DUMPING {} patches", ordered_patches.len());
//     //     return Ok(());
//     // }
//
//     let mut fail_counter = 0;
//     let ordered_patches_len = ordered_patches.len();
//     for (ordered_patch_index, patch_start) in ordered_patches.into_iter().enumerate() {
//         debug!(
//             "path {} of {} - actual paths created {} max {:?}",
//             ordered_patch_index, ordered_patches_len, made_paths, PATH_LIMIT,
//         );
//         if patch_start
//             .area
//             .start
//             .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
//         {
//             warn!("broken patch in the remove area {:?}", patch_start);
//             continue;
//         }
//
//         if patch_start.area.start.y() < 0 {
//             warn!("tmp skip below 0 patch");
//             continue;
//         }
//
//         if let Some(limit) = PATH_LIMIT {
//             if limit == made_paths {
//                 debug!("path limit");
//                 break;
//             }
//         }
//
//         if patch_start
//             .area
//             .start
//             .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
//         {
//             error!(
//                 "WTF? remove within {} but patch in {:?}",
//                 REMOVE_RESOURCE_BASE_TILES, patch_start
//             );
//         }
//
//         // Search area
//         // let search_area = VArea::from_arbitrary_points(
//         //     &VPoint::new(CENTRAL_BASE_TILES, -REMOVE_RESOURCE_BASE_TILES),
//         //     &VPoint::new(surface.get_radius() as i32, REMOVE_RESOURCE_BASE_TILES),
//         // );
//         let search_area = VArea::from_arbitrary_points_pair(
//             &VPoint::new(CENTRAL_BASE_TILES, -surface.get_radius_i32()),
//             &VPoint::new(surface.get_radius_i32(), surface.get_radius_i32()),
//         );
//
//         // let Some(destination) = destinations.next() else {
//         //     debug!("Out of destinations, stopping");
//         //     break;
//         // };
//         let destinations_iter = if patch_start.area.start.y() > 0 {
//             &mut destinations_positive
//         } else {
//             &mut destinations_negative
//         };
//         let Some(destination) = destinations_iter.peek() else {
//             debug!("Out of destinations, stopping");
//             break;
//         };
//
//         let patch_corner = patch_start.area.start;
//         // surface.draw_text(
//         //     "start",
//         //     Point {
//         //         x: patch_corner.x as i32 + 150,
//         //         y: patch_corner.y as i32 + 50,
//         //     },
//         // );
//
//         // let start = Rail::new_straight(
//         //     patch_corner + SHIFT_POINT_ONE - SHIFT_POINT_EIGHT,
//         //     RailDirection::Right,
//         // );
//         // .move_forward_step();
//         let patch_rail_ends = {
//             let mut ends = Vec::new();
//             let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(&patch_start);
//
//             ends.push(Rail::new_straight(patch_top_left, RailDirection::Right));
//             ends.push(Rail::new_straight(patch_bottom_right, RailDirection::Left));
//
//             // opposite corners
//             let patch_bottom_left = VPoint::new(patch_top_left.x(), patch_bottom_right.y());
//             let patch_top_right = VPoint::new(patch_bottom_right.x(), patch_top_left.y());
//
//             ends.push(Rail::new_straight(patch_bottom_left, RailDirection::Right));
//             ends.push(Rail::new_straight(patch_top_right, RailDirection::Left));
//
//             ends.retain(|v| {
//                 if !search_area.contains_point(&v.endpoint) || !v.is_area_buildable(&surface) {
//                     trace!("removing bad point {:?}", v);
//                     false
//                 } else {
//                     true
//                 }
//             });
//
//             ends
//         };
//
//         let base_start = Rail::new_straight(*destination, RailDirection::Right);
//
//         // surface.draw_square_area(&search_area, Pixel::Highlighter, None);
//         // if 1 + 1 == 2 {
//         //     return Ok(());
//         // }
//
//         let mut found_path: Option<(PathingResult, &Rail)> = None;
//         for threaded_end in &patch_rail_ends {
//             let path_result = mori_start(
//                 surface,
//                 base_start.clone(),
//                 threaded_end.clone(),
//                 &search_area,
//             );
//             // .map(|path| (path, threaded_end));
//
//             found_path = Some((path_result, threaded_end));
//             if let Some((PathingResult::Route(_), _)) = &found_path {
//                 break;
//             }
//
//             // found_path = shinri_start_2(
//             //     surface,
//             //     base_start.clone(),
//             //     threaded_end.clone(),
//             //     &search_area,
//             // )
//             // .map(|v| {
//             //     if v.is_empty() {
//             //         warn!("empty path!");
//             //         None
//             //     } else {
//             //         Some(v)
//             //     }
//             // })
//             // .flatten()
//             // .map(|path| (path, threaded_end));
//         }
//         let found_path = found_path.unwrap();
//
//         let patch_center = patch_start.area.point_center();
//         surface.draw_square_around_point(&patch_center, 20, Pixel::CrudeOil, None);
//
//         match found_path {
//             (PathingResult::Route(path), end) => {
//                 let last_path = path.last().unwrap().clone();
//                 write_rail(surface, &path)?;
//                 surface.add_rail(path);
//
//                 // destination no longer usable
//                 destinations_iter.next();
//                 made_paths += 1;
//
//                 params.metrics.borrow_mut().increment_slow("path-success");
//
//                 // surface.draw_square_around_point(&end.endpoint, 5, Pixel::CrudeOil, None);
//
//                 // if made_paths > 4 {
//                 // surface.draw_debug_square(&last_path.endpoint);
//                 //     surface.draw_debug_square(&patch_start.area.point_center());
//                 //
//                 //     error!("last {:?}", last_path.endpoint);
//                 //     surface
//                 //         .set_pixel(last_path.endpoint, Pixel::CopperOre)
//                 //         .unwrap();
//                 //     error!("patch {:?}", patch_start);
//                 //     error!("patch center {:?}", patch_start.area.point_center());
//                 //     surface
//                 //         .set_pixel(patch_start.area.point_center(), Pixel::CrudeOil)
//                 //         .unwrap();
//                 //     break;
//                 // }
//             }
//             (PathingResult::FailingDebug(path), end) => {
//                 params.metrics.borrow_mut().increment_slow("path-failure");
//
//                 fail_counter += 1;
//                 if fail_counter >= 1 {
//                     write_rail(surface, &path).unwrap();
//
//                     surface.draw_square_around_point(
//                         &base_start.endpoint,
//                         10,
//                         Pixel::CrudeOil,
//                         None,
//                     );
//                     write_rail_with_pixel(surface, &[end.clone()], Pixel::CrudeOil).unwrap();
//
//                     error!("over fail");
//                     break;
//                 }
//             }
//         }
//
//         // if 1 + 1 == 2 {
//         //     info!("TOO BREAK");
//         //     break;
//         // }
//
//         // if nearest_count >= 2 {
//         //     info!("BREAK");
//         //     break;
//         // }
//     }
//     info!("Total found patches {}", ordered_patches_len);
//
//     Ok(())
// }
