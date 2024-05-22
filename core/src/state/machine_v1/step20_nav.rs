use crate::navigator::mori::{
    mori_start, write_rail, write_rail_with_pixel, Rail, RailDirection, RAIL_STEP_SIZE_I32,
};
use crate::navigator::path_grouper::{base_bottom_right_corner, get_patches};
use crate::navigator::PathingResult;
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::state::machine_v1::step10_base::{CENTRAL_BASE_TILES, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::patch::{DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::err::VResult;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_EIGHT, SHIFT_POINT_ONE};
use crate::surfacev::vsurface::VSurface;
use opencv::core::Point;
use tracing::{debug, error, info, trace, warn};

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

    fn transformer(&self, mut params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        // let mut counter: usize = 0;
        // for item in surface.buffer {
        //     if item == Pixel::IronOre {
        //         counter = counter + 1;
        //     }
        // }
        // panic!("found {} iron", counter.to_formatted_string(&LOCALE));

        navigate_patches_to_base(&mut surface, &mut params)?;

        // for dest in main_base_destinations() {
        //     surface.draw_square(&Pixel::Stone, 20, &dest);
        // }

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

const PATH_LIMIT: Option<u8> = Some(10);
// const PATH_LIMIT: Option<u8> = None;

enum SpeculationTypes {
    CurrentEnd,
    CurrentEndAdd(u8),     // 1 and 2 after
    NearestPatchToEnd(u8), // "somehow", keep the last
}

/// Vastly improve performance utilizing free CPU cores to try other paths.
fn navigate_patches_to_base_speculation(
    surface: Surface,
    disk_patches: DiskPatch,
    params: &mut StepParams,
) -> Surface {
    surface
}

fn navigate_patches_to_base(surface: &mut VSurface, params: &mut StepParams) -> VResult<()> {
    // if 1 + 2 == 34 {
    //     let x_start = -REMOVE_RESOURCE_BASE_TILES;
    //     let x_end = REMOVE_RESOURCE_BASE_TILES;
    //     let y_start = -REMOVE_RESOURCE_BASE_TILES;
    //     let y_end = REMOVE_RESOURCE_BASE_TILES;
    //     for set_x in x_start..x_end {
    //         for set_y in x_start..x_end {
    //             surface.set_pixel(VPoint::new(set_x, set_y), Pixel::Highlighter)?;
    //         }
    //     }
    //     return Ok(());
    // }

    //     write_rail(surface, &Vec::from([start.clone(), end.clone()]))?;
    //     // surface.draw_debug_square(&start.endpoint);
    //
    // for base in main_base_destinations() {
    //     // surface.set_pixel(base, Pixel::Rail).unwrap();
    //     write_rail(
    //         surface,
    //         &Vec::from([Rail::new_straight(base, RailDirection::Left)]),
    //     )?;
    // }
    // if 1 + 1 == 2 {
    //     return Ok(());
    // }

    // let mut destinations = main_base_destinations_base_corner().into_iter();
    let mut destinations_positive = main_base_destinations_positive_side()
        .into_iter()
        .peekable();
    let mut destinations_negative = main_base_destinations_negative_side()
        .into_iter()
        .peekable();

    let base_corner = base_bottom_right_corner();
    let mut made_paths: u8 = 0;

    let ordered_patches: Vec<_> = get_patches(surface)
        .into_iter()
        .cloned()
        // .skip(10)
        .collect();

    // Wrap patches in a no touching zone, so rail doesn't drive between start and the patch
    for patch in &ordered_patches {
        let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(patch);

        let padding = 6;
        surface.draw_square(
            patch_top_left.x() + padding,
            patch_bottom_right.x() - padding,
            patch_top_left.y() + padding,
            patch_bottom_right.y() - padding,
            Pixel::SteelChest,
            Some(patch.resource),
        )
    }

    // for end in &ordered_patches {
    //     for super_x in 0..100 {
    //         for super_y in 0..100 {
    //             let hpoint = end.area.start.move_xy(super_x, super_y);
    //             if !surface.is_point_out_of_bounds(&hpoint) {
    //                 surface.set_pixel(hpoint, Pixel::Highlighter).unwrap();
    //             }
    //         }
    //     }
    // }
    // if true {
    //     info!("DUMPING {} patches", ordered_patches.len());
    //     return Ok(());
    // }

    let mut fail_counter = 0;
    let ordered_patches_len = ordered_patches.len();
    for (ordered_patch_index, patch_start) in ordered_patches.into_iter().enumerate() {
        debug!(
            "path {} of {} - actual paths created {} max {:?}",
            ordered_patch_index, ordered_patches_len, made_paths, PATH_LIMIT,
        );
        if patch_start
            .area
            .start
            .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
        {
            warn!("broken patch in the remove area {:?}", patch_start);
            continue;
        }

        if patch_start.area.start.y() < 0 {
            warn!("tmp skip below 0 patch");
            continue;
        }

        if let Some(limit) = PATH_LIMIT {
            if limit == made_paths {
                debug!("path limit");
                break;
            }
        }

        if patch_start
            .area
            .start
            .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
        {
            error!(
                "WTF? remove within {} but patch in {:?}",
                REMOVE_RESOURCE_BASE_TILES, patch_start
            );
        }

        // Search area
        // let search_area = VArea::from_arbitrary_points(
        //     &VPoint::new(CENTRAL_BASE_TILES, -REMOVE_RESOURCE_BASE_TILES),
        //     &VPoint::new(surface.get_radius() as i32, REMOVE_RESOURCE_BASE_TILES),
        // );
        let search_area = VArea::from_arbitrary_points_pair(
            &VPoint::new(CENTRAL_BASE_TILES, -surface.get_radius_i32()),
            &VPoint::new(surface.get_radius_i32(), surface.get_radius_i32()),
        );

        // let Some(destination) = destinations.next() else {
        //     debug!("Out of destinations, stopping");
        //     break;
        // };
        let destinations_iter = if patch_start.area.start.y() > 0 {
            &mut destinations_positive
        } else {
            &mut destinations_negative
        };
        let Some(destination) = destinations_iter.peek() else {
            debug!("Out of destinations, stopping");
            break;
        };

        let patch_corner = patch_start.area.start;
        // surface.draw_text(
        //     "start",
        //     Point {
        //         x: patch_corner.x as i32 + 150,
        //         y: patch_corner.y as i32 + 50,
        //     },
        // );

        // let start = Rail::new_straight(
        //     patch_corner + SHIFT_POINT_ONE - SHIFT_POINT_EIGHT,
        //     RailDirection::Right,
        // );
        // .move_forward_step();
        let patch_rail_ends = {
            let mut ends = Vec::new();
            let (patch_top_left, patch_bottom_right) = get_expanded_patch_points(&patch_start);

            ends.push(Rail::new_straight(patch_top_left, RailDirection::Right));
            ends.push(Rail::new_straight(patch_bottom_right, RailDirection::Left));

            // opposite corners
            let patch_bottom_left = VPoint::new(patch_top_left.x(), patch_bottom_right.y());
            let patch_top_right = VPoint::new(patch_bottom_right.x(), patch_top_left.y());

            ends.push(Rail::new_straight(patch_bottom_left, RailDirection::Right));
            ends.push(Rail::new_straight(patch_top_right, RailDirection::Left));

            ends.retain(|v| {
                if !search_area.contains_point(&v.endpoint) || !v.is_area_buildable(&surface) {
                    trace!("removing bad point {:?}", v);
                    false
                } else {
                    true
                }
            });

            ends
        };

        let base_start = Rail::new_straight(*destination, RailDirection::Right);

        // surface.draw_square_area(&search_area, Pixel::Highlighter, None);
        // if 1 + 1 == 2 {
        //     return Ok(());
        // }

        let mut found_path: Option<(PathingResult, &Rail)> = None;
        for threaded_end in &patch_rail_ends {
            let path_result = mori_start(
                surface,
                base_start.clone(),
                threaded_end.clone(),
                &search_area,
            );
            // .map(|path| (path, threaded_end));

            found_path = Some((path_result, threaded_end));
            if let Some((PathingResult::Route(_), _)) = &found_path {
                break;
            }

            // found_path = shinri_start_2(
            //     surface,
            //     base_start.clone(),
            //     threaded_end.clone(),
            //     &search_area,
            // )
            // .map(|v| {
            //     if v.is_empty() {
            //         warn!("empty path!");
            //         None
            //     } else {
            //         Some(v)
            //     }
            // })
            // .flatten()
            // .map(|path| (path, threaded_end));
        }
        let found_path = found_path.unwrap();

        let patch_center = patch_start.area.point_center();
        surface.draw_square_around_point(&patch_center, 20, Pixel::CrudeOil, None);

        match found_path {
            (PathingResult::Route(path), end) => {
                let last_path = path.last().unwrap().clone();
                write_rail(surface, &path)?;
                surface.add_rail(path);

                // destination no longer usable
                destinations_iter.next();
                made_paths += 1;

                params.metrics.borrow_mut().increment_slow("path-success");

                // surface.draw_square_around_point(&end.endpoint, 5, Pixel::CrudeOil, None);

                // if made_paths > 4 {
                // surface.draw_debug_square(&last_path.endpoint);
                //     surface.draw_debug_square(&patch_start.area.point_center());
                //
                //     error!("last {:?}", last_path.endpoint);
                //     surface
                //         .set_pixel(last_path.endpoint, Pixel::CopperOre)
                //         .unwrap();
                //     error!("patch {:?}", patch_start);
                //     error!("patch center {:?}", patch_start.area.point_center());
                //     surface
                //         .set_pixel(patch_start.area.point_center(), Pixel::CrudeOil)
                //         .unwrap();
                //     break;
                // }
            }
            (PathingResult::FailingDebug(path), end) => {
                params.metrics.borrow_mut().increment_slow("path-failure");

                fail_counter += 1;
                if fail_counter >= 1 {
                    write_rail(surface, &path).unwrap();

                    surface.draw_square_around_point(
                        &base_start.endpoint,
                        10,
                        Pixel::CrudeOil,
                        None,
                    );
                    write_rail_with_pixel(surface, &[end.clone()], Pixel::CrudeOil).unwrap();

                    error!("over fail");
                    break;
                }
            }
        }

        // if 1 + 1 == 2 {
        //     info!("TOO BREAK");
        //     break;
        // }

        // if nearest_count >= 2 {
        //     info!("BREAK");
        //     break;
        // }
    }
    info!("Total found patches {}", ordered_patches_len);

    Ok(())
}

fn main_base_destinations_base_corner() -> Vec<VPoint> {
    let mut res = Vec::new();

    let base_corner = base_bottom_right_corner().move_x(10);
    for nearest_count in 0..PATH_LIMIT.unwrap() as i32 * 2 {
        let end = base_corner.move_y(nearest_count * -20);
        res.push(end);
    }

    res
}

const CENTRAL_BASE_TILES_BY_RAIL_STEP: i32 = CENTRAL_BASE_TILES
    + ((RAIL_STEP_SIZE_I32 * 2) - (CENTRAL_BASE_TILES % (RAIL_STEP_SIZE_I32 * 2)));

fn main_base_destinations_positive_side() -> Vec<VPoint> {
    let mut res = Vec::new();
    for nearest_count in 1..(PATH_LIMIT.unwrap() * 2) as i32 {
        res.push(
            VPoint::new(
                CENTRAL_BASE_TILES_BY_RAIL_STEP,
                nearest_count * RAIL_STEP_SIZE_I32 * 2,
            ) + SHIFT_POINT_ONE,
        );
    }
    res
}

fn main_base_destinations_negative_side() -> Vec<VPoint> {
    let mut res = Vec::new();
    for nearest_count in 1..(PATH_LIMIT.unwrap() * 2) as i32 {
        res.push(
            VPoint::new(
                CENTRAL_BASE_TILES_BY_RAIL_STEP,
                nearest_count * -(RAIL_STEP_SIZE_I32 * 2),
            ) + SHIFT_POINT_ONE,
        );
    }
    res
}

fn find_end_simple(surface: &Surface, patch: &Patch) -> PointU32 {
    let mut current = patch.corner_point_u32();
    while surface.get_pixel_point_u32(&current) != &Pixel::EdgeWall {
        current.x -= 1
    }
    //back away
    current.x += 15;

    current
}

#[allow(unused)]
fn right_mid_edge_point(surface: &Surface) -> Point {
    Point {
        x: surface.width as i32,
        y: (surface.height / 2) as i32,
    }
}

fn get_expanded_patch_points(patch: &VPatch) -> (VPoint, VPoint) {
    // main corners
    let mut patch_top_left = patch.area.start.move_round16_down() + SHIFT_POINT_ONE;
    patch_top_left.assert_odd_16x16_position();

    let mut patch_bottom_right = patch.area.point_bottom_left().move_round16_up() + SHIFT_POINT_ONE;
    patch_bottom_right.assert_odd_16x16_position();

    for _ in 0..2 {
        patch_top_left = patch_top_left - SHIFT_POINT_EIGHT;
        patch_bottom_right = patch_bottom_right + SHIFT_POINT_EIGHT;
    }

    (patch_top_left, patch_bottom_right)
}

// fn debug_patch(surface: &mut VSurface, patch_start: &VPatch) {
//     surface.draw_debug_varea_square(&patch_start.area);
//     surface
//         .set_pixel(patch_start.area.start, Pixel::CrudeOil)
//         .unwrap();
//     surface
//         .set_pixel(patch_start.area.point_bottom_left(), Pixel::CrudeOil)
//         .unwrap();
// }

// #[allow(unused)]
// fn base_resource_bottom_right_corner(surface: &Surface) -> Point {
//     Point { x: 5300, y: 5300 }
// }
