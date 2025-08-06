use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::util::ansi::C_ARROW_TO_CORNER_SE;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VArea {
    top_left: VPoint,
    bottom_right: VPoint,
}

impl VArea {
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

        VArea {
            top_left: VPoint::new(x_min, y_min),
            bottom_right: VPoint::new(x_max, y_max),
        }
    }

    pub fn from_radius(origin: VPoint, depth: u32) -> Self {
        let mut top_left = origin;
        let mut bottom_right = origin;
        for _ in 0..depth {
            top_left -= VPOINT_ONE;
            bottom_right += VPOINT_ONE;
        }
        VArea {
            top_left,
            bottom_right,
        }
    }

    pub fn contains_point(&self, target: &VPoint) -> bool {
        target.x() >= self.top_left.x()
            && target.x() <= self.bottom_right.x()
            && target.y() >= self.top_left.y()
            && target.y() <= self.bottom_right.y()
    }

    pub fn contains_points_any(
        &self,
        targets: impl IntoIterator<Item = impl Borrow<VPoint>>,
    ) -> bool {
        targets.into_iter().any(|p| self.contains_point(p.borrow()))
    }

    pub fn contains_points_all(
        &self,
        targets: impl IntoIterator<Item = impl Borrow<VPoint>>,
    ) -> bool {
        targets.into_iter().all(|p| self.contains_point(p.borrow()))
    }

    pub fn get_points(&self) -> Vec<VPoint> {
        let mut points = Vec::new();
        for point_x in self.top_left.x()..=self.bottom_right.x() {
            for point_y in self.top_left.y()..=self.bottom_right.y() {
                points.push(VPoint::new(point_x, point_y));
            }
        }
        points
    }

    pub fn desugar(&self) -> VAreaSugar {
        VAreaSugar {
            start_x: self.top_left.x(),
            start_y: self.top_left.y(),
            end_x: self.bottom_right.x(),
            end_y: self.bottom_right.y(),
        }
    }

    pub fn point_top_left(&self) -> VPoint {
        self.top_left
    }

    pub fn point_bottom_right(&self) -> VPoint {
        self.bottom_right
    }

    pub fn calc_point_top_right(&self) -> VPoint {
        VPoint::new(self.bottom_right.x(), self.top_left.y())
    }

    pub fn calc_point_bottom_left(&self) -> VPoint {
        VPoint::new(self.top_left.x(), self.bottom_right.y())
    }

    pub fn point_center(&self) -> VPoint {
        self.top_left.midpoint(self.bottom_right)
    }

    pub fn get_corner_points(&self) -> [VPoint; 2] {
        [self.top_left, self.bottom_right]
    }

    pub fn normalize_step_rail(&self, padding: u32) -> Self {
        let padding_point = VPoint::new(padding as i32, padding as i32);
        VArea {
            top_left: self.top_left.move_round_rail_down() - padding_point,
            bottom_right: self.bottom_right.move_round_rail_up() + padding_point,
        }
    }

    pub fn normalize_3x3(&self) -> Self {
        VArea {
            top_left: self.top_left.move_round_3_down(),
            bottom_right: self.bottom_right.move_round_3_up(),
        }
    }

    pub fn normalize_within_radius(&self, radius: i32) -> Self {
        VArea {
            top_left: self.top_left.trim_max(VPoint::new(-radius, -radius)),
            bottom_right: self.bottom_right.trim_min(VPoint::new(radius, radius)),
        }
    }

    pub fn expand_margin(&self, radius: i32) -> Self {
        let expander = VPoint::new(radius, radius);
        VArea {
            top_left: self.top_left - expander,
            bottom_right: self.bottom_right + expander,
        }
    }

    pub fn as_size(&self) -> VPoint {
        self.bottom_right - self.top_left
    }
}

impl Display for VArea {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!(
            "{} {C_ARROW_TO_CORNER_SE} {}",
            self.top_left, self.bottom_right
        ))
    }
}

pub struct VAreaSugar {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
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
        assert_eq!(area.point_top_left(), VPOINT_ZERO);
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
            area.point_top_left(),
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
        assert_eq!(area.point_top_left(), VPoint::new(-4, -4), "{:?}", area);
        assert_eq!(area.point_bottom_right(), VPoint::new(4, 4), "{:?}", area);
    }

    #[test]
    fn test_normalize_radius_partial() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-5, -5), VPOINT_ONE)
            .normalize_within_radius(4);
        assert_eq!(area.point_top_left(), VPoint::new(-4, -4), "{:?}", area);
        assert_eq!(area.point_bottom_right(), VPOINT_ONE, "{:?}", area);
    }

    #[test]
    fn test_normalize_radius_inside() {
        let area = VArea::from_arbitrary_points_pair(VPoint::new(-2, -2), VPoint::new(2, 2))
            .normalize_within_radius(4);
        assert_eq!(area.point_top_left(), VPoint::new(-2, -2), "{:?}", area);
        assert_eq!(area.point_bottom_right(), VPoint::new(2, 2), "{:?}", area);
    }
}
