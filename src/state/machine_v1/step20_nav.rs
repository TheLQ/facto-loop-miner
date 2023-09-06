use crate::navigator::devo::{devo_start, Rail, RailDirection};
use crate::state::machine::{Step, StepParams};
use crate::surface::patch::{map_patch_corners_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
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

        // let mut counter: usize = 0;
        // for item in surface.buffer {
        //     if item == Pixel::IronOre {
        //         counter = counter + 1;
        //     }
        // }
        // panic!("found {} iron", counter.to_formatted_string(&LOCALE));

        navigate_patches_to_base(&mut surface, patches, &params);

        surface.save(&params.step_out_dir);
    }
}

fn navigate_patches_to_base(surface: &mut Surface, disk_patches: DiskPatch, params: &StepParams) {
    let pixel = Pixel::IronOre;

    let patches = disk_patches.patches.get(&pixel).unwrap();
    let cloud = map_patch_corners_to_kdtree(patches.iter().cloned());

    let needle = point_to_slice_f32(&right_mid_edge_point(surface));
    let (patch_distance, patch_index) = cloud.nearest_one(&needle, &squared_euclidean);
    println!("found {} patch {} away", pixel.as_ref(), patch_distance);

    let patch_start = &patches[patch_index];
    let patch_corner = patch_start.corner_point_u32();
    // surface.draw_text(
    //     "start",
    //     Point {
    //         x: patch_corner.x as i32 + 150,
    //         y: patch_corner.y as i32 + 50,
    //     },
    // );

    let end = match 2 {
        1 => {
            let end = find_end_simple(surface, patch_start);
            surface.set_pixel_point_u32(Pixel::Empty, end);
            end
        }
        2 => {
            let end = PointU32 { x: 150, y: 150 };
            // surface.draw_square(&Pixel::IronOre, 100, &end);
            end
        }
        _ => panic!("unexpected"),
    };

    // endpoint box
    // for super_x in 0..100 {
    //     for super_y in 0..100 {
    //         surface.set_pixel(Pixel::Rail, end.x + super_x, end.y + super_y);
    //     }
    // }

    // start line
    // route_patch(surface, patch_start);

    // let mut nav = Navigator {
    //     surface,
    //     end,
    //     current: patch_start.corner_point_u32(),
    // };
    // nav.start();

    let start = Rail::new_straight(patch_corner, RailDirection::Left)
        .move_forward()
        .unwrap();

    let end = Rail::new_straight(end, RailDirection::Left);
    // let next = start.move_forward().unwrap().move_forward().unwrap();
    // write_rail(surface, Vec::from([start.clone(), next, end.clone()]));

    devo_start(surface, start, end, params)
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

fn right_mid_edge_point(surface: &Surface) -> Point {
    Point {
        x: surface.width as i32,
        y: (surface.height / 2) as i32,
    }
}

fn point_to_slice_f32(point: &Point) -> [f32; 2] {
    [point.x as f32, point.y as f32]
}
