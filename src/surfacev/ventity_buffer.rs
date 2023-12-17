use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VPixel;
use crate::util::duration::BasicWatch;
use crate::util::io::{read_entire_file, write_entire_file};
use crate::LOCALE;
use itertools::*;
use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::path::Path;
use std::rc::Rc;
use tracing::debug;

const USIZE_BYTES: usize = (usize::BITS / u8::BITS) as usize;
const EMPTY_XY_INDEX: usize = usize::MAX;

pub trait VEntityXY {
    fn get_xy(&self) -> &[VPoint];
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
    /// A *square* centered on 0,0
    radius: u32,
}

impl<E> VEntityBuffer<E>
where
    E: VEntityXY + Clone + Eq + Hash,
{
    pub fn new(radius: u32) -> Self {
        let mut res = VEntityBuffer {
            entities: Vec::new(),
            xy_to_entity: Vec::new(),
            radius,
        };
        res.init_xy_to_entity();
        res
    }

    fn init_xy_to_entity(&mut self) {
        self.xy_to_entity = vec![EMPTY_XY_INDEX; Self::_xy_array_length_from_radius(self.radius)]
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
                "Cannot make index {}",
                VError::XYOutOfBounds {
                    positions: vec![],
                    backtrace: Backtrace::capture()
                }
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

    pub fn is_xy_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let radius = self.radius as i32;
        let x_valid = x > -radius && x < radius;
        let y_valid = y > -radius && y < radius;
        !x_valid || !y_valid
    }
    //</editor-fold>

    //<editor-fold desc="query point">
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
        self.add_positions(entity_index, positions);
        self.entities.push(entity);

        Ok(())
    }

    pub fn add_positions(&mut self, entity_index: usize, positions: &[VPoint]) {
        for position in positions {
            let xy_index = self.xy_to_index_unchecked(position.x(), position.y());
            self.xy_to_entity[xy_index] = entity_index;
        }
    }

    pub fn remove_positions(&mut self, indexes: impl IntoIterator<Item = usize>) {
        for index in indexes.into_iter().sorted().unique().rev() {
            self.xy_to_entity.remove(index);
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

        let new_xy_length = self.xy_array_length_from_radius();
        debug!(
            "Reduce entities from {} to {}, xy_map from {} to {}",
            old_entity_length,
            self.entities.len(),
            old_xy_length,
            new_xy_length
        );

        self.init_xy_to_entity();

        for i in 0..self.entities.len() {
            let xy_insertable = self.entities[i].get_xy().to_owned();
            self.add_positions(i, &xy_insertable);
        }
    }

    //<editor-fold desc="io">
    pub fn save_xy_file(&self, path: &Path) -> VResult<()> {
        let mut serialize_watch = BasicWatch::start();
        let big_xy_bytes: Vec<u8> = self
            .xy_to_entity
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
        let mut read_watch = BasicWatch::start();
        let big_xy_bytes = read_entire_file(path)?;
        read_watch.stop();

        // Serde does not use new() so this is still uninitialized
        // self.init_xy_to_entity();

        // TODO: Slow :-(
        assert_eq!(self.xy_to_entity.len(), 0, "not empty");
        let deserialize_watch = BasicWatch::start();
        self.xy_to_entity.extend(
            big_xy_bytes
                .into_iter()
                .array_chunks::<USIZE_BYTES>()
                .map(usize::from_ne_bytes),
        );
        debug!(
            "Loading Entity XY read {} deserialize {} path {}",
            read_watch,
            deserialize_watch,
            path.display()
        );

        Ok(())
    }

    // pub fn iter_xy_entities_or_default<'a>(&'a self, default: &'a E) -> impl Iterator<Item = &E> {
    //     self.xy_to_entity.iter().map(move |index| {
    //         if *index == EMPTY_XY_INDEX {
    //             default
    //         } else {
    //             &self.entities[*index]
    //         }
    //     })
    // }

    pub fn iter_entities(&self) -> impl Iterator<Item = &E> {
        self.xy_to_entity
            .iter()
            .filter(|index| **index != EMPTY_XY_INDEX)
            .map(|index| &self.entities[*index])
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
        &self.entities[index]
    }

    pub fn get_entity_by_point(&self, point: &VPoint) -> Option<&E> {
        if self.is_point_out_of_bounds(point) {
            None
        } else {
            let index = self.xy_to_index_unchecked(point.x(), point.y());
            Some(self.get_entity_by_index(index))
        }
    }

    //</editor-fold>
}

impl<E> Display for VEntityBuffer<E> {
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

impl VEntityBuffer<VPixel> {
    pub fn iter_xy_pixels(&self) -> impl Iterator<Item = &Pixel> {
        self.xy_to_entity.iter().map(|index| {
            if *index == EMPTY_XY_INDEX {
                &Pixel::Empty
            } else {
                self.entities[*index].pixel()
            }
        })
    }

    pub fn map_pixel_xy_to_cv(&self, filter: Option<Pixel>) -> Mat {
        let metrics = Rc::new(RefCell::new(Metrics::new("entity-cv-mapper")));

        let output = self.map_xy_entities_to_bigger_u8_vec(|e| {
            if let Some(e) = e {
                if let Some(filter) = filter {
                    if e.pixel() == &filter {
                        metrics
                            .borrow_mut()
                            .increment(&format!("f-{:?}", e.pixel()));
                        [Pixel::Highlighter.into_id()]
                    } else {
                        metrics.borrow_mut().increment("f-empty");
                        [0]
                    }
                } else {
                    metrics.borrow_mut().increment("not-empty");
                    [Pixel::Highlighter.into_id()]
                }
            } else {
                metrics.borrow_mut().increment("empty");
                [0]
            }
        });
        metrics.borrow().log_final();
        let side_length = self.diameter();
        Mat::from_slice_rows_cols(&output, side_length, side_length).unwrap()
        // Mat::new_rows_cols_with_data(side_length, side_length, )
    }
}

#[cfg(test)]
mod test {
    use crate::surfacev::ventity_buffer::VEntityBuffer;
    use crate::surfacev::vpoint::VPoint;
    use crate::surfacev::vsurface::VPixel;

    #[test]
    pub fn to_xy_index_and_back() {
        let buffer: VEntityBuffer<VPixel> = VEntityBuffer::new(50);

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
