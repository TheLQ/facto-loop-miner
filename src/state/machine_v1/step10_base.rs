use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
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

        draw_mega_box(&mut surface, &mut params.metrics.borrow_mut());

        surface.save(&params.step_out_dir);
    }
}

const CENTRAL_BASE_TILES: isize = 20;
const REMOVE_RESOURCE_BASE_TILES: isize = 40;

pub fn draw_mega_box(img: &mut Surface, metrics: &mut Metrics) {
    let tiles: isize = CENTRAL_BASE_TILES * TILES_PER_CHUNK as isize;
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
                    img.area_box.absolute_x_u32(root_x as i32),
                    img.area_box.absolute_y_u32(root_y as i32),
                );
                metrics.increment("loop-box");
            }
        }
    }
    metrics.increment("fff-box");

    draw_resource_exclude(img, metrics);

    println!("megabox?")
}

fn draw_resource_exclude(img: &mut Surface, metrics: &mut Metrics) {
    let tiles: isize = REMOVE_RESOURCE_BASE_TILES * TILES_PER_CHUNK as isize;
    let edge_neg = -tiles;
    // bottom right edges
    let edge_pos = tiles + 1;
    for root_x in edge_neg..edge_pos {
        for root_y in edge_neg..edge_pos {
            let point = PointU32 {
                x: img.area_box.absolute_x_u32(root_x as i32),
                y: img.area_box.absolute_y_u32(root_y as i32),
            };

            if !((root_x > -tiles && root_x < tiles) && (root_y > -tiles && root_y < tiles)) {
                img.set_pixel_point_u32(Pixel::EdgeWall, point);
                metrics.increment("loop-box-2");
            }

            // remove resources
            match img.get_pixel_point_u32(point) {
                Pixel::IronOre
                | Pixel::CopperOre
                | Pixel::Stone
                | Pixel::CrudeOil
                | Pixel::Coal
                | Pixel::UraniumOre => img.set_pixel_point_u32(Pixel::Empty, point),
                _ => {}
            }
        }
    }
}
