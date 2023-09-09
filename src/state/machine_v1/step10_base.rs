use crate::navigator::resource_cloud::point_to_slice_f32;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::patch::{map_patch_map_to_kdtree, DiskPatch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::TILES_PER_CHUNK;
use opencv::core::Point;
use std::collections::HashMap;
use std::ops::{Add, Mul};

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
        let mut patches = DiskPatch::load_from_step_history(&params.step_history_out_dirs);

        draw_mega_box(&mut surface, &mut params.metrics.borrow_mut(), &mut patches);

        surface.save(&params.step_out_dir);
        patches.save(&params.step_out_dir);
    }
}

pub const CENTRAL_BASE_CHUNKS: i32 = 20;
pub const REMOVE_RESOURCE_BASE_CHUNKS: i32 = 40;
pub const REMOVE_RESOURCE_BORDER_CHUNKS: i32 = 2;

pub const CENTRAL_BASE_TILES: i32 = CENTRAL_BASE_CHUNKS * TILES_PER_CHUNK as i32;
pub const REMOVE_RESOURCE_BASE_TILES: i32 = REMOVE_RESOURCE_BASE_CHUNKS * TILES_PER_CHUNK as i32;
pub const REMOVE_RESOURCE_BORDER_TILES: i32 =
    REMOVE_RESOURCE_BORDER_CHUNKS * TILES_PER_CHUNK as i32;

pub fn draw_mega_box(img: &mut Surface, metrics: &mut Metrics, patches: &mut DiskPatch) {
    let tiles: i32 = CENTRAL_BASE_TILES;
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

fn draw_resource_exclude(img: &mut Surface, metrics: &mut Metrics, disk_patches: &mut DiskPatch) {
    let patch_cloud = map_patch_map_to_kdtree(&disk_patches.patches);
    let mut patches_to_remove: HashMap<Pixel, Vec<Point>> = HashMap::new();

    let border = REMOVE_RESOURCE_BORDER_CHUNKS * TILES_PER_CHUNK as i32;

    let tiles: i32 = REMOVE_RESOURCE_BASE_TILES;
    let edge_neg: i32 = -tiles;
    // bottom right edges
    let edge_pos = tiles + 1;
    for root_x in edge_neg..edge_pos {
        for root_y in edge_neg..edge_pos {
            let point = PointU32 {
                x: img.area_box.game_centered_x_i32(root_x),
                y: img.area_box.game_centered_y_i32(root_y),
            };

            let existing = img.get_pixel_point_u32(&point).clone();
            if !((root_x > -tiles && root_x < tiles) && (root_y > -tiles && root_y < tiles)) {
                // if (root_x == -tiles + border || root_x == tiles - border)
                //     && (root_y == -tiles + border || root_y == tiles - border)
                if existing.is_resource() {
                    // remove patches at the edge
                    let patches_for_resource = disk_patches
                        .patches
                        .get(&existing)
                        .expect(format!("match not found {:?}", existing).as_str());
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
                        // img.draw_square(&Pixel::IronOre, 100, &patch.corner_point_u32());
                        metrics.increment(&format!(
                            "nearby-patches-removed-{}{}",
                            removed / mult,
                            &mult.to_string()[1..]
                        ));

                        patches_to_remove
                            .entry(existing.clone())
                            .or_default()
                            .push(patch.corner_point_i32());
                    }
                }
                // img.set_pixel_point_u32(Pixel::EdgeWall, point);
                // metrics.increment("resource-exclude-wall");
            } else if existing.is_resource() {
                // resource exclude
                img.set_pixel_point_u32(Pixel::Empty, point);
                metrics.increment("resource-removed");

                patches_to_remove.entry(existing).or_default().push(Point {
                    x: point.x as i32,
                    y: point.y as i32,
                });
            }
        }
    }

    for (resource, mut patches) in patches_to_remove.into_iter() {
        patches.sort_by(|a, b| {
            calculate_index(img.width as i32, a.x, a.y).cmp(&calculate_index(
                img.width as i32,
                b.x,
                b.y,
            ))
        });
        patches.reverse();
        for patch in patches {
            let vec = disk_patches.patches.get_mut(&resource).unwrap();
            let before_len = vec.len();
            vec.retain(|v| !(v.x == patch.x && v.y == patch.y));
            let after_len = vec.len();

            for _ in 0..(before_len - after_len) {
                metrics.increment("detected-patches-removed");
            }
        }
    }
}

fn calculate_index<N>(width: N, x: N, y: N) -> N
where
    N: Mul<Output = N> + Add<Output = N>,
{
    width * y + x
}
