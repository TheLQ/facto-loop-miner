use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufWriter, Read, Write};
use std::mem::transmute;
use std::path::Path;
use tracing::debug;

pub trait VEntityXY {
    fn get_xy(&self) -> Vec<VPoint>;
}

/// Collection of entities and xy positions they cover
///
/// For example, ore tiles cover 1 positions. Assembly machines cover 9 positions
#[derive(Serialize, Deserialize)]
pub struct VEntityBuffer<E> {
    entities: Vec<E>,
    /// More efficient to store a (radius * 2)^2 length Array as a raw file instead of JSON  
    #[serde(skip)]
    xy_to_entity: Vec<usize>,
    radius: u32,
}

impl<E> VEntityBuffer<E>
where
    E: VEntityXY + Clone + Eq + Hash,
{
    pub fn new(radius: u32) -> Self {
        VEntityBuffer {
            entities: Vec::new(),
            xy_to_entity: vec![0; Self::_xy_array_length_from_radius(radius)],
            radius,
        }
    }

    pub fn radius(&self) -> u32 {
        self.radius
    }

    pub fn diameter(&self) -> usize {
        self.radius as usize * 2
    }

    //<editor-fold desc="query xy">
    /// Fast Get index in xy_to_entity buffer
    pub fn xy_to_index_unchecked(&self, x: i32, y: i32) -> usize {
        let radius = self.radius as i64;
        let abs_x = (x as i64 + radius) as usize;
        let abs_y = (y as i64 + radius) as usize;
        self.diameter() * abs_y + abs_x
        // let mut value = self.diameter();
        // value = if let Some(v) = value.checked_mul(abs_y) {
        //     v
        // } else {
        //     panic!(
        //         "can't multiply diameter {} and {} (orig {} radius {})",
        //         self.diameter(),
        //         abs_y,
        //         y,
        //         radius
        //     );
        // };
        // value + abs_x
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

    pub fn check_points_if_in_range_iter<'a>(
        &self,
        points: impl IntoIterator<Item = &'a VPoint>,
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

    pub fn get_points_if_in_range_vec(&self, points: Vec<VPoint>) -> VResult<Vec<VPoint>> {
        self.check_points_if_in_range_iter(points.iter())
            .map(|_| points)
    }
    //</editor-fold>

    pub fn add(&mut self, entity: E) -> VResult<()> {
        let positions = self.get_points_if_in_range_vec(entity.get_xy())?;

        self.entities.push(entity);
        let entity_index = self.entities.len() - 1;
        self.add_positions(entity_index, positions);

        Ok(())
    }

    pub fn add_positions(&mut self, entity_index: usize, positions: Vec<VPoint>) {
        for position in positions {
            let xy_index = self.xy_to_index_unchecked(position.x, position.y);
            self.xy_to_entity[xy_index] = entity_index;
        }
    }

    /// crop entities then rebuild xy_to_entity lookup
    pub fn crop(&mut self, new_radius: u32) {
        self.radius = new_radius;

        let old_entity_length = self.entities.len();
        let old_xy_length = self.xy_to_entity.len();
        self.entities.retain(|e| {
            e.get_xy()
                .iter()
                .all(|i| i.x.unsigned_abs() < new_radius && i.y.unsigned_abs() < new_radius)
        });

        let new_xy_length = self.xy_array_length_from_radius();
        debug!(
            "Reduce entities from {} to {}, xy_map from {} to {}",
            old_entity_length,
            self.entities.len(),
            old_xy_length,
            new_xy_length
        );

        self.xy_to_entity = vec![0usize; new_xy_length];

        for i in 0..self.entities.len() {
            self.add_positions(i, self.entities[i].get_xy());
        }
    }

    //<editor-fold desc="io">
    pub fn save_xy_file(&self, path: &Path) -> VResult<()> {
        let file = File::create(path).map_err(|e| VError::IoError {
            e,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        })?;
        let mut writer = BufWriter::new(file);
        for entry in &self.xy_to_entity {
            let bytes = entry.to_ne_bytes();
            writer.write(&bytes).map_err(|e| VError::IoError {
                e,
                path: path.to_string_lossy().to_string(),
                backtrace: Backtrace::capture(),
            })?;
        }

        Ok(())
    }

    pub fn load_xy_file(&mut self, path: &Path) -> VResult<()> {
        let mut file = File::open(path).map_err(VError::io_error(path))?;

        let working_u8: &mut [u8] = unsafe { transmute(self.xy_to_entity.as_mut_slice()) };
        file.read_exact(working_u8).map_err(|e| VError::IoError {
            e,
            path: path.to_string_lossy().to_string(),
            backtrace: Backtrace::capture(),
        })?;

        Ok(())
    }

    pub fn new_xy_entity_array(&self) -> impl Iterator<Item = &E> {
        self.xy_to_entity.iter().map(|index| &self.entities[*index])
    }

    pub fn xy_array_length_from_radius(&self) -> usize {
        Self::_xy_array_length_from_radius(self.radius)
    }

    fn _xy_array_length_from_radius(radius: u32) -> usize {
        (radius as usize * 2).pow(2)
    }
    //</editor-fold>
}
