use crate::surfacev::vpoint::VPoint;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VArea {
    pub start: VPoint,
    pub width: u32,
    pub height: u32,
}

impl VArea {
    pub fn from_rect(rect: &Rect) -> Self {
        VArea {
            start: VPoint::new(rect.x, rect.y),
            height: rect.height.try_into().unwrap(),
            width: rect.width.try_into().unwrap(),
        }
    }

    pub fn from_arbitrary_points(a: &VPoint, b: &VPoint) -> VArea {
        let x_min = a.x().min(b.x());
        let x_max = a.x().max(b.x());
        let y_min = a.y().min(b.y());
        let y_max = a.y().max(b.y());

        let start = VPoint::new(x_min, y_min);
        VArea {
            start,
            width: (x_max - start.x()).try_into().unwrap(),
            height: (y_max - start.y()).try_into().unwrap(),
        }
    }

    pub fn to_rect(&self) -> Rect {
        Rect {
            x: self.start.x(),
            y: self.start.y(),
            width: self.width as i32,
            height: self.height as i32,
        }
    }

    pub fn contains_point(&self, target: &VPoint) -> bool {
        target.x() >= self.start.x()
            && target.x() < self.end_x_exclusive()
            && target.y() >= self.start.y()
            && target.y() < self.end_y_exclusive()
    }

    pub fn get_points(&self) -> Vec<VPoint> {
        let mut points = Vec::new();
        for point_x in self.start.x()..self.end_x_exclusive() {
            for point_y in self.start.y()..self.end_y_exclusive() {
                points.push(VPoint::new(point_x, point_y));
            }
        }
        points
    }

    pub fn end_x_exclusive(&self) -> i32 {
        self.start.x() + self.width as i32
    }

    pub fn end_y_exclusive(&self) -> i32 {
        self.start.y() + self.height as i32
    }

    pub fn point_bottom_left(&self) -> VPoint {
        VPoint::new(self.end_x_exclusive(), self.end_y_exclusive())
    }

    pub fn normalize_even_8x8(&self) -> Self {
        let x_adjust = self.start.x() % 8;
        let y_adjust = self.start.y() % 8;

        let mut new = VArea {
            start: self.start.move_xy(-x_adjust, -y_adjust),
            height: (self.height as i32 + y_adjust) as u32,
            width: (self.width as i32 + x_adjust) as u32,
        };
        new.start.assert_even_8x8_position();

        let bottom_left = new.point_bottom_left();
        let x_adjust = 8 - (bottom_left.x() % 8);
        let y_adjust = 8 - (bottom_left.y() % 8);
        new.width = (new.width as i32 + x_adjust) as u32;
        new.height = (new.height as i32 + y_adjust) as u32;

        new.point_bottom_left().assert_even_8x8_position();
        new
    }
}
