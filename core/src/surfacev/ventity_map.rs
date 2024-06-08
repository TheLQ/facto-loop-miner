use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::path::Path;

use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VPixel;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_io::varray::{VArray, EMPTY_XY_INDEX};

use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use facto_loop_miner_io::{
    get_mebibytes_of_slice_usize, read_entire_file_varray_mmap_lib, write_entire_file,
};

pub trait VEntityXY {
    fn get_xy(&self) -> &[VPoint];
}

/// Collection of entities and xy positions they cover
///
/// For example, ore tiles cover 1 positions. Assembly machines cover 9 positions
#[derive(Serialize, Deserialize, Clone)]
pub struct VEntityMap<E> {
    entities: Vec<E>,
    /// More efficient to store a (radius * 2)^2 length Array as a raw file instead of JSON  
    #[serde(skip)]
    xy_to_entity: VArray,
    /// A *square* centered on 0,0
    radius: u32,
}

impl<E> VEntityMap<E>
where
    E: VEntityXY + Clone + Eq + Hash,
{
    pub fn new(radius: u32) -> Self {
        let res = VEntityMap {
            entities: Vec::new(),
            xy_to_entity: VArray::new_length(Self::_xy_array_length_from_radius(radius)),
            radius,
        };
        res
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
        let radius = self.radius as isize;
        let abs_x = (x as isize + radius) as usize;
        let abs_y = (y as isize + radius) as usize;
        self.diameter() * abs_y + abs_x
    }

    /// Get index in xy_to_entity buffer
    pub fn xy_to_index(&self, x: i32, y: i32) -> usize {
        if self.is_xy_out_of_bounds(x, y) {
            panic!(
                "Cannot make index radius {} {}",
                self.radius,
                VError::XYOutOfBounds {
                    positions: vec![VPoint::new(x, y)],
                    backtrace: Backtrace::capture()
                }
            )
        }
        self.xy_to_index_unchecked(x, y)
    }

    pub fn index_to_xy(&self, index: usize) -> VPoint {
        if index > self.xy_to_entity.len() {
            panic!(
                "too big {} {}",
                index,
                VError::XYOutOfBounds {
                    positions: vec![],
                    backtrace: Backtrace::capture()
                }
            );
        }
        let radius = self.radius as i32;
        let diameter = self.diameter();

        let diameter_component = index - (index % diameter);
        let y = diameter_component / diameter;
        let x = index - diameter_component;
        VPoint::new(x as i32 - radius, y as i32 - radius)
    }

    pub fn is_xy_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let radius = self.radius as i32;
        let x_valid = x > -radius && x < radius;
        let y_valid = y > -radius && y < radius;
        !x_valid || !y_valid
    }
    //</editor-fold>

    //<editor-fold desc="query point">
    pub fn point_to_index(&self, point: &VPoint) -> usize {
        self.xy_to_index(point.x(), point.y())
    }

    pub fn is_point_out_of_bounds(&self, point: &VPoint) -> bool {
        self.is_xy_out_of_bounds(point.x(), point.y())
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

    pub fn is_points_free(&self, points: &[VPoint]) -> bool {
        let mut is_out_of_bounds = false;
        for point in points {
            is_out_of_bounds = is_out_of_bounds || self.is_point_out_of_bounds(point);
        }
        if is_out_of_bounds {
            return false;
        }

        let xy_lookup = self.xy_to_entity.as_slice();
        let mut not_free = false;
        for point in points {
            let index = self.xy_to_index_unchecked(point.x(), point.y());
            let is_not_empty = xy_lookup[index] != EMPTY_XY_INDEX;
            not_free = not_free || is_not_empty;
        }
        !not_free
    }

    pub fn is_points_free_unchecked_iter(&self, points: &[VPoint]) -> bool {
        let xy_lookup = self.xy_to_entity.as_slice();
        // !self.is_point_out_of_bounds(v)
        points
            .iter()
            .all(|v| xy_lookup[self.xy_to_index_unchecked(v.x(), v.y())] == EMPTY_XY_INDEX)
    }

    // pub fn get_points_if_in_range_vec<'a, R>(&self, points: Vec<VPoint>) -> VResult<R>
    // where
    //     R: Iterator<Item = &'a VPoint>,
    // {
    //     self.check_points_if_in_range_iter(points.iter())
    //         .map(|_| points)
    // }
    //</editor-fold>

    pub fn add(&mut self, entity: E) -> VResult<()> {
        let positions = entity.get_xy();
        self.check_points_if_in_range_iter(positions)?;

        let entity_index = self.entities.len();
        self.entities.push(entity);
        self.add_positions_of_entity(entity_index);

        Ok(())
    }

    fn add_positions_of_entity(&mut self, entity_index: usize) {
        for position in self.entities[entity_index].get_xy() {
            let xy_index = self.xy_to_index_unchecked(position.x(), position.y());
            self.xy_to_entity.as_mut_slice()[xy_index] = entity_index;
        }
    }

    pub fn remove(&mut self, entity_index: usize) {}

    pub fn remove_positions(&mut self, points: &[VPoint]) {
        for point in points {
            let index = self.point_to_index(point);
            self.xy_to_entity.as_mut_slice()[index] = EMPTY_XY_INDEX;
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
                .all(|i| i.is_within_center_radius(new_radius))
        });

        self.xy_to_entity = VArray::new_length(self.xy_array_length_from_radius());
        debug!(
            "Reduce entities from {} to {}, xy_map from {} to {}",
            old_entity_length.to_formatted_string(&LOCALE),
            self.entities.len().to_formatted_string(&LOCALE),
            old_xy_length.to_formatted_string(&LOCALE),
            self.xy_to_entity.len().to_formatted_string(&LOCALE)
        );

        for i in 0..self.entities.len() {
            self.add_positions_of_entity(i);
        }
    }

    //<editor-fold desc="io">
    pub fn save_xy_file(&self, path: &Path) -> VResult<()> {
        let mut serialize_watch = BasicWatch::start();
        let big_xy_bytes: Vec<u8> = self
            .xy_to_entity
            .as_slice()
            .iter()
            .flat_map(|v| usize::to_ne_bytes(*v))
            .collect();
        serialize_watch.stop();

        let write_watch = BasicWatch::start();
        write_entire_file(path, &big_xy_bytes)?;

        debug!(
            "Saving Entity XY serialize {} write {} bytes {} path {}",
            serialize_watch,
            write_watch,
            big_xy_bytes.len(),
            path.display()
        );

        Ok(())
    }

    pub fn load_xy_file(&mut self, path: &Path) -> VResult<()> {
        match 3 {
            // 0 => self._load_xy_file_slow(path),
            // 1 => self._load_xy_file_bytemuck(path),
            // 2 => self._load_xy_file_transmute(path),
            3 => self._load_xy_file_mmap(path),
            _ => panic!(),
        }
    }

    // fn _load_xy_file_slow(&mut self, path: &Path) -> VResult<()> {
    //     let mut read_watch = BasicWatch::start();
    //     let xy_bytes_u8 = read_entire_file(path)?;
    //     read_watch.stop();
    //
    //     // Serde does not use new() so this is still uninitialized
    //     // self.init_xy_to_entity();
    //
    //     // TODO: Slow :-(
    //     assert_eq!(self.xy_to_entity.len(), 0, "not empty");
    //     let deserialize_watch = BasicWatch::start();
    //     self.xy_to_entity = vec![0; xy_bytes_u8.len() * 8];
    //     map_u8_to_usize_slice(&xy_bytes_u8, self.xy_to_entity.as_mut_slice());
    //     debug!(
    //         "Loading Entity XY (slow) read {} deserialize {} bytes {} path {}",
    //         read_watch,
    //         deserialize_watch,
    //         get_mebibytes_of_slice_usize(&self.xy_to_entity),
    //         path.display()
    //     );
    //
    //     Ok(())
    // }
    //
    // fn _load_xy_file_bytemuck(&mut self, path: &Path) -> VResult<()> {
    //     let total_watch = BasicWatch::start();
    //     let raw_bytes = read_entire_file(path)?;
    //
    //     let converted_bytes: Vec<usize> = cast_vec(raw_bytes);
    //     self.xy_to_entity = converted_bytes;
    //     debug!(
    //         "Loading Entity XY total {} path {}",
    //         total_watch,
    //         path.display()
    //     );
    //
    //     Ok(())
    // }
    //
    // fn _load_xy_file_transmute(&mut self, path: &Path) -> VResult<()> {
    //     let total_watch = BasicWatch::start();
    //     self.xy_to_entity = read_entire_file_usize_transmute_broken(path)?;
    //     debug!(
    //         "Loading Entity XY (transmute) total {} path {}",
    //         total_watch,
    //         path.display()
    //     );
    //
    //     Ok(())
    // }

    fn _load_xy_file_mmap(&mut self, path: &Path) -> VResult<()> {
        let total_watch = BasicWatch::start();
        self.xy_to_entity = read_entire_file_varray_mmap_lib(path)?;
        debug!(
            "Loading Entity XY (mmap) total {} / {} in {} path {}",
            self.xy_to_entity.len().to_formatted_string(&LOCALE),
            get_mebibytes_of_slice_usize(self.xy_to_entity.as_slice()),
            total_watch,
            path.display()
        );

        Ok(())
    }

    pub fn load_xy_from_other(&mut self, other: Self) {
        self.xy_to_entity = other.xy_to_entity;
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &E> {
        self.entities.iter()
    }

    pub fn xy_array_length_from_radius(&self) -> usize {
        Self::_xy_array_length_from_radius(self.radius)
    }

    fn _xy_array_length_from_radius(radius: u32) -> usize {
        (radius as usize * 2).pow(2)
    }

    pub fn map_xy_entities_to_bigger_u8_vec<const MAPPED_SIZE: usize>(
        &self,
        mapper: impl Fn(Option<&E>) -> [u8; MAPPED_SIZE],
    ) -> Vec<u8> {
        match 1 {
            // 0 => self._map_xy_to_vec_targeted(mapper),
            1 => self._map_xy_to_vec_inlined(mapper),
            _ => panic!("unknown"),
        }
    }

    fn _map_xy_to_vec_inlined<const INLINED_MAPPED_SIZE: usize>(
        &self,
        mapper: impl Fn(Option<&E>) -> [u8; INLINED_MAPPED_SIZE],
    ) -> Vec<u8> {
        let result: Vec<u8> = self
            .xy_to_entity
            .as_slice()
            .iter()
            .map(|index| self.entities.get(*index))
            .flat_map(mapper)
            .collect();
        assert_eq!(
            result.len(),
            self.xy_array_length_from_radius() * INLINED_MAPPED_SIZE,
            "unexpected array size from base {} and mapper size {}",
            self.xy_to_entity.len(),
            INLINED_MAPPED_SIZE
        );
        result
    }

    // fn _map_xy_to_vec_targeted<const TARGETED_MAPPED_SIZE: usize>(
    //     &self,
    //     mapper: impl Fn(Option<&E>) -> [u8; TARGETED_MAPPED_SIZE],
    // ) -> Vec<u8> {
    //     let mut image = vec![0u8; self.xy_to_entity.len() * TARGETED_MAPPED_SIZE];
    //     trace!("mapping {} total {}", TARGETED_MAPPED_SIZE, image.len());
    //     for entity_index in &self.xy_to_entity {
    //         let mapped_value = mapper(self.entities.get(*entity_index));
    //         let index = self.xy_to_index(entity_pos.x(), entity_pos.y()) * TARGETED_MAPPED_SIZE;
    //         image[index..(index + TARGETED_MAPPED_SIZE)].copy_from_slice(&color);
    //
    //         for entity_pos in entity.get_xy() {
    //             let color = mapper(entity);
    //             if index + TARGETED_MAPPED_SIZE > image.len() {
    //                 trace!("overflowing for index {}", index);
    //             }
    //             // todo recheck this for >1
    //             if TARGETED_MAPPED_SIZE == 1 {
    //                 image[index] = color[0];
    //             } else {
    //                 image[index..(index + TARGETED_MAPPED_SIZE)].copy_from_slice(&color);
    //             }
    //         }
    //     }
    //     trace!(
    //         "mapped xy from {} to {}",
    //         self.xy_to_entity.len(),
    //         image.len()
    //     );
    //     image
    // }

    pub fn get_entity_by_index(&self, index: usize) -> &E {
        match self.entities.get(index) {
            Some(v) => v,
            None => panic!("bad index {}", index),
        }
    }

    pub fn get_entity_by_index_mut(&mut self, index: usize) -> &mut E {
        self.entities.get_mut(index).unwrap()
    }

    pub fn get_entity_by_point(&self, point: &VPoint) -> Option<&E> {
        let index = self.xy_to_index(point.x(), point.y());
        let entity_id = self.xy_to_entity.as_slice()[index];
        if entity_id == EMPTY_XY_INDEX {
            None
        } else {
            Some(self.get_entity_by_index(entity_id))
        }
    }

    pub fn get_entity_by_point_mut(&mut self, point: &VPoint) -> Option<&mut E> {
        let index = self.xy_to_index(point.x(), point.y());
        let entity_id = self.xy_to_entity.as_slice()[index];
        if entity_id == EMPTY_XY_INDEX {
            None
        } else {
            Some(self.get_entity_by_index_mut(entity_id))
        }
    }

    pub fn iter_xy_entities_and_points(&self) -> impl Iterator<Item = (VPoint, Option<&E>)> {
        self.xy_to_entity
            .as_slice()
            .iter()
            .enumerate()
            .map(|(index, entity_id)| {
                let point = self.index_to_xy(index);
                if *entity_id == EMPTY_XY_INDEX {
                    (point, None)
                } else {
                    (point, Some(self.get_entity_by_index(*entity_id)))
                }
            })
    }

    //</editor-fold>
}

