use crate::navigator::resource_cloud::point_to_slice_f32;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::patch::{map_patch_map_to_kdtree, DiskPatch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::TILES_PER_CHUNK;

pub struct Step10 {}

impl Step10 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step10 {})
    }
}

impl Step for Step10 {
    fn name(&self) -> String {
        "step10-base".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let mut surface = Surface::load_from_step_history(&params.step_history_out_dirs);
        let patches = DiskPatch::load_from_step_history(&params.step_history_out_dirs);

        draw_mega_box(&mut surface, &mut params.metrics.borrow_mut(), &patches);

        surface.save(&params.step_out_dir);
    }
}

const CENTRAL_BASE_TILES: i32 = 20;
const REMOVE_RESOURCE_BASE_TILES: i32 = 40;

pub fn draw_mega_box(img: &mut Surface, metrics: &mut Metrics, patches: &DiskPatch) {
    let tiles: i32 = CENTRAL_BASE_TILES * TILES_PER_CHUNK as i32;
    let banner_width = 50;
    let edge_neg = -tiles - banner_width;
    let edge_pos = tiles + banner_width;
    println!("edge {} to {}", edge_neg, edge_pos);
    // lazy way
    for root_x in edge_neg..edge_pos {
        for root_y in edge_neg..edge_pos {
            if !((root_x > -tiles && root_x < tiles) && (root_y > -tiles && root_y < tiles)) {
                img.set_pixel(
                    Pixel::EdgeWall,
                    img.area_box.game_centered_x_i32(root_x as i32),
                    img.area_box.game_centered_y_i32(root_y as i32),
                );
                metrics.increment("base-box");
            }
        }
    }

    draw_resource_exclude(img, metrics, patches);

    println!("megabox?")
}

fn draw_resource_exclude(img: &mut Surface, metrics: &mut Metrics, patches: &DiskPatch) {
    let patch_cloud = map_patch_map_to_kdtree(&patches.patches);

    let tiles: i32 = REMOVE_RESOURCE_BASE_TILES * TILES_PER_CHUNK as i32;
    let edge_neg: i32 = -tiles;
    // bottom right edges
    let edge_pos = tiles + 1;
    for root_x in edge_neg..edge_pos {
        for root_y in edge_neg..edge_pos {
            let point = PointU32 {
                x: img.area_box.game_centered_x_i32(root_x),
                y: img.area_box.game_centered_y_i32(root_y),
            };

            if !((root_x > -tiles && root_x < tiles) && (root_y > -tiles && root_y < tiles)) {
                let existing = img.get_pixel_point_u32(&point).clone();
                if existing.is_resource() {
                    // remove patches at the edge
                    let patches_for_resource = &patches.patches[&existing];
                    let nearby_patches = patch_cloud[&existing].within_unsorted(
                        &point_to_slice_f32(point),
                        1000000f32,
                        &kiddo::distance::squared_euclidean,
                    );

                    metrics.increment(&format!("nearby-patches-{}", nearby_patches.len()));
                    for nearby_patch in nearby_patches {
                        let patch = &patches_for_resource[nearby_patch.item];
                        let removed = patch.remove_resource_from_surface_square(&existing, img);
                        let mult = 100;
                        // img.draw_square(&Pixel::IronOre, 100, patch.corner_point_u32());
                        metrics.increment(&format!(
                            "nearby-patches-removed-{}x{}",
                            removed / mult,
                            mult
                        ));
                    }
                }

                img.set_pixel_point_u32(Pixel::EdgeWall, point);
                metrics.increment("resource-exclude-wall");
            }

            if img.get_pixel_point_u32(&point).is_resource() {
                // resource exclude
                img.set_pixel_point_u32(Pixel::Empty, point);
                metrics.increment("resource-removed");
            }
        }
    }
}
