use crate::blueprint::bpfac::position::FacBpPosition;
use crate::err::{FError, FResult};
use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::util::ansi::C_BLOCK_LINE;
use core::fmt;
use opencv::core::{Point, Rect, Size};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Core XY Point. Entity origin is top left, not Factorio's center
#[derive(Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct VPoint {
    x: i32,
    y: i32,
}

pub const VPOINT_ZERO: VPoint = VPoint { x: 0, y: 0 };
pub const VPOINT_ONE: VPoint = VPoint { x: 1, y: 1 };
pub const VPOINT_THREE: VPoint = VPoint { x: 3, y: 3 };
pub const VPOINT_EIGHT: VPoint = VPoint { x: 8, y: 8 };
pub const VPOINT_TEN: VPoint = VPoint { x: 10, y: 10 };
pub const VPOINT_SECTION: VPoint = VPoint {
    x: SECTION_POINTS_I32,
    y: SECTION_POINTS_I32,
};
pub const VPOINT_SECTION_Y_ONLY: VPoint = VPoint {
    x: 0,
    y: SECTION_POINTS_I32,
};
pub const VPOINT_SECTION_NEGATIVE: VPoint = VPoint {
    x: -SECTION_POINTS_I32,
    y: -SECTION_POINTS_I32,
};

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

    pub fn new_extreme_min() -> Self {
        Self {
            x: i32::MIN,
            y: i32::MIN,
        }
    }

    pub fn new_extreme_max() -> Self {
        Self {
            x: i32::MAX,
            y: i32::MAX,
        }
    }

    pub fn from_cv_point(point: Point) -> Self {
        VPoint {
            x: point.x,
            y: point.y,
        }
    }

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
        // Both top-left and bottom-right are included
        // But bottom-right is off-by-one out of bounds
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

    pub fn test_step_rail(&self) -> Option<String> {
        if self.x % SECTION_POINTS_I32 != 0 {
            Some(format!(
                "x={} is not {SECTION_POINTS_I32} for {self}",
                self.x
            ))
        } else if self.y % SECTION_POINTS_I32 != 0 {
            Some(format!(
                "y={} is not {SECTION_POINTS_I32} for {self}",
                self.y
            ))
        } else {
            None
        }
    }

    pub fn assert_odd_position(&self) {
        assert_eq!((self.x - 1) % 2, 0, "x={} is not odd", self.x);
        assert_eq!((self.y - 1) % 2, 0, "y={} is not odd", self.y);
    }

    pub fn is_even(&self) -> bool {
        self.x % 2 == 0 && self.y % 2 == 0
    }

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

    pub fn sort_by_x_then_y_column(a: impl Borrow<VPoint>, b: impl Borrow<VPoint>) -> Ordering {
        let a = a.borrow();
        let b = b.borrow();
        match a.x().cmp(&b.x()) {
            Ordering::Equal => a.y().cmp(&b.y()),
            x => x,
        }
    }

    pub fn sort_by_y_then_x_row(a: impl Borrow<VPoint>, b: impl Borrow<VPoint>) -> Ordering {
        let a = a.borrow();
        let b = b.borrow();
        match a.y().cmp(&b.y()) {
            Ordering::Equal => a.x().cmp(&b.x()),
            x => x,
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

    ////

    pub const fn move_round_rail_down(&self) -> Self {
        self.move_round_down(SECTION_POINTS_I32)
    }

    pub const fn move_round_even_down(&self) -> Self {
        self.move_round_down(2)
    }

    pub const fn move_round_3_down(&self) -> Self {
        self.move_round_down(3)
    }

    const fn move_round_down(&self, size: i32) -> Self {
        VPoint {
            x: self.x - (self.x.rem_euclid(size)),
            y: self.y - (self.y.rem_euclid(size)),
        }
    }

    ////

    pub const fn move_round_rail_up(&self) -> Self {
        self.move_round_up(SECTION_POINTS_I32)
    }

    pub const fn move_round_even_up(&self) -> Self {
        self.move_round_up(2)
    }

    pub const fn move_round_3_up(&self) -> Self {
        self.move_round_up(3)
    }

    const fn move_round_up(&self, size: i32) -> Self {
        // avoid rounding up on rem=0
        VPoint {
            x: self.x.next_multiple_of(size),
            y: self.y.next_multiple_of(size),
        }
    }

    ////

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

    pub fn get_entity_area_square(&self, size: u8) -> Vec<VPoint> {
        let capacity = (size as usize * 2).pow(2);
        let mut total = Vec::with_capacity(capacity);
        let size = size as i32;
        for x in (-size)..size {
            for y in (-size)..size {
                total.push(self.move_xy(x, y))
            }
        }
        assert_eq!(total.len(), capacity, "{size}");
        total
    }

    // aka Manhattan distance
    pub const fn distance_to(&self, other: &Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    // aka Pythagorean theorem
    pub fn distance_bird(&self, other: &Self) -> f32 {
        let hypotenuse = self.x.abs_diff(other.x).pow(2) + self.y.abs_diff(other.y).pow(2);
        (hypotenuse as f32).sqrt()
    }

    pub const fn subtract_x(&self, other: &Self) -> i32 {
        self.x - other.x
    }

    pub const fn subtract_y(&self, other: &Self) -> i32 {
        self.y - other.y
    }

    pub fn to_cv_size(&self) -> Size {
        Size {
            height: self.y,
            width: self.x,
        }
    }

    // pub fn sugar(&self) -> VPointSugar {
    //     VPointSugar(self.x, self.y)
    // }
}

impl Display for VPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { x, y } = self;
        display_any_pos(f, format_args!("{x:>4}"), format_args!("{y:>4}"))
    }
}

