use crate::surface::pixel::Pixel;
use crate::surfacev::vpoint::VPoint;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VPatch {
    pub resource: Pixel,
    pub start: VPoint,
    pub width: u32,
    pub height: u32,
}

impl VPatch {
    pub fn new_from_rect(rect: Rect, resource: Pixel) -> Self {
        if !resource.is_resource() {
            panic!("not a resource {:?}", resource);
        }
        VPatch {
            resource,
            start: VPoint {
                x: rect.x,
                y: rect.y,
            },
            height: rect.height.try_into().unwrap(),
            width: rect.width.try_into().unwrap(),
        }
    }

    pub fn to_rect(&self) -> Rect {
        Rect {
            x: self.start.x,
            y: self.start.y,
            width: self.width as i32,
            height: self.height as i32,
        }
    }
}
