use crate::navigator::mori::{mori_start, write_rail, Rail, RailDirection};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::state::machine_v1::step10_base::REMOVE_RESOURCE_BASE_TILES;
use crate::surface::patch::{map_vpatch_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::sector::SectorSide;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::err::VResult;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use kiddo::distance::squared_euclidean;
use kiddo::float::neighbour::Neighbour;
use opencv::core::Point;
use tracing::{debug, info};

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

const NEAREST_COUNT: usize = 100;
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
    // TODO: Port to VSurface
    let x_start = -REMOVE_RESOURCE_BASE_TILES;
    let x_end = REMOVE_RESOURCE_BASE_TILES;
    let y_start = -REMOVE_RESOURCE_BASE_TILES;
    let y_end = REMOVE_RESOURCE_BASE_TILES;

    if 1 + 2 == 34 {
        for set_x in x_start..x_end {
            for set_y in x_start..x_end {
                surface.set_pixel(VPoint::new(set_x, set_y), Pixel::Highlighter)?;
            }
        }
        return Ok(());
    }

    // tracing::debug!("found {} patch {} away", pixel.as_ref(), patch_distance);

    let mut destinations = main_base_destinations().into_iter();

    let base_corner = base_bottom_right_corner();
    let mut made_paths: u8 = 0;

    let ordered_patches: Vec<VPatch> = ordered_patches_by_radial_base_corner(surface)
        .into_iter()
        .cloned()
        .collect();
    let ordered_size = ordered_patches.len();
    for (nearest_count, patch_start) in ordered_patches.into_iter().enumerate() {
        tracing::debug!(
            "path {} of {} - actually made {} max {:?}",
            nearest_count,
            NEAREST_COUNT,
            made_paths,
            PATH_LIMIT,
        );
        if x_start < patch_start.area.start.x()
            && x_end > patch_start.area.start.x()
            && y_start < patch_start.area.start.y()
            && y_end > patch_start.area.start.y()
        {
            tracing::debug!("[Warn] broken patch in the remove area {:?}", patch_start);
            continue;
        }

        if let Some(limit) = PATH_LIMIT {
            if limit == made_paths {
                tracing::debug!("path limit");
                break;
            }
        }
        let Some(destination) = destinations.next() else {
            tracing::debug!("Out of destinations, stopping");
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

        let start = Rail::new_straight(patch_corner, RailDirection::Left).move_forward();
        let end = Rail::new_straight(destination, RailDirection::Left);

        if 1 + 1 == 24 {
            write_rail(surface, &Vec::from([start.clone(), end.clone()]))?;
            // surface.draw_square(
            //     &Pixel::IronOre,
            //     100,
            //     &start.endpoint.to_point_u32().unwrap(),
            // );
            return Ok(());
        }

        if let Some(path) = mori_start(surface, start, end, params) {
            write_rail(surface, &path)?;
            params.metrics.borrow_mut().increment_slow("path-success")
        } else {
            params.metrics.borrow_mut().increment_slow("path-failure")
        }
        made_paths += 1;
    }
    info!("out of patches in {}", ordered_size);

    Ok(())
}

fn main_base_destinations() -> Vec<VPoint> {
    let mut res = Vec::new();

    let base_corner = base_bottom_right_corner();
    for nearest_count in 0..PATH_LIMIT.unwrap() as i32 * 2 {
        let end = base_corner.move_y(nearest_count * 20);
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
        .filter(|v| v.resource == pixel)
        .collect();
    let cloud = map_vpatch_to_kdtree(patches.clone().into_iter());

    let base_corner = base_bottom_right_corner();
    let nearest: Vec<Neighbour<f32, usize>> = cloud.nearest_n(
        &base_corner.to_slice_f32(),
        NEAREST_COUNT,
        &squared_euclidean,
    );
    debug!("found {} from {}", nearest.len(), patches.len());

    patches
        .iter()
        .enumerate()
        .filter_map(|(i, value)| {
            if nearest.iter().any(|neighbor| neighbor.item == i) {
                Some(*value)
            } else {
                None
            }
        })
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
    VPoint::new(4700, 4700)
}

#[allow(unused)]
fn base_resource_bottom_right_corner(surface: &Surface) -> Point {
    Point { x: 5300, y: 5300 }
}

fn point_to_slice_f32(point: &Point) -> [f32; 2] {
    [point.x as f32, point.y as f32]
}
