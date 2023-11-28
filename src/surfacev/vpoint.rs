use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vsurface::VPixel;
use opencv::core::Point2f;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct VPoint {
    pub x: i32,
    pub y: i32,
}

impl VPoint {
    pub fn from_f32_with_offset(point: Point2f, offset: f32) -> VResult<Self> {
        let new_point = Point2f {
            x: point.x - offset,
            y: point.y - offset,
        };
        if is_integer_f32(new_point) {
            Ok(VPoint {
                x: new_point.x as i32,
                y: new_point.y as i32,
            })
        } else {
            Err(VError::XYNotInteger { position: point })
        }
    }
}

fn is_integer_f32(point: Point2f) -> bool {
    point.x.round() == point.x && point.y.round() == point.y
}