impl<E> Display for VEntityMap<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VEntityBuffer radius {} entities {} xy {}",
            self.radius,
            self.entities.len().to_formatted_string(&LOCALE),
            self.xy_to_entity.len().to_formatted_string(&LOCALE)
        )
    }
}

impl VEntityMap<VPixel> {
    pub fn iter_xy_pixels(&self) -> impl Iterator<Item = &Pixel> {
        self.xy_to_entity.as_slice().iter().map(|index| {
            if *index == EMPTY_XY_INDEX {
                &Pixel::Empty
            } else {
                self.entities[*index].pixel()
            }
        })
    }

    pub fn map_pixel_xy_to_cv(&self, filter: Option<Pixel>) -> Mat {
        let metrics = RefCell::new(FastMetrics::new("map_pixel_xy_to_cv".to_string()));

        let output = self.map_xy_entities_to_bigger_u8_vec(|e| {
            if let Some(e) = e {
                if let Some(filter) = filter {
                    if *e.pixel() == filter {
                        metrics
                            .borrow_mut()
                            .increment(FastMetric::PixelCvMapper_Filter(*e.pixel()));
                        [Pixel::Highlighter.into_id()]
                    } else {
                        metrics
                            .borrow_mut()
                            .increment(FastMetric::PixelCvMapper_FilterEmpty);
                        [0]
                    }
                } else {
                    metrics
                        .borrow_mut()
                        .increment(FastMetric::PixelCvMapper_NotEmpty);
                    [Pixel::Highlighter.into_id()]
                }
            } else {
                metrics
                    .borrow_mut()
                    .increment(FastMetric::PixelCvMapper_Empty);
                [0]
            }
        });
        metrics.into_inner().log_final();
        let side_length = self.diameter();
        Mat::from_slice_rows_cols(&output, side_length, side_length).unwrap()
        // Mat::new_rows_cols_with_data(side_length, side_length, )
    }
}

#[cfg(test)]
mod test {
    use crate::surfacev::ventity_map::VEntityMap;
    use crate::surfacev::vpoint::VPoint;
    use crate::surfacev::vsurface::VPixel;

    #[test]
    pub fn to_xy_index_and_back() {
        let buffer: VEntityMap<VPixel> = VEntityMap::new(50);

        let test = VPoint::new(25, 20);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index(test.x(), test.y())),
        );

        let test = VPoint::new(-25, -20);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index(test.x(), test.y())),
        );

        let test = VPoint::new(-49, -49);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index(test.x(), test.y())),
        );
    }
}
