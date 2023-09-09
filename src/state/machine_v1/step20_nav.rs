use crate::navigator::mori::{mori_start, write_rail, Rail, RailDirection};
use crate::state::machine::{Step, StepParams};
use crate::state::machine_v1::step10_base::REMOVE_RESOURCE_BASE_TILES;
use crate::surface::patch::{map_patch_corners_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use kiddo::distance::squared_euclidean;
use kiddo::float::neighbour::Neighbour;
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

    fn transformer(&self, mut params: StepParams) {
        let mut surface = Surface::load_from_step_history(&params.step_history_out_dirs);
        let patches = DiskPatch::load_from_step_history(&params.step_history_out_dirs);

        // let mut counter: usize = 0;
        // for item in surface.buffer {
        //     if item == Pixel::IronOre {
        //         counter = counter + 1;
        //     }
        // }
        // panic!("found {} iron", counter.to_formatted_string(&LOCALE));

        surface = navigate_patches_to_base(surface, patches, &mut params);

        surface.save(&params.step_out_dir);
    }
}

const NEAREST_COUNT: usize = 100;
const PATH_LIMIT: Option<u8> = Some(5);
// const PATH_LIMIT: Option<u8> = None;

fn navigate_patches_to_base(
    mut surface: Surface,
    disk_patches: DiskPatch,
    params: &mut StepParams,
) -> Surface {
    let pixel = Pixel::IronOre;

    let patches = disk_patches.patches.get(&pixel).unwrap();
    let cloud = map_patch_corners_to_kdtree(patches.iter().cloned());

    let base_corner = base_bottom_right_corner(&surface);
    let needle = point_to_slice_f32(&base_corner);
    let nearest: Vec<Neighbour<f32, usize>> =
        cloud.nearest_n(&needle, NEAREST_COUNT, &squared_euclidean);

    let x_start = surface
        .area_box
        .game_centered_x_i32(-REMOVE_RESOURCE_BASE_TILES) as i32;
    let x_end = surface
        .area_box
        .game_centered_x_i32(REMOVE_RESOURCE_BASE_TILES) as i32;
    let y_start = surface
        .area_box
        .game_centered_y_i32(-REMOVE_RESOURCE_BASE_TILES) as i32;
    let y_end = surface
        .area_box
        .game_centered_y_i32(REMOVE_RESOURCE_BASE_TILES) as i32;

    if 1 + 2 == 99 {
        for set_x in x_start..x_end {
            for set_y in x_start..x_end {
                let pos = surface.xy_to_index(set_x as u32, set_y as u32);
                surface.buffer[pos] = Pixel::Highlighter;
            }
        }
        return surface;
    }

    // println!("found {} patch {} away", pixel.as_ref(), patch_distance);

    // TODO: Speculation
    enum SpeculationTypes {
        CurrentEnd,
        CurrentEndAdd(u8),     // 1 and 2 after
        NearestPatchToEnd(u8), // "somehow", keep the last
    }

    let mut made_paths: u8 = 0;
    for (nearest_count, nearest_entry) in nearest.iter().enumerate() {
        println!("path {} of {}", nearest_count, NEAREST_COUNT);
        let patch_start = patches.get(nearest_entry.item).unwrap();

        if let Some(test) = PATH_LIMIT {
            if test == made_paths {
                println!("path limit");
                break;
            }
        }

        if x_start < patch_start.x
            && x_end > patch_start.x
            && y_start < patch_start.y
            && y_end > patch_start.y
        {
            println!("[Warn] broken patch in the remove area {:?}", patch_start);
            continue;
        }

        let patch_corner = patch_start.corner_point_u32();

        // surface.draw_text(
        //     "start",
        //     Point {
        //         x: patch_corner.x as i32 + 150,
        //         y: patch_corner.y as i32 + 50,
        //     },
        // );

        let end = PointU32 {
            x: base_corner.x as u32,
            y: base_corner.y as u32 - (nearest_count as u32 * 20),
        };

        // endpoint box
        // for super_x in 0..100 {
        //     for super_y in 0..100 {
        //         surface.set_pixel(Pixel::Rail, end.x + super_x, end.y + super_y);
        //     }
        // }

        // start line
        // route_patch(surface, patch_start);

        let start = Rail::new_straight(patch_corner, RailDirection::Left)
            .move_forward()
            .unwrap();
        let end = Rail::new_straight(end, RailDirection::Left);

        if 1 + 1 == 23 {
            write_rail(&mut surface, Vec::from([start.clone(), end.clone()]));
            // surface.draw_square(
            //     &Pixel::IronOre,
            //     100,
            //     &start.endpoint.to_point_u32().unwrap(),
            // );
            return surface;
        }

        if let Some(path) = mori_start(&surface, start, end, params) {
            write_rail(&mut surface, path);
            params.metrics.borrow_mut().increment("path-success")
        } else {
            params.metrics.borrow_mut().increment("path-failure")
        }
        made_paths = made_paths + 1;
    }

    surface
}

fn find_end_simple(surface: &Surface, patch: &Patch) -> PointU32 {
    let mut current = patch.corner_point_u32();
    while surface.get_pixel_point_u32(&current) != &Pixel::EdgeWall {
        current.x = current.x - 1
    }
    //back away
    current.x = current.x + 15;

    current.into()
}

#[allow(unused)]
fn right_mid_edge_point(surface: &Surface) -> Point {
    Point {
        x: surface.width as i32,
        y: (surface.height / 2) as i32,
    }
}

#[allow(unused)]
fn base_bottom_right_corner(surface: &Surface) -> Point {
    Point { x: 4700, y: 4700 }
}

#[allow(unused)]
fn base_resource_bottom_right_corner(surface: &Surface) -> Point {
    Point { x: 5300, y: 5300 }
}

fn point_to_slice_f32(point: &Point) -> [f32; 2] {
    [point.x as f32, point.y as f32]
}
