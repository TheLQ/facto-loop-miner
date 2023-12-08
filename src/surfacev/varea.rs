use crate::surfacev::vpoint::VPoint;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VArea {
    pub start: VPoint,
    pub width: u32,
    pub height: u32,
}

impl VArea {
    pub fn new_from_rect(rect: &Rect) -> Self {
        VArea {
            start: VPoint::new(rect.x, rect.y),
            height: rect.height.try_into().unwrap(),
            width: rect.width.try_into().unwrap(),
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
}
