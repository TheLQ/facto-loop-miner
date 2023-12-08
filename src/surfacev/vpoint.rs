use crate::surfacev::err::{VError, VResult};
use opencv::core::{Point, Point2f};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::ops::{Add, Sub};

/// Core XY Point. i32 for simpler math
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct VPoint {
    x: i32,
    y: i32,
}

impl VPoint {
    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn new(x: i32, y: i32) -> Self {
        VPoint { x, y }
    }
    pub fn from_value(value: i32) -> Self {
        VPoint { x: value, y: value }
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

    pub fn to_slice_f32(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    pub fn is_within_center_radius(&self, center_radius: u32) -> bool {
        let center_radius = center_radius as i32;
        self.x > -center_radius
            && self.x < center_radius
            && self.y > -center_radius
            && self.y < center_radius
    }
}

const VPOINT_ABS_0: VPoint = VPoint { x: 0, y: 0 };
const VPOINT_ABS_1: VPoint = VPoint { x: 1, y: 1 };

impl Add for VPoint {
    type Output = VPoint;
    fn add(self, rhs: VPoint) -> Self::Output {
        VPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for VPoint {
    type Output = VPoint;
    fn sub(self, rhs: VPoint) -> Self::Output {
        VPoint {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

fn is_integer_f32(point: Point2f) -> bool {
    point.x.round() == point.x && point.y.round() == point.y
}

pub fn must_whole_number(point: Point2f) {
    let rounded = Point2f {
        x: point.x.round(),
        y: point.y.round(),
    };
    assert_eq!(rounded, point, "Point is not round {:?}", rounded);
}

pub fn must_odd_number(point: Point2f) {
    assert!(
        !(point.x as i32 % 2 == 0 || point.y as i32 % 2 == 0),
        "Point is even {:?}",
        point
    );
}

pub fn must_even_number(point: Point2f) {
    assert!(
        !(point.x as i32 % 2 == 1 || point.y as i32 % 2 == 1),
        "Point is odd {:?}",
        point
    );
}

pub fn must_half_number(point: Point2f) {
    let dec_x = point.x.floor() - point.x;
    let dec_y = point.y.floor() - point.y;
    assert!(
        !(dec_x > 0.4 && dec_x < 0.6 && dec_y > 0.4 && dec_y < 0.6),
        "Point isn't half {:?}",
        point
    );
}
