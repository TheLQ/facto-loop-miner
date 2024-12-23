use crate::blueprint::bpfac::position::FacBpPosition;
use crate::err::{FError, FResult};
use crate::game_entities::direction::FacDirectionQuarter;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Core XY Point. Entity origin is top left, not Factorio's center
#[derive(
    Debug, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Hash, Ord,
)]
pub struct VPoint {
    x: i32,
    y: i32,
}

pub const VPOINT_ZERO: VPoint = VPoint { x: 0, y: 0 };
pub const VPOINT_ONE: VPoint = VPoint { x: 1, y: 1 };
pub const VPOINT_EIGHT: VPoint = VPoint { x: 8, y: 8 };

pub const C_BLOCK_LINE: char = '\u{1FB72}';

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

    pub fn new_usize(x: usize, y: usize) -> Self {
        VPoint {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        }
    }

    pub fn zero() -> Self {
        VPOINT_ZERO
    }

    // pub fn from_cv_point(point: Point) -> Self {
    //     VPoint {
    //         x: point.x,
    //         y: point.y,
    //     }
    // }

    pub fn from_value(value: i32) -> Self {
        VPoint { x: value, y: value }
    }

    /// Factorio import. Offset is half the entity width
    pub fn from_f32_with_offset(point: FacBpPosition, offset: f32) -> FResult<Self> {
        let new_point = FacBpPosition {
            x: point.x - offset,
            y: point.y - offset,
        };
        if is_integer_f32(&new_point) {
            Ok(VPoint {
                x: new_point.x as i32,
                y: new_point.y as i32,
            })
        } else {
            Err(FError::XYNotInteger {
                position: point,
                backtrace: Backtrace::capture(),
            })
        }
    }

    /// Factorio export. Offset is half the entity width
    pub fn to_fac_with_offset(&self, offset: f32) -> FacBpPosition {
        FacBpPosition::new(self.x as f32 + offset, self.y as f32 + offset)
    }

    // pub fn to_cv_point(&self) -> Point {
    //     Point {
    //         x: self.x,
    //         y: self.y,
    //     }
    // }

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

    pub fn assert_even_position(&self) {
        assert_eq!(self.x % 2, 0, "x={} is not even", self.x);
        assert_eq!(self.y % 2, 0, "y={} is not even", self.y);
    }

    pub fn assert_even_8x8_position(&self) {
        self.assert_even_position();
        assert_eq!(self.x % 8, 0, "x={} is not 8", self.x);
        assert_eq!(self.y % 8, 0, "y={} is not 8", self.y);
    }

    pub fn assert_odd_position(&self) {
        assert_eq!((self.x - 1) % 2, 0, "x={} is not odd", self.x);
        assert_eq!((self.y - 1) % 2, 0, "y={} is not odd", self.y);
    }

    pub fn assert_odd_8x8_position(&self) {
        self.assert_odd_position();
        assert_eq!((self.x - 1) % 8, 0, "x={} is not 8", self.x);
        assert_eq!((self.y - 1) % 8, 0, "y={} is not 8", self.y);
    }

    pub fn assert_odd_16x16_position(&self) {
        self.assert_odd_position();
        assert_eq!((self.x - 1) % 16, 0, "x={} is not 16", self.x);
        assert_eq!((self.y - 1) % 16, 0, "y={} is not 16", self.y);
    }

    // pub fn is_odd_16x16_position(&self) -> bool {
    //     return (self.x - 1) % 16 == 0 && (self.y - 1) % 16 == 0;
    // }
    // pub fn is_odd_16x16_for_x(&self) -> bool {
    //     (self.x - 1) % 16 == 0
    // }
    //
    // pub fn is_odd_16x16_for_y(&self) -> bool {
    //     (self.y - 1) % 16 == 0
    // }

    pub fn move_x(&self, steps: i32) -> Self {
        VPoint {
            x: self.x + steps,
            y: self.y,
        }
    }

    pub fn move_x_usize(&self, steps: usize) -> Self {
        self.move_x(steps as i32)
    }

    pub fn move_y(&self, steps: i32) -> Self {
        VPoint {
            x: self.x,
            y: self.y + steps,
        }
    }

    pub fn move_y_usize(&self, steps: usize) -> Self {
        self.move_y(steps as i32)
    }

    pub fn move_xy(&self, x_steps: i32, y_steps: i32) -> Self {
        VPoint {
            x: self.x + x_steps,
            y: self.y + y_steps,
        }
    }

    pub fn move_xy_usize(&self, x_steps: usize, y_steps: usize) -> Self {
        self.move_xy(x_steps as i32, y_steps as i32)
    }

    pub fn move_direction(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: usize,
    ) -> Self {
        // cardinal directions are "north == up == -1" not "north == +1"
        match direction.borrow() {
            FacDirectionQuarter::North => self.move_y(-(steps as i32)),
            FacDirectionQuarter::South => self.move_y_usize(steps),
            FacDirectionQuarter::East => self.move_x_usize(steps),
            FacDirectionQuarter::West => self.move_x(-(steps as i32)),
        }
    }

    pub fn move_direction_sideways(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: i32,
    ) -> Self {
        // cardinal directions are "north == up == -1" not "north == +1"
        match direction.borrow() {
            FacDirectionQuarter::North => self.move_x(steps),
            FacDirectionQuarter::South => self.move_x(-steps),
            FacDirectionQuarter::East => self.move_y(steps),
            FacDirectionQuarter::West => self.move_y(-steps),
        }
    }

    // pub fn move_xy_u32(&self, x_steps: u32, y_steps: u32) -> Self {
    //     self.move_xy(x_steps as i32, y_steps as i32)
    // }

    // fn move_round2_down(&self) -> Self {
    //     self.move_round_down(2)
    // }

    // fn move_round3_down(&self) -> Self {
    //     self.move_round_down(3)
    // }

    pub fn move_round16_down(&self) -> Self {
        self.move_round_down(16)
    }

    fn move_round_down(&self, size: i32) -> Self {
        VPoint {
            x: self.x - (self.x % size),
            y: self.y - (self.y % size),
        }
    }

    pub fn move_round16_up(&self) -> Self {
        self.move_round_up(16)
    }

    fn move_round_up(&self, size: i32) -> Self {
        let x_rem = self.x % size;
        let y_rem = self.y % size;
        VPoint {
            x: if x_rem != 0 {
                self.x + (size - x_rem)
            } else {
                self.x
            },
            y: if y_rem != 0 {
                self.y + (size - y_rem)
            } else {
                self.y
            },
        }
    }

    pub fn get_entity_area_2x2(&self) -> [Self; 4] {
        [
            *self,
            self.move_x(1),
            self.move_y(1),
            self.move_x(1).move_y(1),
        ]
    }
    pub fn get_entity_area_3x3(&self) -> [Self; 9] {
        [
            *self,
            self.move_x(1),
            self.move_y(1),
            self.move_x(1).move_y(1),
            //
            self.move_x(2),
            self.move_x(2).move_y(1),
            self.move_y(2),
            self.move_y(2).move_x(1),
            //
            self.move_y(2).move_x(2),
        ]
    }

    // aka Manhattan distance
    pub fn distance_to(&self, other: &Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    // aka hypotenuse I guess?
    pub fn distance_bird(&self, other: &Self) -> f32 {
        let squared_euclidean = self.x.abs_diff(other.x).pow(2) + self.y.abs_diff(other.y).pow(2);
        (squared_euclidean as f32).sqrt()
    }

    pub fn subtract_x(&self, other: &Self) -> i32 {
        self.x - other.x
    }

    pub fn subtract_y(&self, other: &Self) -> i32 {
        self.y - other.y
    }

    pub fn display(&self) -> String {
        format!("{}{}{}", self.x(), C_BLOCK_LINE, self.y())
    }
}

