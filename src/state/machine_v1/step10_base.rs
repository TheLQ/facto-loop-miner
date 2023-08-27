use crate::gamedata::lua::{EasyBox, LuaData};
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::TILES_PER_CHUNK;
use std::path::PathBuf;

pub struct Step10 {}

impl Step10 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step10 {})
    }
}

impl Step for Step10 {
    fn name(&self) -> String {
        "step01-base".to_string()
    }

    fn transformer(&self, params: StepParams) {
        // draw_mega_box(&data.area_box, surface, metrics);
    }
}

pub fn draw_mega_box(area_box: &EasyBox, img: &mut Surface, metrics: &mut Metrics) {
    let tiles: isize = 20 * TILES_PER_CHUNK as isize;
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
                    area_box.absolute_x_i32(root_x as i32),
                    area_box.absolute_y_i32(root_y as i32),
                );
                metrics.increment("loop-box");
            }
        }
    }
    metrics.increment("fff-box");
    println!("megabox?")
}
