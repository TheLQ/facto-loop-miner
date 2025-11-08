use crate::opencv::GeneratedMat;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{CoreConvertPathResult, VError, VResult, XYOutOfBoundsError};
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_io::varray::{EMPTY_XY_INDEX, VArray};
use facto_loop_miner_io::{get_mebibytes_of_slice_usize, write_entire_file};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::Path;
use std::simd::prelude::{SimdInt, SimdPartialOrd};
use std::simd::{Mask, Simd};
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
    /// A *square* centered on 0,0
    radius: u32,
}

impl<E> VEntityMap<E>
// where
//     E: Clone + Eq + Hash + Debug,
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
    fn xy_to_index_unchecked(&self, x: i32, y: i32) -> usize {
        let radius = self.radius as i32;
        let abs_x = (x + radius) as usize;
        let abs_y = (y + radius) as usize;
        self.diameter() * abs_y + abs_x
    }

    /// Get index in xy_to_entity buffer
    fn xy_to_index_safe(&self, x: i32, y: i32) -> usize {
        if self.is_xy_out_of_bounds(x, y) {
            panic!(
                "Cannot make index radius {} {}",
                self.radius,
                XYOutOfBoundsError::new(vec![VPoint::new(x, y)])
            )
        }
        self.xy_to_index_unchecked(x, y)
    }

    pub fn index_to_xy(&self, index: usize) -> VPoint {
        if index > self.xy_to_entity.len() {
            panic!("too big {}", index);
        }
        let radius = self.radius as i32;
        let diameter = self.diameter();

        let diameter_component = index - (index % diameter);
        let y = diameter_component / diameter;
        let x = index - diameter_component;
        VPoint::new(x as i32 - radius, y as i32 - radius)
    }

    fn is_xy_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let radius = self.radius as i32;
        let x_valid = x >= -radius && x < radius;
        let y_valid = y >= -radius && y < radius;
        !x_valid || !y_valid
    }
    //</editor-fold>

    //<editor-fold desc="query point">
    fn point_to_index_safe(&self, point: &VPoint) -> usize {
        self.xy_to_index_safe(point.x(), point.y())
    }

    pub fn point_to_index_unchecked(&self, point: &VPoint) -> usize {
        self.xy_to_index_unchecked(point.x(), point.y())
    }

    pub fn is_point_out_of_bounds(&self, point: &VPoint) -> bool {
        self.is_xy_out_of_bounds(point.x(), point.y())
    }

    pub fn is_points_out_of_bounds_any<'a>(
        &self,
        points: impl IntoIterator<Item = &'a VPoint>,
    ) -> bool {
        points.into_iter().any(|p| self.is_point_out_of_bounds(p))
    }

    pub fn is_points_free_safe(&self, points: &[VPoint]) -> bool {
        let xy_lookup = self.xy_to_entity.as_slice();

        points.iter().all(|v| {
            if self.is_point_out_of_bounds(v) {
                // silent
                true
            } else {
                xy_lookup[self.xy_to_index_unchecked(v.x(), v.y())] == EMPTY_XY_INDEX
            }
        })

        // let mut is_out_of_bounds = false;
        // for point in points {
        //     is_out_of_bounds = is_out_of_bounds || self.is_point_out_of_bounds(point);
        // }
        // if is_out_of_bounds {
        //     return false;
        // }
        //
        // let xy_lookup = self.xy_to_entity.as_slice();
        // let mut not_free = false;
        // for point in points {
        //     let index = self.xy_to_index_unchecked(point.x(), point.y());
        //     let is_not_empty = xy_lookup[index] != EMPTY_XY_INDEX;
        //     not_free = not_free || is_not_empty;
        // }
        // !not_free
    }

    // #[inline(never)]
    pub fn is_points_free_unchecked_iter(&self, points: &[VPoint]) -> bool {
        let xy_lookup = self.xy_to_entity.as_slice();

        // This is an extremely hot function. Attempt SIMD
        if false {
            points
                .iter()
                .all(|v| xy_lookup[self.xy_to_index_unchecked(v.x(), v.y())] == EMPTY_XY_INDEX)
        } else {
            // todo: holy magic wtf
            const MAGIC_TOTAL: usize = 104;
            if points.len() != 104 {
                panic!("processing {}", points.len());
            }
            const POINTS_SIZE: usize = 8;
            static_assertions::const_assert!(MAGIC_TOTAL.is_multiple_of(POINTS_SIZE));

            let radius = Simd::splat(self.radius as i32);
            let diameter = Simd::splat(self.diameter() as i32);
            let xy_lookup_len = Simd::splat(xy_lookup.len());
            const EMPTY_INDEXES: Simd<usize, POINTS_SIZE> = Simd::splat(EMPTY_XY_INDEX);

            // magic lets us use pure SIMD ignoring remainder
            let (chunks, _remainder) = points.as_chunks::<POINTS_SIZE>();

            for chunk in chunks {
                let mut as_x: Simd<i32, POINTS_SIZE> = Simd::splat(0);
                let mut as_y: Simd<i32, POINTS_SIZE> = Simd::splat(0);
                for i in 0..POINTS_SIZE {
                    as_x[i] = chunk[i].x();
                    as_y[i] = chunk[i].y();
                }

                let indexes = diameter * (as_y + radius) + (as_x + radius);
                let indexes_usize: Simd<usize, POINTS_SIZE> = indexes.cast();

                assert!(indexes_usize.simd_lt(xy_lookup_len).all());
                // dummy empty indexes
                let resu = unsafe {
                    Simd::gather_select_unchecked(
                        xy_lookup,
                        Mask::splat(true),
                        indexes.cast(),
                        EMPTY_INDEXES,
                    )
                };
                if resu != EMPTY_INDEXES {
                    return false;
                }
            }
            true
        }
    }
    //</editor-fold>

    #[must_use]
    pub fn change<I>(&mut self, positions: I) -> VMapChange<'_, E, I>
    where
        I: IntoIterator<Item = VPoint>,
    {
        VMapChange {
            map: self,
            positions,
        }
    }

    /// crop entities then rebuild xy_to_entity lookup
    pub fn crop(&mut self, new_radius: u32) {
        let mut new = Self {
            radius: new_radius,
            entities: Vec::new(), // dummy
            xy_to_entity: VArray::new_length(Self::_xy_array_length_from_radius(new_radius)),
        };
        debug!(
            "Reduce entities from {} to {}, xy_map from {} to {}",
            self.entities.len().to_formatted_string(&LOCALE),
            new.entities.len().to_formatted_string(&LOCALE),
            self.xy_to_entity.len().to_formatted_string(&LOCALE),
            new.xy_to_entity.len().to_formatted_string(&LOCALE)
        );

        let old_xy_to_entity = self.xy_to_entity.as_slice();
        for xy_index in 0..new.xy_to_entity.as_mut_slice().len() {
            let position = new.index_to_xy(xy_index);
            new.xy_to_entity.as_mut_slice()[xy_index] =
                old_xy_to_entity[self.xy_to_index_unchecked(position.x(), position.y())];
        }

        let Self {
            radius,
            entities: _, // we didn't touch this
            xy_to_entity,
        } = new;
        self.radius = radius;
        self.xy_to_entity = xy_to_entity;
    }

    //<editor-fold desc="io">
    pub fn save_xy_file(&self, path: &Path) -> VResult<()> {
        let write_watch = BasicWatch::start();
        let source = self.xy_to_entity.as_slice();
        let source_len = source.len();
        let (before, data, after) = unsafe { source.align_to() };
        assert_eq!(before.len(), 0);
        assert_eq!(after.len(), 0);
        assert_eq!(data.len(), source_len * 8);
        write_entire_file(path, data).convert(path)?;

        debug!(
            "Saving Entity XY write {} bytes path {} in {write_watch}",
            data.len(),
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
        self.xy_to_entity = VArray::from_path(path).convert(path)?;
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

    pub fn load_clone_prep(&mut self, clone_prep_file: &Path) -> VResult<()> {
        if self.xy_to_entity.is_dirty_for_clone() {
            // Can't write mmap's data back to itself apparently. Failed with "Bad Address"
            match remove_file(clone_prep_file).convert(clone_prep_file) {
                Err(VError::IoError { err, .. }) if err.kind() == ErrorKind::NotFound => {
                    // do nothing
                }
                Err(e) => return Err(e),
                Ok(()) => {}
            };

            self.save_xy_file(clone_prep_file)?;
            self.load_xy_file(clone_prep_file)?;
            assert!(!self.xy_to_entity.is_dirty_for_clone());
        }
        Ok(())
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

    pub fn get_entity_id_at(&self, point: &VPoint) -> usize {
        self.xy_to_entity.as_slice()[self.point_to_index_safe(point)]
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
        let entity_id = self.get_entity_id_at(point);
        if entity_id == EMPTY_XY_INDEX {
            None
        } else {
            Some(self.get_entity_by_index(entity_id))
        }
    }

    pub fn get_entity_by_point_mut(&mut self, point: &VPoint) -> Option<&mut E> {
        let entity_id = self.get_entity_id_at(point);
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

    pub fn validate(&self)
    where
        E: Debug,
    {
        let mut checks = 0;
        for entity_index in self.xy_to_entity.as_slice().iter() {
            if *entity_index == EMPTY_XY_INDEX {
                continue;
            }
            assert!(*entity_index < self.entities.len());
            checks += 1;
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VPixel {
    pub(super) pixel: Pixel,
}

impl VPixel {
    pub fn pixel(&self) -> &Pixel {
        &self.pixel
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

    // pub fn assert_no_empty_pixels(&self) {
    //     for entity_id in self.xy_to_entity.as_slice() {
    //         if *entity_id != EMPTY_XY_INDEX {
    //             assert_ne!(self.entities[*entity_id].pixel().as_ref(), "Empty");
    //         }
    //     }
    // }
}

/// One-stop collection of change operations
pub struct VMapChange<'m, N, I: IntoIterator<Item = VPoint>> {
    map: &'m mut VEntityMap<N>,
    positions: I,
}

impl<'m> VMapChange<'m, VPixel, Vec<VPoint>> {
    pub fn stomp(self, entity: Pixel) {
        assert_ne!(entity, Pixel::Empty);
        assert!(!self.map.is_points_out_of_bounds_any(&self.positions));

        let entity_index = self.map.entities.len();

        for position in &self.positions {
            let xy_index = self.map.point_to_index_unchecked(position);
            let existing_entity_index = &mut self.map.xy_to_entity.as_mut_slice()[xy_index];
            *existing_entity_index = entity_index;
        }

        assert_eq!(self.map.entities.len(), entity_index);
        self.map.entities.push(VPixel { pixel: entity });
    }
}

impl<'m, I> VMapChange<'m, VPixel, I>
where
    I: IntoIterator<Item = VPoint>,
{
    pub fn find_empty_into(self, replace: Pixel) {
        assert_ne!(replace, Pixel::Empty);

        let entity_index = self.map.entities.len();

        for position in self.positions {
            // use safe since iterator can't pre-pass
            let xy_index = self.map.point_to_index_safe(&position);
            let existing_entity_index = &mut self.map.xy_to_entity.as_mut_slice()[xy_index];
            if *existing_entity_index == EMPTY_XY_INDEX {
                // remove existing
                *existing_entity_index = entity_index;
            }
        }

        assert_eq!(self.map.entities.len(), entity_index);
        self.map.entities.push(VPixel { pixel: replace });
    }

    pub fn find_into(self, find: Pixel, replace: Pixel) {
        assert_ne!(find, Pixel::Empty);
        assert_ne!(replace, Pixel::Empty);

        let entity_index = self.map.entities.len();

        for position in self.positions {
            // use safe since iterator can't pre-pass
            let xy_index = self.map.point_to_index_safe(&position);
            let existing_entity_index = &mut self.map.xy_to_entity.as_mut_slice()[xy_index];
            if *existing_entity_index != EMPTY_XY_INDEX
                && self.map.entities[*existing_entity_index].pixel == find
            {
                *existing_entity_index = entity_index;
            }
        }

        assert_eq!(self.map.entities.len(), entity_index);
        self.map.entities.push(VPixel { pixel: replace });
    }

    pub fn remove(self) {
        for point in self.positions {
            assert!(!self.map.is_point_out_of_bounds(&point));

            let xy_index = self.map.point_to_index_unchecked(&point);
            self.map.xy_to_entity.as_mut_slice()[xy_index] = EMPTY_XY_INDEX;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::surfacev::ventity_map::{VEntityMap, VPixel};
    use facto_loop_miner_fac_engine::common::vpoint::VPoint;

    #[test]
    pub fn to_xy_index_and_back() {
        let buffer: VEntityMap<VPixel> = VEntityMap::new(50);

        let test = VPoint::new(25, 20);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index_safe(test.x(), test.y())),
        );

        let test = VPoint::new(-25, -20);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index_safe(test.x(), test.y())),
        );

        let test = VPoint::new(-49, -49);
        assert_eq!(
            test,
            buffer.index_to_xy(buffer.xy_to_index_safe(test.x(), test.y())),
        );
    }
}
