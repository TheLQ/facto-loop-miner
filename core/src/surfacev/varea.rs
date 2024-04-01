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
            && target.x() < self.start.x() + self.width as i32
            && target.y() >= self.start.y()
            && target.y() < self.start.y() + self.height as i32
    }
}
