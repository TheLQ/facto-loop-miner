use crate::surface::surface::PointU32;
use crate::surfacev::err::{VError, VResult};
use bitvec::macros::internal::funty::Fundamental;
use opencv::core::Point2f;
use crate::surfacev::vpoint::VPoint;

pub trait VEntityXY {
    fn get_xy(&self) -> Vec<VPoint>;
}

pub struct VEntityBuffer<E> {
    entities: Vec<E>,
    xy_to_entity: Vec<usize>,
    radius: u32,
}

impl<E> VEntityBuffer<E>
where
    E: VEntityXY,
{
    pub fn new(radius: u32) -> Self {
        VEntityBuffer {
            entities: Vec::new(),
            xy_to_entity: Vec::new(),
            radius,
        }
    }

    //<editor-fold desc="get xy">
    /// Fast Get index in xy_to_entity buffer
    pub fn xy_to_index_unchecked(&self, x: i32, y: i32) -> usize {
        let radius = self.radius as i32;
        let diameter = (self.radius * 2) as usize;
        // todo: convert to `as usize`
        let abs_x: usize = (x + radius) as usize;
        let abs_y: usize = (y + radius) as usize;
        diameter * abs_y + abs_x
    }

    /// Get index in xy_to_entity buffer
    pub fn xy_to_index(&self, x: i32, y: i32) -> usize {
        if self.is_xy_out_of_bounds(x, y) {
            panic!(
                "Cannot make index {}",
                VError::XYOutOfBounds { positions: vec![] }
            )
        }
        self.xy_to_index_unchecked(x, y)
    }

    pub fn is_xy_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let radius = self.radius as i32;
        let x_valid = x > -radius && x < radius;
        let y_valid = y > -radius && y < radius;
        !x_valid || !y_valid
    }
    //</editor-fold>

    //<editor-fold desc="point">
    pub fn is_point_out_of_bounds(&self, point: &VPoint) -> bool {
        self.is_xy_out_of_bounds(point.x, point.y)
    }

    pub fn is_points_out_of_bounds_iter<'a>(
        &self,
        points: impl Iterator<Item = &'a VPoint>,
    ) -> VResult<()> {
        let mut bad = Vec::new();
        for point in points {
            if self.is_point_out_of_bounds(point) {
                bad.push(*point);
            }
        }
        if bad.is_empty() {
            Ok(())
        } else {
            Err(VError::XYOutOfBounds { positions: bad })
        }
    }

    pub fn is_points_out_of_bounds_vec(&self, points: Vec<VPoint>) -> VResult<Vec<VPoint>> {
        self.is_points_out_of_bounds_iter(points.iter())
            .map(|_| points)
    }
    //</editor-fold>

    pub fn add(&mut self, entity: E) -> VResult<()> {
        let positions = self.is_points_out_of_bounds_vec(entity.get_xy())?;

        self.entities.push(entity);
        let entity_index = self.entities.len() - 1;

        for position in positions {
            let xy_index = self.xy_to_index_unchecked(position.x, position.y);
            self.xy_to_entity[xy_index] = entity_index;
        }

        Ok(())
    }
}
