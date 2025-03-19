use crate::blueprint::bpfac::position::FacBpPosition;
use crate::err::{FError, FResult};
use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::util::ansi::C_BLOCK_LINE;
use opencv::core::{Point, Rect};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
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
pub const VPOINT_TEN: VPoint = VPoint { x: 10, y: 10 };

impl VPoint {
    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }

    pub const fn new(x: i32, y: i32) -> Self {
        VPoint { x, y }
    }

    pub fn new_usize(x: usize, y: usize) -> Self {
        VPoint {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        }
    }

    // pub fn from_cv_point(point: Point) -> Self {
    //     VPoint {
    //         x: point.x,
    //         y: point.y,
    //     }
    // }

    pub const fn from_value(value: i32) -> Self {
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
    pub const fn to_fac_with_offset_rectangle(
        &self,
        offset_x: f32,
        offset_y: f32,
    ) -> FacBpPosition {
        FacBpPosition::new(self.x as f32 + offset_x, self.y as f32 + offset_y)
    }

    pub const fn to_fac_exact(&self) -> FacBpPosition {
        FacBpPosition::new(self.x as f32, self.y as f32)
    }

    pub const fn to_cv_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub const fn to_slice_f32(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    pub const fn is_within_center_radius(&self, center_radius: u32) -> bool {
        let center_radius = center_radius as i32;
        // note: Excludes -radius,-radius in VArray
        // because origin per-pixel is top-left. The "absolute 0,0" is included,
        // but the "absolute bottom-right" is out-of-bounds as off-by-one
        self.x > -center_radius
            && self.x < center_radius
            && self.y > -center_radius
            && self.y < center_radius
    }

    pub const fn from_rect_start(rect: &Rect) -> Self {
        VPoint {
            x: rect.x,
            y: rect.y,
        }
    }

    pub fn assert_even_position(&self) {
        assert_eq!(self.x % 2, 0, "x={} is not even for {self}", self.x);
        assert_eq!(self.y % 2, 0, "y={} is not even for {self}", self.y);
    }

    pub fn assert_step_rail(&self) {
        // self.assert_even_position();
        assert_eq!(
            self.x % SECTION_POINTS_I32,
            0,
            "x={} is not {SECTION_POINTS_I32} for {self}",
            self.x
        );
        assert_eq!(
            self.y % SECTION_POINTS_I32,
            0,
            "y={} is not {SECTION_POINTS_I32} for {self}",
            self.y
        );
    }

    // pub fn assert_even_8x8_position(&self) {
    //     self.assert_even_position();
    //     assert_eq!(self.x % 8, 0, "x={} is not 8", self.x);
    //     assert_eq!(self.y % 8, 0, "y={} is not 8", self.y);
    // }

    pub fn assert_odd_position(&self) {
        assert_eq!((self.x - 1) % 2, 0, "x={} is not odd", self.x);
        assert_eq!((self.y - 1) % 2, 0, "y={} is not odd", self.y);
    }

    // pub fn assert_odd_8x8_position(&self) {
    //     self.assert_odd_position();
    //     assert_eq!((self.x - 1) % 8, 0, "x={} is not 8", self.x);
    //     assert_eq!((self.y - 1) % 8, 0, "y={} is not 8", self.y);
    // }

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

    pub const fn move_x(&self, steps: i32) -> Self {
        Self {
            x: self.x + steps,
            y: self.y,
        }
    }

    pub const fn move_x_usize(&self, steps: usize) -> Self {
        self.move_x(steps as i32)
    }

    pub const fn move_y(&self, steps: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + steps,
        }
    }

    pub const fn move_y_usize(&self, steps: usize) -> Self {
        self.move_y(steps as i32)
    }

    pub const fn move_xy(&self, x_steps: i32, y_steps: i32) -> Self {
        Self {
            x: self.x + x_steps,
            y: self.y + y_steps,
        }
    }

    pub const fn move_xy_usize(&self, x_steps: usize, y_steps: usize) -> Self {
        self.move_xy(x_steps as i32, y_steps as i32)
    }

    pub fn move_direction_int(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: i32,
    ) -> Self {
        // cardinal directions are "north == up == -1" not "north == +1"
        // println!("move {}", steps);
        match direction.borrow() {
            FacDirectionQuarter::North => self.move_y(-steps),
            FacDirectionQuarter::South => self.move_y(steps),
            FacDirectionQuarter::East => self.move_x(steps),
            FacDirectionQuarter::West => self.move_x(-steps),
        }
    }

    // pub fn move_direction_half(&self, direction: impl Borrow<FacDirectionQuarter>) -> Self {
    //     match direction.borrow() {
    //         FacDirectionQuarter::North => {
    //             self.move_y(-((self.y() as f32 + 0.5).floor() as i32 - self.y()))
    //         }
    //         FacDirectionQuarter::South => {
    //             self.move_y((self.y() as f32 + 0.5).floor() as i32 - self.y())
    //         }
    //         FacDirectionQuarter::East => {
    //             self.move_x((self.x() as f32 + 0.5).floor() as i32 - self.x())
    //         }
    //         FacDirectionQuarter::West => {
    //             self.move_x(-((self.x() as f32 + 0.5).floor() as i32 - self.x()))
    //         }
    //     }
    // }

    pub fn move_direction_usz(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: usize,
    ) -> Self {
        self.move_direction_int(direction, steps as i32)
    }

    pub fn move_direction_if_usz(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: usize,
        enabled: bool,
    ) -> Self {
        if enabled {
            self.move_direction_int(direction, steps as i32)
        } else {
            *self
        }
    }

    pub fn move_direction_sideways_int(
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

    pub fn move_direction_sideways_usz(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: usize,
    ) -> Self {
        self.move_direction_sideways_int(direction, steps as i32)
    }

    pub fn move_direction_sideways_axis_int(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: i32,
    ) -> Self {
        // cardinal directions are "north == up == -1" not "north == +1"
        match direction.borrow() {
            FacDirectionQuarter::North | FacDirectionQuarter::South => self.move_x(steps),
            FacDirectionQuarter::East | FacDirectionQuarter::West => self.move_y(steps),
        }
    }

    pub fn move_direction_sideways_axis_usz(
        &self,
        direction: impl Borrow<FacDirectionQuarter>,
        steps: usize,
    ) -> Self {
        self.move_direction_sideways_axis_int(direction, steps as i32)
    }

    // pub fn move_between_entity_centers(
    //     &self,
    //     first: &Box<dyn FacEntity>,
    //     last: &Box<dyn FacEntity>,
    //     float_x: f32,
    //     float_y: f32,
    // ) -> Self {
    //     let first_facpos = first.to_fac_position(&self);
    //     let last_facpos =
    //         FacBpPosition::new(first_facpos.x() + float_x, first_facpos.y() + float_y);
    //     last.from_fac_position(&last_facpos)
    // }

    // pub fn move_factorio_style_direction_entity(
    //     &self,
    //     first: &Box<dyn FacEntity>,
    //     last: &Box<dyn FacEntity>,
    //     direction: FacDirectionQuarter,
    //     steps: f32,
    // ) -> Self {
    //     let first_facpos = first.to_fac_position(&self);
    //     let mut float_x = 0.0;
    //     let mut float_y = 0.0;
    //     match direction {
    //         FacDirectionQuarter::North => float_y += -steps,
    //         FacDirectionQuarter::South => float_y += steps,
    //         FacDirectionQuarter::East => float_x += steps,
    //         FacDirectionQuarter::West => float_x += -steps,
    //     }
    //     let last_facpos =
    //         FacBpPosition::new(first_facpos.x() + float_x, first_facpos.y() + float_y);
    //     let last_point = last.from_fac_position(&last_facpos);
    //     last_point
    // }

    /// "Why Factorio uses Floats"
    ///
    /// Imagine this belt
    /// ██ ██ ██
    ///    ██
    /// FacPos Y is 10.5, 11.0, 10.5
    /// VPoint Y is 10,   10,   10
    ///
    /// Now we flip
    ///    ██
    /// ██ ██ ██
    /// FacPos Y is 10.5, 10.0, 10.5
    /// VPoint Y is 10,   09,   10
    ///
    ///
    /// In VPoint Integer, we must "if flip add_one else add_zero" which is... annoying
    /// In VPoint converted float, (10 - 0.5).floor() = 9 (good), but (10 + 0.5).floor(10)
    ///
    /// In float we can consistently add or subtract 0.5 then backconvert to VPoint
    pub fn move_factorio_style_direction(
        &self,
        direction: FacDirectionQuarter,
        steps: f32,
    ) -> Self {
        self.to_fac_exact()
            .move_direction_and_vpoint_floor(direction, steps)
    }

    // pub fn move_direction_corrected(
    //     &self,
    //     direction: impl Borrow<FacDirectionQuarter>,
    //     steps: usize,
    // ) -> Self {
    //     let desired_x = steps as f32 + 1.5;
    //     let actual_x = self.x as f32;

    //     let comuted_x = (desired_x + actual_x).floor();
    //     let desired_x_corrected = (comuted_x - actual_x).abs() as usize;
    //     if steps == desired_x_corrected {
    //         trace!("corrected same {steps}")
    //     } else {
    //         trace!("corrected DIFF {steps} to {desired_x_corrected}")
    //     }
    //     self.move_direction_sideways_usz(direction, desired_x_corrected)
    // }

    // pub fn sort_by_direction_fn(
    //     direction: FacDirectionQuarter,
    // ) -> impl FnMut(&VPoint, &VPoint) -> Ordering {
    //     move |left, right| match direction {
    //         FacDirectionQuarter::East => left.x.cmp(&right.x),
    //         _ => unimplemented!(),
    //     }
    // }

    pub fn sort_by_direction(
        direction: FacDirectionQuarter,
        left: VPoint,
        right: VPoint,
    ) -> Ordering {
        match direction {
            FacDirectionQuarter::East => left.x.cmp(&right.x),
            _ => unimplemented!(),
        }
    }

    pub fn trim_max(&self, other: impl Borrow<VPoint>) -> Self {
        let other = other.borrow();
        VPoint::new(self.x().max(other.x()), self.y().max(other.y()))
    }

    pub fn trim_min(&self, other: impl Borrow<VPoint>) -> Self {
        let other = other.borrow();
        VPoint::new(self.x().min(other.x()), self.y().min(other.y()))
    }

    pub fn midpoint(&self, other: impl Borrow<VPoint>) -> Self {
        let other = other.borrow();
        VPoint::new(self.x().midpoint(other.x()), self.y().midpoint(other.y()))
    }

    // pub fn move_xy_u32(&self, x_steps: u32, y_steps: u32) -> Self {
    //     self.move_xy(x_steps as i32, y_steps as i32)
    // }

    pub const fn move_round_rail_down(&self) -> Self {
        self.move_round_down(SECTION_POINTS_I32)
    }

    const fn move_round_down(&self, size: i32) -> Self {
        VPoint {
            x: self.x - (self.x.rem_euclid(size)),
            y: self.y - (self.y.rem_euclid(size)),
        }
    }

    pub const fn move_round_rail_up(&self) -> Self {
        self.move_round_up(SECTION_POINTS_I32)
    }

    const fn move_round_up(&self, size: i32) -> Self {
        VPoint {
            x: self.x + (size - self.x.rem_euclid(size)),
            y: self.y + (size - self.y.rem_euclid(size)),
        }
    }

    pub const fn area_2x2(&self) -> [Self; 4] {
        [
            *self,
            self.move_x(1),
            self.move_y(1),
            self.move_x(1).move_y(1),
        ]
    }
    pub const fn get_entity_area_3x3(&self) -> [Self; 9] {
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
    pub const fn distance_to(&self, other: &Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    // aka hypotenuse I guess?
    pub fn distance_bird(&self, other: &Self) -> f32 {
        let squared_euclidean = self.x.abs_diff(other.x).pow(2) + self.y.abs_diff(other.y).pow(2);
        (squared_euclidean as f32).sqrt()
    }

    pub const fn subtract_x(&self, other: &Self) -> i32 {
        self.x - other.x
    }

    pub const fn subtract_y(&self, other: &Self) -> i32 {
        self.y - other.y
    }

    pub fn display(&self) -> String {
        display_any_pos(self.x(), self.y())
    }
}

impl Display for VPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
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

pub fn display_any_pos(x: impl Display + Debug, y: impl Display + Debug) -> String {
    // doing de-bug of float always does as 0.0 display.
    // Without .N wiping out errors of x=24.64532
    // Neat.
    format!("{:4?}{}{:4?}", x, C_BLOCK_LINE, y)
}
