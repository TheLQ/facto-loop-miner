use crate::simd_diff::SurfaceDiff;
use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
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
        let load_time = BasicWatch::start();
        let mut surface = Self::load_state(out_dir)?;
        surface.load_entity_buffers(out_dir)?;
        info!(
            "Loaded VSurface from {} in {}",
            out_dir.display(),
            load_time
        );
        Ok(surface)
    }

    fn load_state(out_dir: &Path) -> VResult<Self> {
        let path = path_state(out_dir);
        let file = File::open(&path).map_err(VError::io_error(&path))?;
        let load_watch = BasicWatch::start();
        let surface = simd_json::serde::from_reader(BufReader::new(file))
            .map_err(VError::simd_json(&path))?;
        info!(
            "Read and loaded surface state from {} in {}",
            path.display(),
            load_watch
        );
        Ok(surface)
    }

    fn load_entity_buffers(&mut self, out_dir: &Path) -> VResult<()> {
        let pixel_path = &path_pixel_xy_indexes(out_dir);
        debug!("loading pixel buffer from {}", pixel_path.display());
        self.pixels.load_xy_file(pixel_path)?;

        let entity_path = &path_entity_xy_indexes(out_dir);
        debug!("loading entity buffer from {}", entity_path.display());
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
        info!("saving {}", self);
        self.save_state(out_dir)?;
        self.save_pixel_img_colorized(out_dir)?;
        self.save_entity_buffers(out_dir)?;
        Ok(())
    }

    fn save_state(&self, out_dir: &Path) -> VResult<()> {
        let state_path = out_dir.join("vsurface-state.json");
        let json_watch = BasicWatch::start();
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(state_path.clone())
            .map_err(VError::io_error(&state_path))?;
        let writer = BufWriter::new(file);
        simd_json::to_writer(writer, self).map_err(VError::simd_json(&state_path))?;
        debug!(
            "Saving state JSON to {} in {}",
            state_path.display(),
            json_watch
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
        debug!("writing pixel buffer to {}", pixel_path.display());
        self.pixels.save_xy_file(&pixel_path)?;

        let entity_path = path_entity_xy_indexes(out_dir);
        debug!("writing entity buffer to {}", entity_path.display());
        self.entities.save_xy_file(&entity_path)?;

        Ok(())
    }

    pub fn to_pixel_cv_image(&self, filter: Option<Pixel>) -> Mat {
        self.pixels.pixel_map_xy_to_cv(filter)
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
}

impl Display for VSurface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VSurface pixels {{ {} }} entities {{ {} }} patches {{ {} }}",
            self.pixels,
            self.entities,
            self.patches.len()
        )
    }
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
