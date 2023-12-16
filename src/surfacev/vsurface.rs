use crate::simd_diff::SurfaceDiff;
use crate::state::machine::StepParams;
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::util::duration::BasicWatch;
use crate::util::io::{read_entire_file, write_entire_file};
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace};

/// A map of background pixels (eg resources, water) and the large entities on top
///
/// Entity Position is i32 relative to top left (3x3 entity has start=0,0) for simpler math.
/// Converted from Factorio style of f32 relative to center (3x3 entity has start=1.5,1.5).
#[derive(Serialize, Deserialize)]
pub struct VSurface {
    pixels: VEntityBuffer<VPixel>,
    entities: VEntityBuffer<VEntity>,
    patches: Vec<VPatch>,
}

impl VSurface {
    //<editor-fold desc="io load">

    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityBuffer::new(radius),
            entities: VEntityBuffer::new(radius),
            patches: Vec::new(),
        }
    }

    pub fn load(out_dir: &Path) -> VResult<Self> {
        info!("+++ Loading VSurface from {}", out_dir.display());
        let load_time = BasicWatch::start();
        let mut surface = Self::load_state(out_dir)?;
        surface.load_entity_buffers(out_dir)?;
        info!("Loaded {}", surface);
        info!("+++ Loaded in {} from {}", load_time, out_dir.display());
        Ok(surface)
    }

    fn load_state(out_dir: &Path) -> VResult<Self> {
        let mut read_watch = BasicWatch::start();
        let path = path_state(out_dir);
        let mut data = read_entire_file(&path)?;
        read_watch.stop();

        let load_watch = BasicWatch::start();
        let surface = simd_json::serde::from_slice(&mut data).map_err(VError::simd_json(&path))?;
        info!(
            "Loading state JSON read {} serialize {} from {}",
            read_watch,
            load_watch,
            path.display(),
        );
        Ok(surface)
    }

    fn load_entity_buffers(&mut self, out_dir: &Path) -> VResult<()> {
        let pixel_path = &path_pixel_xy_indexes(out_dir);
        self.pixels.load_xy_file(pixel_path)?;

        let entity_path = &path_entity_xy_indexes(out_dir);
        self.entities.load_xy_file(entity_path)?;

        Ok(())
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
        self.log_pixel_stats();
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
        let entities = self.pixels.iter_xy_entities();
        // trace!("built entity array of {}", entities.len());
        let mut output: Vec<u8> = vec![0; self.pixels.xy_array_length_from_radius() * 3];
        for (i, pixel) in entities.enumerate() {
            let color = &pixel.pixel.color();
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

    pub fn to_pixel_cv_image(&self, filter: Option<Pixel>) -> Mat {
        self.pixels.map_pixel_xy_to_cv(filter)
    }

    //</editor-fold>

    pub fn get_radius(&self) -> u32 {
        self.pixels.radius()
    }

    pub fn get_pixel(&self, point: &VPoint) -> Option<Pixel> {
        self.pixels.get_entity_by_point(point).map(|e| e.pixel)
    }

    pub fn set_pixel(&mut self, start: VPoint, pixel: Pixel) -> VResult<()> {
        self.pixels.add(VPixel {
            starts: [start].to_vec(),
            pixel,
        })?;
        Ok(())
    }

    pub fn add_patches(&mut self, patches: Vec<VPatch>) {
        self.patches.extend(patches)
    }

    pub fn get_patches_iter(&self) -> impl IntoIterator<Item = &VPatch> {
        self.patches.iter()
    }

    pub fn get_xy_in_patch(&self, patch: &VPatch) -> Vec<VPoint> {
        patch
            .pixel_indexes
            .iter()
            .map(|v| self.pixels.get_entity_by_index(*v).starts[0])
            .collect()
    }

    pub fn crop(&mut self, new_radius: u32) {
        info!("Crop from {} to {}", self.entities.radius(), new_radius);
        self.entities.crop(new_radius);
        self.pixels.crop(new_radius);
    }

    pub fn xy_side_length(&self) -> usize {
        self.entities.diameter()
    }

    pub fn remove_patches_within_radius(&mut self, radius: u32) {
        let mut removed_points: Vec<usize> = Vec::new();
        for patch in &self.patches {
            if !patch.area.start.is_within_center_radius(radius * 2) {
                continue;
            }
            for index in &patch.pixel_indexes {
                let pixel = self.pixels.get_entity_by_index(*index);
                if pixel.get_xy()[0].is_within_center_radius(radius) {
                    removed_points.push(*index);
                }
            }
        }
        info!(
            "removing {} patches within {} radius",
            removed_points.len(),
            radius
        );
        self.pixels.remove_positions(removed_points);
    }

    pub fn to_surface_diff(&self) -> SurfaceDiff {
        SurfaceDiff::from_surface(self)
    }

    pub fn log_pixel_stats(&self) {
        let mut metrics = Metrics::new("vsurface-pixel");
        for pixel in self.pixels.iter_xy_entities() {
            metrics.increment(pixel.pixel.as_ref());
        }
        metrics.log_final();
    }
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
        .write_image(rgb, width, height, ColorType::Rgb8)
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
