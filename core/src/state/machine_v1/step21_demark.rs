use crate::navigator::mori::Rail;
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use tracing::{error, info, warn};

pub struct Step21 {}

impl Step21 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step21 {})
    }
}

impl Step for Step21 {
    fn name(&self) -> &'static str {
        "step21-demark"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        strip_rail_endpoints(&mut surface);
        strip_rail_area(&mut surface);
        strip_all_steel_chest(&mut surface);
        reapply_patch_pixels(&mut surface);

        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

fn strip_rail_endpoints(surface: &mut VSurface) {
    let mut rail_endpoints: Vec<VPoint> = surface.get_rail_TODO().map(|v| v.endpoint).collect();
    let rail_endpoints_copy = rail_endpoints.clone();
    let raw_size = rail_endpoints.len();
    rail_endpoints.sort();
    rail_endpoints.dedup();
    let after_size = rail_endpoints.len();
    if raw_size != after_size {
        error!(
            "Rail endpoints reduced from {} to {} diff {}",
            raw_size,
            after_size,
            raw_size - after_size
        );
        // seems small enough now to print
        for entry in rail_endpoints_copy {
            if !rail_endpoints.contains(&entry) {
                warn!("Removed point {:?}", entry);
            }
        }
    }
    strip_points(surface, rail_endpoints, Pixel::IronOre)
}

fn strip_rail_area(surface: &mut VSurface) {
    let mut rail_area_points: Vec<VPoint> = surface
        .get_rail_TODO()
        .flat_map(|v| v.area(surface).0)
        .collect();
    let raw_size = rail_area_points.len();
    rail_area_points.sort();
    rail_area_points.dedup();
    let after_size = rail_area_points.len();
    if raw_size != after_size {
        error!(
            "Rail Area points reduced from {} to {} diff {}",
            raw_size,
            after_size,
            raw_size - after_size
        );
    }
    strip_points(surface, rail_area_points, Pixel::Rail)
}

fn strip_all_steel_chest(surface: &mut VSurface) {
    let chests = surface
        .get_pixels_all()
        .filter(|(_, pixel)| *pixel == Pixel::SteelChest)
        .map(|(point, _)| point)
        .collect_vec();
    for point in chests {
        surface.set_pixel(point, Pixel::Empty).unwrap();
    }
}

fn reapply_patch_pixels(surface: &mut VSurface) {
    for patch in surface.get_patches_slice() {
        let mut count = 0;
        for point in &patch.pixel_indexes {
            if surface.get_pixel(point) != patch.resource {
                count += 1;
            }
        }
        if count != 0 {
            info!("changed {} for {:?}", count, patch.resource)
        }
    }
}

fn strip_points(surface: &mut VSurface, points: Vec<VPoint>, existing_expected_pixel: Pixel) {
    for point in points {
        let existing_point = surface.get_pixel(point);
        if existing_point != existing_expected_pixel {
            warn!(
                "what is this? expected {:?} got {:?} at {:?}",
                existing_expected_pixel, existing_point, point
            );
        } else {
            surface.set_pixel(point, Pixel::Empty).unwrap();
        }
    }
}
