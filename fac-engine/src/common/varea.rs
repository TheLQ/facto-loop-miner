use crate::common::vpoint::VPoint;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn from_arbitrary_points_pair(a: impl Borrow<VPoint>, b: impl Borrow<VPoint>) -> VArea {
        let a = a.borrow();
        let b = b.borrow();

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

    pub fn from_arbitrary_points(points: impl IntoIterator<Item = impl Borrow<VPoint>>) -> VArea {
        let mut x_min = i32::MAX;
        let mut x_max = i32::MIN;
        let mut y_min = i32::MAX;
        let mut y_max = i32::MIN;
        for point in points {
            let borrowed_point = point.borrow();
            x_min = x_min.min(borrowed_point.x());
            x_max = x_max.max(borrowed_point.x());
            y_min = y_min.min(borrowed_point.y());
            y_max = y_max.max(borrowed_point.y());
        }

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
            && target.x() <= self.end_x_exclusive()
            && target.y() >= self.start.y()
            && target.y() <= self.end_y_exclusive()
    }

    pub fn get_points(&self) -> Vec<VPoint> {
        let mut points = Vec::new();
        for point_x in self.start.x()..=self.end_x_exclusive() {
            for point_y in self.start.y()..=self.end_y_exclusive() {
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

    pub fn desugar(&self) -> (i32, i32, i32, i32) {
        (
            self.start.x(),
            self.end_x_exclusive(),
            self.start.y(),
            self.end_y_exclusive(),
        )
    }

    pub fn point_bottom_left(&self) -> VPoint {
        VPoint::new(self.end_x_exclusive(), self.end_y_exclusive())
    }

    pub fn get_corner_points(&self) -> [VPoint; 2] {
        [self.start, self.point_bottom_left()]
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

    pub fn point_center(&self) -> VPoint {
        VPoint::new(
            // (self.end_x_exclusive() - self.start.x()) / 2,
            // (self.end_y_exclusive() - self.start.y()) / 2,
            self.start.x() + (self.width as i32 / 2),
            self.start.y() + (self.height as i32 / 2),
        )
    }
}
