use crate::navigator::mori::{mori_start, write_rail, Rail, RailDirection};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::state::machine_v1::step10_base::{CENTRAL_BASE_TILES, REMOVE_RESOURCE_BASE_TILES};
use crate::surface::patch::{map_vpatch_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::sector::SectorSide;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::err::VResult;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
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

const NEAREST_COUNT: usize = 25;
const PATH_LIMIT: Option<u8> = Some(20);
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

    let mut destinations = main_base_destinations().into_iter();

    let base_corner = base_bottom_right_corner();
    let mut made_paths: u8 = 0;

    let ordered_patches: Vec<VPatch> = ordered_patches_by_radial_base_corner(surface)
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

    let ordered_size = ordered_patches.len();
    for (nearest_count, patch_start) in ordered_patches.into_iter().enumerate() {
        debug!(
            "path {} of {} - actually made {} max {:?}",
            nearest_count, NEAREST_COUNT, made_paths, PATH_LIMIT,
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
        let Some(destination) = destinations.next() else {
            debug!("Out of destinations, stopping");
            break;
        };

        let patch_corner = patch_start.area.start.move_round2_down();
        // surface.draw_text(
        //     "start",
        //     Point {
        //         x: patch_corner.x as i32 + 150,
        //         y: patch_corner.y as i32 + 50,
        //     },
        // );

        // endpoint box
        // for super_x in 0..100 {
        //     for super_y in 0..100 {
        //         surface.set_pixel(Pixel::Rail, end.x + super_x, end.y + super_y);
        //     }
        // }

        let start = Rail::new_straight(patch_corner, RailDirection::Left).move_forward_step();
        // let end = start
        //     .move_forward_step()
        //     .move_forward_step()
        //     .move_forward_step()
        //     .move_forward_step();
        let end = Rail::new_straight(destination, RailDirection::Left);

        if let Some(path) = mori_start(surface, start, end) {
            write_rail(surface, &path)?;
            // surface.draw_debug_square(&path[0].endpoint);
            params.metrics.borrow_mut().increment_slow("path-success")
        } else {
            params.metrics.borrow_mut().increment_slow("path-failure")
        }
        made_paths += 1;

        // if nearest_count >= 2 {
        //     info!("BREAK");
        //     break;
        // }
    }
    info!("out of patches in {}", ordered_size);

    Ok(())
}

fn main_base_destinations() -> Vec<VPoint> {
    let mut res = Vec::new();

    let base_corner = base_bottom_right_corner().move_x(10);
    for nearest_count in 0..PATH_LIMIT.unwrap() as i32 * 2 {
        let end = base_corner.move_y(nearest_count * -20);
        res.push(end);
    }

    res
}

fn ordered_patches_by_base_side(surface: &Surface, side: SectorSide) {}

fn ordered_patches_by_radial_base_corner(surface: &VSurface) -> Vec<&VPatch> {
    let pixel = Pixel::CopperOre;
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
        .filter(|v| v.resource == pixel)
        .collect();
    let cloud = map_vpatch_to_kdtree(patches.iter());

    let base_corner = base_bottom_right_corner();
    let nearest: Vec<NearestNeighbour<f32, usize>> =
        cloud.nearest_n::<Manhattan>(&base_corner.to_slice_f32(), NEAREST_COUNT);
    debug!("found {} from {}", nearest.len(), cloud.size());

    nearest
        .iter()
        .map(|neighbor| patches[neighbor.item])
        .collect()
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
