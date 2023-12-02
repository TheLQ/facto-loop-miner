use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::hash::Hash;

pub trait VEntityXY {
    fn get_xy(&self) -> Vec<VPoint>;
}

#[derive(Serialize, Deserialize)]
pub struct VEntityBuffer<E> {
    entities: Vec<E>,
    xy_to_entity: Vec<usize>,
    radius: u32,
}

impl<E> VEntityBuffer<E>
where
    E: VEntityXY + Clone + Eq + Hash,
{
    pub fn new(radius: u32) -> Self {
        let diameter = radius as usize * 2;
        VEntityBuffer {
            entities: Vec::new(),
            xy_to_entity: vec![0; diameter * diameter],
            radius,
        }
    }

    //<editor-fold desc="query xy">
    /// Fast Get index in xy_to_entity buffer
    pub fn xy_to_index_unchecked(&self, x: i32, y: i32) -> usize {
        let radius = self.radius as i32;
        let diameter = (self.radius * 2) as usize;
        let abs_x: usize = (x + radius) as usize;
        let abs_y: usize = (y + radius) as usize;
        diameter * abs_y + abs_x
    }

    /// Get index in xy_to_entity buffer
    pub fn xy_to_index(&self, x: i32, y: i32) -> usize {
        if self.is_xy_out_of_bounds(x, y) {
            panic!(
                "Cannot make index {}",
                VError::XYOutOfBounds {
                    positions: vec![],
                    backtrace: Backtrace::capture()
                }
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

    //<editor-fold desc="query point">
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
            Err(VError::XYOutOfBounds {
                positions: bad,
                backtrace: Backtrace::capture(),
            })
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

    pub fn new_xv_entity_array(&self) -> Vec<E> {
        self.xy_to_entity
            .iter()
            .map(|index| self.entities[*index].clone())
            .collect()
    }

    pub fn diameter(&self) -> usize {
        self.radius as usize * 2
    }
}