impl Debug for VPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { x, y } = self;
        write!(f, "VPoint ",)?;
        display_any_pos(f, format_args!("{x:>4}"), format_args!("{y:>4}"))
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

impl Add for &VPoint {
    type Output = VPoint;
    fn add(self, rhs: &VPoint) -> Self::Output {
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

impl Sub for &VPoint {
    type Output = VPoint;
    fn sub(self, rhs: &VPoint) -> Self::Output {
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

// pub struct VPointSugar(pub i32, pub i32);
//
// impl Display for VPointSugar {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let Self(x, y) = self;
//         display_any_pos(f, format_args!("{x:>4}"), format_args!("{y:>4}"))
//     }
// }

fn is_integer_f32(point: impl Borrow<FacBpPosition>) -> bool {
    let point = point.borrow();
    point.x.round() == point.x && point.y.round() == point.y
}

pub fn must_whole_number(point: FacBpPosition) {
    let rounded = FacBpPosition {
        x: point.x.round(),
        y: point.y.round(),
    };
    assert_eq!(rounded, point, "Point is not round {rounded:?}");
}

pub fn must_odd_number(point: FacBpPosition) {
    assert!(
        !(point.x as i32 % 2 == 0 || point.y as i32 % 2 == 0),
        "Point is even {point:?}"
    );
}

pub fn must_even_number(point: FacBpPosition) {
    assert!(
        !(point.x as i32 % 2 == 1 || point.y as i32 % 2 == 1),
        "Point is odd {point:?}"
    );
}

pub fn must_half_number(point: FacBpPosition) {
    let dec_x = point.x.floor() - point.x;
    let dec_y = point.y.floor() - point.y;
    assert!(
        !(dec_x > 0.4 && dec_x < 0.6 && dec_y > 0.4 && dec_y < 0.6),
        "Point isn't half {point:?}"
    );
}

pub fn display_any_pos(f: &mut Formatter<'_>, x: fmt::Arguments, y: fmt::Arguments) -> fmt::Result {
    f.write_fmt(x)?;
    f.write_char(C_BLOCK_LINE)?;
    f.write_fmt(y)
}

#[cfg(test)]
mod test {
    use crate::common::vpoint::{VPOINT_ZERO, VPoint};
    use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;

    #[test]
    fn write() {
        println!("{}", VPoint::new(0, 0));
    }

    #[test]
    fn rounding_sanity_down() {
        let pos = VPoint::new(SECTION_POINTS_I32 - 1, SECTION_POINTS_I32 - 1);
        assert_eq!(pos.move_round_rail_down(), VPOINT_ZERO);
    }

    #[test]
    fn rounding_sanity_up() {
        let pos = VPoint::new(1, 1);
        assert_eq!(
            pos.move_round_rail_up(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32)
        );
    }

    #[test]
    fn rounding_sanity_divisor() {
        fn check(v: i32) {
            let pos = VPoint::new(v, v);
            assert_eq!(pos.move_round_rail_down(), pos);
            assert_eq!(pos.move_round_rail_up(), pos);
        }
        check(SECTION_POINTS_I32);
        check(0);
        check(-SECTION_POINTS_I32);
    }
}