impl Add for VPoint {
    type Output = VPoint;
    fn add(self, rhs: VPoint) -> Self::Output {
        VPoint {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for VPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
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

impl SubAssign for VPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

fn is_integer_f32(point: impl Borrow<FacBpPosition>) -> bool {
    let point = point.borrow();
    point.x.round() == point.x && point.y.round() == point.y
}

pub fn must_whole_number(point: FacBpPosition) {
    let rounded = FacBpPosition {
        x: point.x.round(),
        y: point.y.round(),
    };
    assert_eq!(rounded, point, "Point is not round {:?}", rounded);
}

pub fn must_odd_number(point: FacBpPosition) {
    assert!(
        !(point.x as i32 % 2 == 0 || point.y as i32 % 2 == 0),
        "Point is even {:?}",
        point
    );
}

pub fn must_even_number(point: FacBpPosition) {
    assert!(
        !(point.x as i32 % 2 == 1 || point.y as i32 % 2 == 1),
        "Point is odd {:?}",
        point
    );
}

pub fn must_half_number(point: FacBpPosition) {
    let dec_x = point.x.floor() - point.x;
    let dec_y = point.y.floor() - point.y;
    assert!(
        !(dec_x > 0.4 && dec_x < 0.6 && dec_y > 0.4 && dec_y < 0.6),
        "Point isn't half {:?}",
        point
    );
}
