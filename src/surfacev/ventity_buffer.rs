use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VPixel;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{debug, trace};

const USIZE_BYTES: usize = (usize::BITS / u8::BITS) as usize;

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
        self.xy_to_entity = vec![0; Self::_xy_array_length_from_radius(self.radius)]
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
            let xy_insertable = self.entities[i].get_xy().to_owned();
            self.add_positions(i, &xy_insertable);
        }
    }

    //<editor-fold desc="io">
    pub fn save_xy_file(&self, path: &Path) -> VResult<()> {
        let mut file = File::create(path).map_err(VError::io_error(path))?;
        let serialize_watch = BasicWatch::start();
        let big_xy_bytes: Vec<u8> = self
            .xy_to_entity
            .iter()
            .flat_map(|e| e.to_ne_bytes())
            .collect();
        trace!("Serialized xy in {}", serialize_watch);
        file.write_all(&big_xy_bytes)
            .map_err(VError::io_error(path))?;
        Ok(())
    }

    pub fn load_xy_file(&mut self, path: &Path) -> VResult<()> {
        let mut file = File::open(path).map_err(VError::io_error(path))?;

        // Serde does not use new() so this is still uninitialized
        // self.init_xy_to_entity();

        let mut big_xy_bytes: Vec<u8> = Vec::new();
        file.read_to_end(&mut big_xy_bytes)
            .map_err(VError::io_error(path))?;

        // TODO: Slow :-(
        assert_eq!(self.xy_to_entity.len(), 0, "not empty");
        let deserialize_watch = BasicWatch::start();
        self.xy_to_entity.extend(
            big_xy_bytes
                .into_iter()
                .array_chunks::<USIZE_BYTES>()
                .map(usize::from_ne_bytes),
        );
        trace!("Deserialized xy in {}", deserialize_watch);

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

    pub fn map_xy_to_vec<const MAPPED_SIZE: usize>(
        &self,
        mapper: impl Fn(&E) -> [u8; MAPPED_SIZE],
    ) -> Vec<u8> {
        match 1 {
            0 => self._map_xy_to_vec_targeted(mapper),
            1 => self._map_xy_to_vec_inlined(mapper),
            _ => panic!("unknown"),
        }
    }

    fn _map_xy_to_vec_inlined<const INLINED_MAPPED_SIZE: usize>(
        &self,
        mapper: impl Fn(&E) -> [u8; INLINED_MAPPED_SIZE],
    ) -> Vec<u8> {
        let result: Vec<u8> = self
            .xy_to_entity
            .iter()
            .map(|index| &self.entities[*index])
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

    fn _map_xy_to_vec_targeted<const TARGETED_MAPPED_SIZE: usize>(
        &self,
        mapper: impl Fn(&E) -> [u8; TARGETED_MAPPED_SIZE],
    ) -> Vec<u8> {
        let mut image = vec![0u8; self.xy_to_entity.len() * TARGETED_MAPPED_SIZE];
        trace!("mapping {} total {}", TARGETED_MAPPED_SIZE, image.len());
        for entity in &self.entities {
            for entity_pos in entity.get_xy() {
                let index = self.xy_to_index(entity_pos.x, entity_pos.y) * TARGETED_MAPPED_SIZE;
                let color = mapper(entity);
                if index + TARGETED_MAPPED_SIZE > image.len() {
                    trace!("overflowing for index {}", index);
                }
                // todo recheck this for >1
                if TARGETED_MAPPED_SIZE == 1 {
                    image[index] = color[0];
                } else {
                    image[index..(index + TARGETED_MAPPED_SIZE)].copy_from_slice(&color);
                }
            }
        }
        trace!(
            "mapped xy from {} to {}",
            self.xy_to_entity.len(),
            image.len()
        );
        image
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
    pub fn pixel_map_xy_to_cv(&self, filter: Option<Pixel>) -> Mat {
        // let mapper = if let Some(filter) = filter {
        //     move |e: &VPixel| {
        //         if e.pixel() == &filter {
        //             [e.pixel().to_owned() as u8]
        //         } else {
        //             [0]
        //         }
        //     }
        // } else {
        //     move |e: &VPixel| [e.pixel().to_owned() as u8]
        // };
        let output = self.map_xy_to_vec(|e| {
            if let Some(filter) = &filter {
                if e.pixel() == filter {
                    [e.pixel().to_owned() as u8]
                } else {
                    [0]
                }
            } else {
                [e.pixel().to_owned() as u8]
            }
        });
        let side_length = self.diameter();
        Mat::from_slice_rows_cols(&output, side_length, side_length).unwrap()
    }
}
