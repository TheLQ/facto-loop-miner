use crate::admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity;
use crate::navigator::mori::Rail;
use crate::simd_diff::SurfaceDiff;
use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use crate::surfacev::varea::VArea;
use crate::surfacev::ventity_map::{VEntityMap, VEntityXY};
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_io::{read_entire_file, write_entire_file};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::JoinHandle;
use tracing::{debug, info, trace};

/// A map of background pixels (eg resources, water) and the large entities on top
///
/// Entity Position is i32 relative to top left (3x3 entity has start=0,0) for simpler math.
/// Converted from Factorio style of f32 relative to center (3x3 entity has start=1.5,1.5).
#[derive(Serialize, Deserialize)]
pub struct VSurface {
    pixels: VEntityMap<VPixel>,
    entities: VEntityMap<VEntity>,
    patches: Vec<VPatch>,
    #[serde(default)]
    place_rail: Vec<Rail>,
}

impl VSurface {
    //<editor-fold desc="io load">

    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityMap::new(radius),
            entities: VEntityMap::new(radius),
            patches: Vec::new(),
            place_rail: Vec::new(),
        }
    }

    pub fn load(out_dir: &Path) -> VResult<Self> {
        info!("+++ Loading VSurface from {}", out_dir.display());
        let load_time = BasicWatch::start();

        let surface_out_dir = out_dir.to_path_buf();
        let surface_thread = thread::spawn(move || Self::load_state(&surface_out_dir));
        let (pixel_thread, entity_thread) = Self::new(1).load_entity_buffers(out_dir);

        let mut new_surface = surface_thread.join().expect("surface join failed")?;

        new_surface
            .entities
            .load_xy_from_other(entity_thread.join().expect("entity thread failed")?);
        new_surface
            .pixels
            .load_xy_from_other(pixel_thread.join().expect("pixel thread failed")?);

        info!("Loaded {}", new_surface);
        new_surface.log_pixel_stats("vsurface load");
        info!("+++ Loaded in {} from {}", load_time, out_dir.display());
        Ok(new_surface)
    }

    fn load_state(out_dir: &Path) -> VResult<Self> {
        match 1 {
            0 => Self::_load_state_sequential(out_dir),
            1 => Self::_load_state_reader(out_dir),
            _ => panic!(),
        }
    }

    fn _load_state_sequential(out_dir: &Path) -> VResult<Self> {
        let mut read_watch = BasicWatch::start();
        let path = path_state(out_dir);
        let mut data = read_entire_file(&path, true)?;
        read_watch.stop();

        let load_watch = BasicWatch::start();
        let surface = simd_json::serde::from_slice(&mut data).map_err(VError::simd_json(&path))?;
        info!(
            "Loading state JSON read {} deserialize {} from {}",
            read_watch,
            load_watch,
            path.display(),
        );
        Ok(surface)
    }

    fn _load_state_reader(out_dir: &Path) -> VResult<Self> {
        trace!("start state thread");
        let total_watch = BasicWatch::start();
        let path = path_state(out_dir);
        let reader = BufReader::new(File::open(&path).map_err(VError::io_error(&path))?);

        let surface = simd_json::serde::from_reader(reader).map_err(VError::simd_json(&path))?;
        info!(
            "Loading state JSON in {} from {}",
            total_watch,
            path.display(),
        );
        Ok(surface)
    }

    #[allow(clippy::type_complexity)]
    fn load_entity_buffers(
        &mut self,
        out_dir: &Path,
    ) -> (
        JoinHandle<VResult<VEntityMap<VPixel>>>,
        JoinHandle<VResult<VEntityMap<VEntity>>>,
    ) {
        let out_dir_buf = out_dir.to_path_buf();
        let pixel_thread = thread::Builder::new()
            .name("pixel-loader".to_string())
            .spawn(move || {
                trace!("start pixel thread");
                let pixel_path = &path_pixel_xy_indexes(&out_dir_buf);
                let mut buffer = VEntityMap::<VPixel>::new(0);
                buffer.load_xy_file(pixel_path).map(|_| buffer)
            })
            .unwrap();

        let out_dir_buf = out_dir.to_path_buf();
        let entity_thread = thread::Builder::new()
            .name("entity-loader".to_string())
            .spawn(move || {
                trace!("start entity thread");
                let entity_path = &path_entity_xy_indexes(&out_dir_buf);
                let mut buffer = VEntityMap::<VEntity>::new(0);
                buffer.load_xy_file(entity_path).map(|_| buffer)
            })
            .unwrap();

        (pixel_thread, entity_thread)
    }

    pub fn load_from_last_step(params: &StepParams) -> VResult<Self> {
        Self::load(params.previous_step_dir())
    }

    pub fn path_pixel_buffer_from_last_step(params: &StepParams) -> PathBuf {
        path_pixel_xy_indexes(params.previous_step_dir())
    }

    //</editor-fold>

    //<editor-fold desc="io save">

    pub fn save(&self, out_dir: &Path) -> VResult<()> {
        info!("+++ Saving to {} {}", out_dir.display(), self);
        self.log_pixel_stats("vsurface save");
        let total_save_watch = BasicWatch::start();
        self.save_state(out_dir)?;
        self.save_pixel_img_colorized(out_dir)?;
        self.save_entity_buffers(out_dir)?;
        info!("+++ Saved in {} to {}", total_save_watch, out_dir.display());
        Ok(())
    }

    fn save_state(&self, out_dir: &Path) -> VResult<()> {
        let state_path = out_dir.join("vsurface-state.json");

        let mut serialize_watch = BasicWatch::start();
        let data = simd_json::to_vec(self).map_err(VError::simd_json(&state_path))?;
        serialize_watch.stop();

        let save_watch = BasicWatch::start();
        write_entire_file(&state_path, &data)?;

        debug!(
            "Saving state JSON serialize {} save {} to {}",
            serialize_watch,
            save_watch,
            state_path.display(),
        );

        Ok(())
    }

    fn save_pixel_img_colorized(&self, out_dir: &Path) -> VResult<()> {
        let build_watch = BasicWatch::start();
        let pixel_map_path = out_dir.join("pixel-map.png");
        debug!("Saving RGB dump image to {}", pixel_map_path.display());

        let entities = self.pixels.iter_xy_pixels();
        // trace!("built entity array of {}", entities.len());
        let mut output: Vec<u8> = vec![0; self.pixels.xy_array_length_from_radius() * 3];
        for (i, pixel) in entities.enumerate() {
            let color = &pixel.color();
            let start = i * color.len();
            output[start] = color[2];
            output[start + 1] = color[1];
            output[start + 2] = color[0];
        }
        trace!(
            "built entity array of {} in {}",
            output.len().to_formatted_string(&LOCALE),
            build_watch
        );

        // &out_dir.join(format!("{}full.png", name_prefix))
        let size = self.pixels.diameter() as u32;
        save_png(&pixel_map_path, &output, size, size);
        Ok(())
    }

    fn save_entity_buffers(&self, out_dir: &Path) -> VResult<()> {
        let pixel_path = path_pixel_xy_indexes(out_dir);
        self.pixels.save_xy_file(&pixel_path)?;

        let entity_path = path_entity_xy_indexes(out_dir);
        self.entities.save_xy_file(&entity_path)?;

        Ok(())
    }
    //</editor-fold>

    pub fn to_pixel_cv_image(&self, filter: Option<Pixel>) -> Mat {
        self.pixels.map_pixel_xy_to_cv(filter)
    }

    pub fn get_radius(&self) -> u32 {
        self.pixels.radius()
    }

    pub fn get_radius_i32(&self) -> i32 {
        self.pixels.radius() as i32
    }

    pub fn get_diameter(&self) -> usize {
        self.entities.diameter()
    }

    pub fn get_pixel(&self, point: &VPoint) -> Pixel {
        match self.pixels.get_entity_by_point(point) {
            Some(e) => e.pixel,
            None => Pixel::Empty,
        }
    }

    pub fn set_pixel(&mut self, start: VPoint, pixel: Pixel) -> VResult<()> {
        self.pixels.add(VPixel {
            starts: [start].to_vec(),
            pixel,
        })?;
        Ok(())
    }

    pub fn add_patches(&mut self, patches: &[VPatch]) {
        self.patches.extend_from_slice(patches)
    }

    pub fn get_patches_slice(&self) -> &[VPatch] {
        &self.patches
    }

    // pub fn get_xy_and_indexes_in_area(&self, area: &VArea) -> Vec<(VPoint, usize, Pixel)> {
    //     self.pixels.get_xy_and_indexes_in_area(area)
    // }

    pub fn is_xy_out_of_bounds(&self, x: i32, y: i32) -> bool {
        self.pixels.is_xy_out_of_bounds(x, y)
    }

    pub fn is_point_out_of_bounds(&self, point: &VPoint) -> bool {
        self.pixels.is_point_out_of_bounds(point)
    }

    pub fn is_points_free(&self, points: &[VPoint]) -> bool {
        self.pixels.is_points_free(points)
    }

    pub fn crop(&mut self, new_radius: u32) {
        info!("Crop from {} to {}", self.entities.radius(), new_radius);
        self.entities.crop(new_radius);
        self.pixels.crop(new_radius);
    }

    pub fn remove_patches_within_radius(&mut self, radius: u32) {
        let mut removed_points: Vec<VPoint> = Vec::new();
        let mut patches_to_remove = Vec::new();
        for (patch_index, patch) in self.patches.iter().enumerate() {
            if !patch.area.start.is_within_center_radius(radius) {
                // trace!("asdf {:?}\tfor {:?}", patch.area.start, patch.resource);
                continue;
            }
            // trace!("hello {:?}", patch);
            removed_points.extend_from_slice(&patch.pixel_indexes);
            // for index in &patch.pixel_indexes {
            // let pixel = self.pixels.get_entity_by_index(*index);
            // removed_points.push(*index);
            // }

            patches_to_remove.push(patch_index);
        }
        info!(
            "removing {} patches with {} entities within {} radius",
            patches_to_remove.len(),
            removed_points.len(),
            radius
        );
        self.pixels.remove_positions(&removed_points);

        patches_to_remove.reverse();
        for patch_index in patches_to_remove {
            self.patches.remove(patch_index);
        }
    }

    pub fn to_surface_diff(&self) -> SurfaceDiff {
        SurfaceDiff::from_surface(self)
    }

    pub fn log_pixel_stats(&self, debug_message: &str) {
        let mut metrics = FastMetrics::new(format!("log_pixel_stats XY {}", debug_message));
        for pixel in self.pixels.iter_xy_pixels() {
            metrics.increment(FastMetric::VSurface_Pixel(*pixel));
        }
        metrics.log_final();

        let mut metrics = FastMetrics::new(format!("log_pixel_stats Entities {}", debug_message));
        for entity in self.pixels.iter_entities() {
            metrics.increment(FastMetric::VSurface_Pixel(*entity.pixel()));
        }
        metrics.log_final();
    }

    pub fn draw_debug_varea_square(&mut self, area: &VArea) {
        let border = 10;
        for x in (area.start.x() - border)..(area.end_x_exclusive() + border) {
            for y in (area.start.y() - border)..(area.end_y_exclusive() + border) {
                let cur = VPoint::new(x, y);
                if self.pixels.is_point_out_of_bounds(&cur) {
                    continue;
                }
                if !area.contains_point(&cur) {
                    if self.get_pixel(&cur) != Pixel::Empty {
                        self.set_pixel(cur, Pixel::EdgeWall).unwrap();
                    } else {
                        self.set_pixel(cur, Pixel::Highlighter).unwrap();
                    }
                }
            }
        }
    }

    pub fn draw_square(
        &mut self,
        start_x: i32,
        end_x_exclusive: i32,
        start_y: i32,
        end_y_exclusive: i32,
        empty_map: Pixel,
        existing_map: Option<Pixel>,
    ) {
        for x in start_x..end_x_exclusive {
            for y in start_y..end_y_exclusive {
                let cur = VPoint::new(x, y);
                if self.pixels.is_point_out_of_bounds(&cur) {
                    continue;
                }

                let pixel_to_set = if self.get_pixel(&cur) != Pixel::Empty {
                    if let Some(existing_map) = existing_map {
                        existing_map
                    } else {
                        empty_map
                    }
                } else {
                    empty_map
                };
                self.set_pixel(cur, pixel_to_set).unwrap();
            }
        }
    }

    pub fn add_rail(&mut self, mut rails: Vec<Rail>) {
        self.place_rail.append(&mut rails)
    }

    pub fn get_rail(&self) -> &[Rail] {
        &self.place_rail
    }

    pub fn dump_pixels_xy(&self) -> impl Iterator<Item = &Pixel> {
        self.pixels.iter_xy_pixels()
    }

    #[cfg(test)]
    pub fn test_global_area(&self) -> VArea {
        let radius = self.get_radius_i32();
        VArea::from_arbitrary_points_pair(
            &VPoint::new(-radius, -radius),
            &VPoint::new(radius, radius),
        )
    }

    // pub fn draw_debug_square(&mut self, point: &VPoint) {
    //     let center_entity = VPixel {
    //         pixel: Pixel::EdgeWall,
    //         starts: vec![*point],
    //     };
    //     let mut background_entitiy = VPixel {
    //         pixel: Pixel::EdgeWall,
    //         starts: Vec::new(),
    //     };
    //     let (background_index, background_points) =
    //         self.pixels
    //             .draw_debug_square(point, center_entity, background_entitiy);
    //     self.pixels
    //         .get_entity_by_index_mut(background_index)
    //         .starts
    //         .extend(background_points.into_iter())
    // }
}

