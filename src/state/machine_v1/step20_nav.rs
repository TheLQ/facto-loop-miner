use crate::state::machine::{Step, StepParams};
use crate::surface::patch::{map_patch_corners_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use kiddo::distance::squared_euclidean;
use opencv::core::Point;

pub struct Step20 {}

impl Step20 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step20 {})
    }
}

impl Step for Step20 {
    fn name(&self) -> String {
        "step20-nav".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let mut surface = Surface::load_from_step_history(&params.step_history_out_dirs);
        let patches = DiskPatch::load_from_step_history(&params.step_history_out_dirs);

        navigator(&mut surface, patches);

        surface.save(&params.step_out_dir);
    }
}

fn navigator(surface: &mut Surface, disk_patches: DiskPatch) {
    let pixel = Pixel::IronOre;

    let patches = disk_patches.patches.get(pixel.as_ref()).unwrap();
    let cloud = map_patch_corners_to_kdtree(patches.iter().cloned());

    let needle = point_to_slice_f32(&right_mid_edge_point(surface));
    let (patch_distance, patch_index) = cloud.nearest_one(&needle, &squared_euclidean);
    println!("found {} patch {} away", pixel.as_ref(), patch_distance);

    let patch_start = &patches[patch_index];

    surface.draw_text("start", patch_start.corner_point())
}

fn route_patch(surface: &mut Surface, patch: &Patch) {
    let mut current_pixel = patch.x;
    loop {
        if let Some(existing_pixel) =
            surface.try_set_pixel(Pixel::Rail, current_pixel as u32, patch.y as u32)
        {
            println!("stopping at something");
            break;
        }

        current_pixel = current_pixel - 1;
    }
}

fn right_mid_edge_point(surface: &Surface) -> Point {
    Point {
        x: surface.width as i32,
        y: (surface.height / 2) as i32,
    }
}

fn point_to_slice_f32(point: &Point) -> [f32; 2] {
    [point.x as f32, point.y as f32]
}
