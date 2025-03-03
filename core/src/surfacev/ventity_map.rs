use crate::opencv::GeneratedMat;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use crate::surfacev::vsurface::VPixel;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_io::varray::{VArray, EMPTY_XY_INDEX};
use facto_loop_miner_io::{
    get_mebibytes_of_slice_usize, read_entire_file_varray_mmap_lib, write_entire_file,
};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::path::Path;
use tracing::debug;

/// Collection of entities and xy positions they cover
///
/// For example, ore tiles cover 1 positions. Assembly machines cover 9 positions
#[derive(Serialize, Deserialize, Clone)]
pub struct VEntityMap<E> {
    entities: Vec<E>,
    /// More efficient to store a (radius * 2)^2 length Array as a raw file instead of JSON  
    #[serde(skip)]
    xy_to_entity: VArray,
    entity_to_xy: Vec<Vec<VPoint>>,
    /// A *square* centered on 0,0
    radius: u32,
}

impl<E> VEntityMap<E>
where
    E: Clone + Eq + Hash + Debug,
{
    pub fn new(radius: u32) -> Self {
        let res = VEntityMap {
            entities: Vec::new(),
            xy_to_entity: VArray::new_length(Self::_xy_array_length_from_radius(radius)),
            entity_to_xy: Vec::new(),
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

    pub fn add(&mut self, entity: E, positions: Vec<VPoint>) -> VResult<()> {
        self.check_points_if_in_range_iter(&positions)?;

        let entity_index = self.entities.len();

        for position in &positions {
            let xy_index = self.point_to_index(position);
            let existing_entity_index = &mut self.xy_to_entity.as_mut_slice()[xy_index];
            if existing_entity_index == &entity_index {
                // we may be called with duplicate points, which we can't remove
                continue;
            } else if existing_entity_index != &EMPTY_XY_INDEX {
                // remove existing
                self.entity_to_xy[*existing_entity_index].retain(|v| v != position)
            }
            *existing_entity_index = entity_index;
        }

        assert_eq!(self.entity_to_xy.len(), entity_index);
        self.entity_to_xy.push(positions);

        self.entities.push(entity);

        Ok(())
    }

    // fn add_entity_positions(&mut self, entity_index: usize, positions: &[VPoint]) {
    //     for position in positions {
    //         let xy_index = self.xy_to_index_unchecked(position.x(), position.y());
    //         self.xy_to_entity.as_mut_slice()[xy_index] = entity_index;
    //     }
    // }

    // pub fn remove(&mut self, entity_index: usize) {}
    //
    pub fn remove_positions(&mut self, points: &[VPoint]) {
        for point in points {
            let xy_index = self.point_to_index(point);
            let entity_index = self.xy_to_entity.as_slice()[xy_index];

            self.xy_to_entity.as_mut_slice()[xy_index] = EMPTY_XY_INDEX;
            self.entity_to_xy[entity_index].retain(|v| v != point)
        }
    }

    /// crop entities then rebuild xy_to_entity lookup
    pub fn crop(&mut self, new_radius: u32) {
        let old_entity_length = self.entities.len();
        let old_xy_length = self.xy_to_entity.len();

        self.radius = new_radius;
        for positions in self.entity_to_xy.iter_mut() {
            positions.retain(|e| e.is_within_center_radius(new_radius));
        }

        self.xy_to_entity = VArray::new_length(self.xy_array_length_from_radius());
        debug!(
            "Reduce entities from {} to {}, xy_map from {} to {}",
            old_entity_length.to_formatted_string(&LOCALE),
            self.entities.len().to_formatted_string(&LOCALE),
            old_xy_length.to_formatted_string(&LOCALE),
            self.xy_to_entity.len().to_formatted_string(&LOCALE)
        );

        for entity_index in 0..self.entities.len() {
            let res: &[VPoint] = &self.entity_to_xy[entity_index];
            // self.sync_positions_to_xy(entity_index, res);
            for position in res {
                let xy_index = self.xy_to_index_unchecked(position.x(), position.y());
                self.xy_to_entity.as_mut_slice()[xy_index] = entity_index;
            }
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

    pub fn validate(&self) {
        let mut checks = 0;
        for (xy_i, entity_index) in self.xy_to_entity.as_slice().iter().enumerate() {
            if *entity_index == EMPTY_XY_INDEX {
                continue;
            }
            let entity_pos_all = &self
                .entity_to_xy
                .get(*entity_index)
                .unwrap_or_else(|| panic!("fail on entity_index {entity_index}"));
            let as_point = self.index_to_xy(xy_i);
            assert!(entity_pos_all.contains(&as_point));
            assert!(*entity_index < self.entities.len());
            checks += 1;
        }
        for (entity_index, points) in self.entity_to_xy.iter().enumerate() {
            for point in points {
                let xy = self.point_to_index(point);
                assert_eq!(*self.xy_to_entity.as_slice().get(xy).unwrap(), entity_index);
                checks += 1;
            }
            assert!(entity_index < self.entities.len());
            if points.is_empty() {
                tracing::warn!("empty {:?}", self.entities[entity_index])
            }
        }
        debug!("validate {checks} checks");
    }
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

    pub fn map_pixel_xy_to_cv(&self, filter: Option<Pixel>) -> GeneratedMat {
        let metrics = RefCell::new(FastMetrics::new("map_pixel_xy_to_cv".to_string()));

        let data = self.map_xy_entities_to_bigger_u8_vec(|e| {
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

        GeneratedMat {
            cols: side_length,
            rows: side_length,
            data,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::surfacev::ventity_map::VEntityMap;
    use crate::surfacev::vsurface::VPixel;
    use facto_loop_miner_fac_engine::common::vpoint::VPoint;

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
