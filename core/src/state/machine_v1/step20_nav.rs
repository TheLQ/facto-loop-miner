use crate::navigator::mori::{
    mori_start, write_rail, Rail, RailDirection, RAIL_STEP_SIZE, RAIL_STEP_SIZE_I32,
};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::state::machine_v1::step10_base::{CENTRAL_BASE_TILES, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::patch::{map_vpatch_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::sector::SectorSide;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::err::VResult;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::{VPoint, SHIFT_POINT_ONE};
use crate::surfacev::vsurface::VSurface;
use kiddo::{Manhattan, NearestNeighbour};
use opencv::core::Point;
use tracing::{debug, info, warn};

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

const MAX_PATCHES: usize = 200;
const PATH_LIMIT: Option<u8> = Some(70);
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

    let ordered_patches: Vec<VPatch> = match 2 {
        1 => patches_by_radial_base_corner(surface, Pixel::IronOre),
        2 => patches_by_cross_sign_expanding(
            surface,
            &[Pixel::IronOre, Pixel::CopperOre, Pixel::Stone, Pixel::Coal],
        ),
        _ => panic!("asd"),
    }
    .into_iter()
    .cloned()
    .collect();
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

        if let Some(limit) = PATH_LIMIT {
            if limit == made_paths {
                debug!("path limit");
                break;
            }
        }
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

        let start = Rail::new_straight(patch_corner + SHIFT_POINT_ONE, RailDirection::Right)
            .move_forward_step();
        // let end = start
        //     .move_forward_step()
        //     .move_forward_step()
        //     .move_forward_step()
        //     .move_forward_step();
        let end = Rail::new_straight(*destination, RailDirection::Right);

        // // start box
        // for super_x in 0..100 {
        //     for super_y in 0..100 {
        //         surface
        //             .set_pixel(
        //                 VPoint::new(start.endpoint.x() + super_x, start.endpoint.y() + super_y),
        //                 Pixel::Highlighter,
        //             )
        //             .unwrap();
        //     }
        // }
        // // endpoint box
        // for super_x in 0..100 {
        //     for super_y in 0..100 {
        //         surface
        //             .set_pixel(
        //                 VPoint::new(end.endpoint.x() + super_x, end.endpoint.y() + super_y),
        //                 Pixel::Rail,
        //             )
        //             .unwrap();
        //     }
        // }
        // if 1 + 1 == 2 {
        //     info!("TOO BREAK");
        //     break;
        // }

        // Search area
        // let search_area = VArea::from_arbitrary_points(
        //     &VPoint::new(CENTRAL_BASE_TILES, -REMOVE_RESOURCE_BASE_TILES),
        //     &VPoint::new(surface.get_radius() as i32, REMOVE_RESOURCE_BASE_TILES),
        // );
        let search_area = VArea::from_arbitrary_points(
            &VPoint::new(CENTRAL_BASE_TILES, -surface.get_radius_i32()),
            &VPoint::new(surface.get_radius_i32(), surface.get_radius_i32()),
        );

        // if 1 + 1 == 2 {
        //     let radius = surface.get_radius() as i32;
        //     for x in -radius..radius {
        //         for y in -radius..radius {
        //             let point = VPoint::new(x, y);
        //             if search_area.contains_point(&point) {
        //                 surface.set_pixel(point, Pixel::Highlighter).unwrap();
        //             }
        //         }
        //     }
        //     break;
        // }

        if let Some(path) = mori_start(surface, end, start, search_area) {
            write_rail(surface, &path)?;
            surface.add_rail(path);

            // destination no longer usable
            destinations_iter.next();
            made_paths += 1;

            // surface.draw_debug_square(&path[0].endpoint);
            params.metrics.borrow_mut().increment_slow("path-success")
        } else {
            params.metrics.borrow_mut().increment_slow("path-failure")
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

const CENTRAL_BASE_TILES_BY_RAIL_STEP: i32 =
    CENTRAL_BASE_TILES + (RAIL_STEP_SIZE_I32 - (CENTRAL_BASE_TILES % RAIL_STEP_SIZE_I32));

fn main_base_destinations_positive_side() -> Vec<VPoint> {
    let mut res = Vec::new();
    for nearest_count in 1..PATH_LIMIT.unwrap() as i32 {
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
    for nearest_count in 1..PATH_LIMIT.unwrap() as i32 {
        res.push(
            VPoint::new(
                CENTRAL_BASE_TILES_BY_RAIL_STEP,
                nearest_count * -(RAIL_STEP_SIZE_I32 * 2),
            ) + SHIFT_POINT_ONE,
        );
    }
    res
}

fn ordered_patches_by_base_side(surface: &Surface, side: SectorSide) {}

fn patches_by_radial_base_corner(surface: &VSurface, resource: Pixel) -> Vec<&VPatch> {
    let patches: Vec<&VPatch> = surface
        .get_patches_slice()
        .iter()
        // remove inner base patches
        .filter(|p| {
            !p.area
                .start
                .is_within_center_radius(REMOVE_RESOURCE_BASE_TILES as u32)
        })
        // temporary left of box only
        .filter(|p| {
            (-REMOVE_RESOURCE_BASE_TILES..REMOVE_RESOURCE_BASE_TILES).contains(&p.area.start.y())
                && p.area.start.x() > REMOVE_RESOURCE_BASE_TILES
        })
        .filter(|v| v.resource == resource)
        .collect();
    let cloud = map_vpatch_to_kdtree(patches.iter());

    let base_corner = base_bottom_right_corner();
    let nearest: Vec<NearestNeighbour<f32, usize>> =
        cloud.nearest_n::<Manhattan>(&base_corner.to_slice_f32(), MAX_PATCHES);
    debug!("found {} from {}", nearest.len(), cloud.size());

    nearest
        .iter()
        .map(|neighbor| patches[neighbor.item])
        .collect()
}

#[allow(clippy::never_loop)]
fn patches_by_cross_sign_expanding<'a>(
    surface: &'a VSurface,
    resources: &[Pixel],
) -> Vec<&'a VPatch> {
    let cross_sides = [Rail::new_straight(
        VPoint::new(REMOVE_RESOURCE_BASE_TILES, 0),
        RailDirection::Right,
    )];
    let mut patches = Vec::new();
    for cross_side in cross_sides {
        for perpendicular_scan_area in (1i32..).flat_map(|i| [i, -i]) {
            if perpendicular_scan_area.unsigned_abs() * RAIL_STEP_SIZE > surface.get_radius() {
                break;
            }

            let mut parallel_scan_area = 0;
            loop {
                parallel_scan_area += 1;

                // let scan_start = {
                //     let mut rail = cross_side.clone();
                //     for _ in 0..(parallel_scan_area - 1) {
                //         rail = rail.move_forward_step();
                //     }
                //     rail
                // };

                let scan_start = {
                    let mut rail = cross_side.move_forward_step_num(parallel_scan_area - 1);
                    // trace!("start moving forward {}", parallel_scan_area - 1);
                    rail = if perpendicular_scan_area > 0 {
                        rail.move_force_rotate_clockwise(1)
                    } else {
                        rail.move_force_rotate_clockwise(3)
                    };
                    rail = rail.move_forward_step_num(perpendicular_scan_area.unsigned_abs() - 1);
                    // trace!(
                    //     "start moving side {}",
                    //     perpendicular_scan_area.unsigned_abs() - 1
                    // );
                    rail
                };

                let scan_end = {
                    let mut rail = cross_side.move_forward_step_num(parallel_scan_area);
                    // trace!("end moving forward {}", parallel_scan_area);
                    rail = if perpendicular_scan_area > 0 {
                        rail.move_force_rotate_clockwise(1)
                    } else {
                        rail.move_force_rotate_clockwise(3)
                    };
                    rail = rail.move_forward_step_num(perpendicular_scan_area.unsigned_abs());
                    // trace!("end moving side {}", perpendicular_scan_area.unsigned_abs());
                    rail
                };

                if !scan_end
                    .endpoint
                    .is_within_center_radius(surface.get_radius())
                {
                    break;
                }

                let search_area =
                    VArea::from_arbitrary_points(&scan_start.endpoint, &scan_end.endpoint);
                for patch in surface.get_patches_slice() {
                    if resources.contains(&patch.resource)
                        && search_area.contains_point(&patch.area.start)
                    {
                        patches.push(patch);
                    }
                }
            }
        }
    }
    patches
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

fn base_bottom_right_corner() -> VPoint {
    VPoint::new(CENTRAL_BASE_TILES, CENTRAL_BASE_TILES)
}

// #[allow(unused)]
// fn base_resource_bottom_right_corner(surface: &Surface) -> Point {
//     Point { x: 5300, y: 5300 }
// }
