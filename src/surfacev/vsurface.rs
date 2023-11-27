use crate::surface::pixel::Pixel;
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use opencv::core::Point;

/// An i32 based collection of Points and Pixels
///
///
pub struct VSurface {
    tiles: VEntityBuffer<VTile>,
    entities: VEntityBuffer<VEntity>,
}

impl VSurface {
    pub fn new(radius: u32) -> Self {
        VSurface {
            tiles: VEntityBuffer::new(radius),
            entities: VEntityBuffer::new(radius),
        }
    }
}

pub(crate) struct VTile {
    pos: Point,
    pixel: Pixel,
}

impl VEntityXY for VTile {
    fn get_xy(&self) -> Vec<Point> {
        todo!()
    }
}

pub(crate) struct VEntity {
    start: Point,
    points: Vec<Point>,
    pos: Point,
}

impl VEntityXY for VEntity {
    fn get_xy(&self) -> Vec<Point> {
        self.points.clone()
    }
}
