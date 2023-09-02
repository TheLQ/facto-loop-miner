use crate::state::machine::{search_step_history_dirs, Step, StepParams};
use crate::surface::easybox::EasyBox;
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::TILES_PER_CHUNK;
use opencv::sys::cv_cuda_DeviceInfo_surfaceAlignment_const;

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
        let recent_surface = search_step_history_dirs(
            params.step_history_out_dirs.clone().into_iter(),
            "surface-full.png",
        );
        let mut surface = Surface::load(recent_surface.parent().unwrap());

        draw_mega_box(&mut surface, &mut params.metrics.borrow_mut());

        surface.save(&params.step_out_dir);
    }
}

pub fn draw_mega_box(img: &mut Surface, metrics: &mut Metrics) {
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
                    img.area_box.absolute_x_i32(root_x as i32),
                    img.area_box.absolute_y_i32(root_y as i32),
                );
                metrics.increment("loop-box");
            }
        }
    }
    metrics.increment("fff-box");
    println!("megabox?")
}
