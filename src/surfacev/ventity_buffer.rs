use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter, Read, Write};
use std::mem::transmute;
use std::path::Path;

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
    #[serde(skip_serializing)]
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

    pub fn diameter(&self) -> usize {
        self.radius as usize * 2
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

    pub fn new_xy_entity_array(&self) -> impl Iterator<Item = &E> {
        self.xy_to_entity.iter().map(|index| &self.entities[*index])
    }

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

    pub fn xy_array_length_from_radius(&self) -> usize {
        Self::_xy_array_length_from_radius(self.radius)
    }

    fn _xy_array_length_from_radius(radius: u32) -> usize {
        let dia = radius as usize * 2;
        dia * dia
    }
}
