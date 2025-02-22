use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
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

    pub fn from_arbitrary_points_pair<P: Borrow<VPoint>>(a: P, b: P) -> VArea {
        Self::from_arbitrary_points([a, b])
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

    fn end_x_exclusive(&self) -> i32 {
        self.start.x() + self.width as i32
    }

    fn end_y_exclusive(&self) -> i32 {
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

    pub fn point_bottom_right(&self) -> VPoint {
        VPoint::new(self.end_x_exclusive(), self.end_y_exclusive())
    }

    pub fn get_corner_points(&self) -> [VPoint; 2] {
        [self.start, self.point_bottom_right()]
    }

    pub fn normalize_step_rail(&self, padding: u32) -> Self {
        let padding_i32 = padding as i32;
        let x_adjust = self.start.x().rem_euclid(SECTION_POINTS_I32);
        let y_adjust = self.start.y().rem_euclid(SECTION_POINTS_I32);

        let mut new = VArea {
            start: self.start.move_xy(-x_adjust, -y_adjust),
            height: (self.height as i32 + y_adjust) as u32,
            width: (self.width as i32 + x_adjust) as u32,
        };
        new.start.assert_step_rail();

        let bottom_left = new.point_bottom_right();
        let x_adjust = SECTION_POINTS_I32 - (bottom_left.x().rem_euclid(SECTION_POINTS_I32));
        let y_adjust = SECTION_POINTS_I32 - (bottom_left.y().rem_euclid(SECTION_POINTS_I32));
        new.width = (new.width as i32 + x_adjust) as u32;
        new.height = (new.height as i32 + y_adjust) as u32;

        new.point_bottom_right().assert_step_rail();

        new.start = new.start.move_xy(padding_i32, padding_i32);
        new.width -= padding;
        new.height -= padding;

        new
    }

    pub fn normalize_within_radius(&self, radius: i32) -> Self {
        let mut next = self.clone();

        {
            let x_adjust = -radius - self.start.x();
            if x_adjust > 0 {
                next.start = next.start.move_x(x_adjust);
                next.width -= x_adjust as u32;
            }
        }
        {
            let y_adjust = -radius - self.start.y();
            if y_adjust > 0 {
                next.start = next.start.move_y(y_adjust);
                next.height -= y_adjust as u32;
            }
        }
        {
            let x_adjust = self.point_bottom_right().x() - radius;
            if x_adjust > 0 {
                next.width -= x_adjust as u32;
            }
        }
        {
            let y_adjust = self.point_bottom_right().y() - radius;
            if y_adjust > 0 {
                next.height -= y_adjust as u32;
            }
        }
        next
    }

    pub fn point_center(&self) -> VPoint {
        VPoint::new(
            self.start.x() + (self.width as i32 / 2),
            self.start.y() + (self.height as i32 / 2),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::common::varea::VArea;
    use crate::common::vpoint::{VPOINT_ONE, VPOINT_TEN, VPOINT_ZERO, VPoint};
    use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;

    #[test]
    fn test_area_inclusive() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(2, 2), VPoint::new(4, 4));
        assert!(!area.contains_point(&VPoint::new(0, 0)));
        assert!(!area.contains_point(&VPoint::new(1, 1)));
        assert!(area.contains_point(&VPoint::new(2, 2)));
        assert!(area.contains_point(&VPoint::new(3, 3)));
        assert!(area.contains_point(&VPoint::new(4, 4)));
        assert!(!area.contains_point(&VPoint::new(5, 5)));
        assert!(!area.contains_point(&VPoint::new(6, 6)));

        let [start, end] = area.get_corner_points();
        assert_eq!(start, VPoint::new(2, 2));
        assert_eq!(end, VPoint::new(4, 4));

        let points = area.get_points();
        assert!(!points.contains(&VPoint::new(0, 0)));
        assert!(!points.contains(&VPoint::new(1, 1)));
        assert!(points.contains(&VPoint::new(2, 2)));
        assert!(points.contains(&VPoint::new(3, 3)));
        assert!(points.contains(&VPoint::new(4, 4)));
        assert!(!points.contains(&VPoint::new(5, 5)));
        assert!(!points.contains(&VPoint::new(6, 6)));
    }

    #[test]
    fn test_normalize_rail_inside() {
        let area = VArea::from_arbitrary_points_pair(VPOINT_ONE, VPOINT_TEN).normalize_step_rail(0);
        assert_eq!(area.start, VPOINT_ZERO);
        assert_eq!(
            area.point_bottom_right(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32)
        );
    }

    #[test]
    fn test_normalize_rail_partial() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-1, -1), VPOINT_TEN)
            .normalize_step_rail(0);
        assert_eq!(
            area.start,
            VPoint::new(-SECTION_POINTS_I32, -SECTION_POINTS_I32)
        );
        assert_eq!(
            area.point_bottom_right(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32)
        );
    }

    #[test]
    fn test_normalize_radius_outside() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-5, -5), VPOINT_TEN)
            .normalize_within_radius(4);
        assert_eq!(area.start, VPoint::new(-4, -4));
        assert_eq!(area.point_bottom_right(), VPoint::new(4, 4));
    }

    #[test]
    fn test_normalize_radius_partial() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-5, -5), VPOINT_ONE)
            .normalize_within_radius(4);
        assert_eq!(area.start, VPoint::new(-4, -4));
        assert_eq!(area.point_bottom_right(), VPOINT_ONE);
    }

    #[test]
    fn test_normalize_radius_inside() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-2, -2), VPoint::new(2, 2))
            .normalize_within_radius(4);
        assert_eq!(area.start, VPoint::new(-2, -2));
        assert_eq!(area.point_bottom_right(), VPoint::new(2, 2));
    }
}