impl Display for VSurface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VSurface pixels {{ {} }} entities {{ {} }} patches {{ {} }}",
            self.pixels,
            self.entities,
            display_patches(&self.patches)
        )
    }
}

fn display_patches(patches: &Vec<VPatch>) -> String {
    let mut map: HashMap<Pixel, usize> = HashMap::new();
    for patch in patches {
        let current_count = map.get(&patch.resource).unwrap_or(&0);
        map.insert(patch.resource, current_count + 1);
    }

    let mut result = format!("total {}", patches.len());
    for (resource, count) in map {
        result = format!("{} {:?} {}", result, resource, count);
    }
    result
}

//<editor-fold desc="io common">

fn save_png(path: &Path, rgb: &[u8], width: u32, height: u32) {
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(&file);

    let encoder = PngEncoder::new(writer);
    encoder
        .write_image(rgb, width, height, ExtendedColorType::Rgb8)
        .unwrap();
    let size = file.metadata().unwrap().len();
    debug!(
        "Saved {} byte image to {}",
        size.to_formatted_string(&LOCALE),
        path.display(),
    );
}

fn path_pixel_xy_indexes(out_dir: &Path) -> PathBuf {
    out_dir.join("pixel-xy-indexes.dat")
}

fn path_entity_xy_indexes(out_dir: &Path) -> PathBuf {
    out_dir.join("entity-xy-indexes.dat")
}

fn path_state(out_dir: &Path) -> PathBuf {
    out_dir.join("vsurface-state.json")
}

//</editor-fold>

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VPixel {
    starts: Vec<VPoint>,
    pixel: Pixel,
}

impl VEntityXY for VPixel {
    fn get_xy(&self) -> &[VPoint] {
        &self.starts
    }
}

impl VPixel {
    pub fn pixel(&self) -> &Pixel {
        &self.pixel
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VEntity {
    start: VPoint,
    points: Vec<VPoint>,
}

impl VEntityXY for VEntity {
    fn get_xy(&self) -> &[VPoint] {
        &self.points
    }
}

pub trait PlaceableEntity: Serialize + for<'a> Deserialize<'a> {
    fn place_lua_command(&self) -> Vec<FacSurfaceCreateEntity>;
}
