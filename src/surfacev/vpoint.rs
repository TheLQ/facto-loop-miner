use crate::surfacev::err::{VError, VResult};
use opencv::core::{Point, Point2f};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;

/// Core XY Point. i32 for simpler math
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct VPoint {
    pub x: i32,
    pub y: i32,
}

impl VPoint {
    pub fn new(x: i32, y: i32) -> Self {
        VPoint { x, y }
    }
    pub fn center() -> Self {
        VPoint { x: 0, y: 0 }
    }

    /// Factorio import. Offset is half the entity width
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
            Err(VError::XYNotInteger {
                position: point,
                backtrace: Backtrace::capture(),
            })
        }
    }

    /// Factorio export. Offset is half the entity width
    pub fn to_f32_with_offset(&self, offset: f32) -> Point2f {
        Point2f {
            x: self.x as f32 + offset,
            y: self.y as f32 + offset,
        }
    }

    pub fn to_cv_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn is_within_center_area(&self, center_radius: u32) -> bool {
        let center_radius = center_radius as i32;
        (self.x > -center_radius && self.x < center_radius)
            && (self.y > -center_radius && self.y < center_radius)
    }
}

fn is_integer_f32(point: Point2f) -> bool {
    point.x.round() == point.x && point.y.round() == point.y
}
