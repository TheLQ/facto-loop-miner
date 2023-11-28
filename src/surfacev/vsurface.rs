use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use crate::surfacev::vpoint::VPoint;
use opencv::core::Point;
use std::path::Path;

/// An i32 based collection of Points and Pixels
///
///
pub struct VSurface {
    pixels: VEntityBuffer<VPixel>,
    entities: VEntityBuffer<VEntity>,
}

impl VSurface {
    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityBuffer::new(radius),
            entities: VEntityBuffer::new(radius),
        }
    }

    pub fn set_pixel(&mut self, start: VPoint, pixel: Pixel) -> VResult<()> {
        self.pixels.add(VPixel { start, pixel });
        Ok(())
    }

    pub fn save(&self, out_dir: &Path) {
        tracing::debug!("Saving RGB dump image to {}", out_dir.display());
        if !out_dir.is_dir() {
            panic!("dir does not exist {}", out_dir.display());
        }

        // self.save_raw(out_dir);
        // self.save_colorized(out_dir, NAME_PREFIX);
    }

    fn save_raw(&self) {}
}

pub(crate) struct VPixel {
    start: VPoint,
    pixel: Pixel,
}

impl VEntityXY for VPixel {
    fn get_xy(&self) -> Vec<VPoint> {
        todo!()
    }
}

pub(crate) struct VEntity {
    start: VPoint,
    points: Vec<VPoint>,
}

impl VEntityXY for VEntity {
    fn get_xy(&self) -> Vec<VPoint> {
        self.points.clone()
    }
}
