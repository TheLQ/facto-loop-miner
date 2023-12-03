use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use crate::surfacev::vpoint::VPoint;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace};

/// A map of background pixels (eg resources, water) and the large entities on top
///
/// Entity Position is i32 relative to top left (3x3 entity has start=0,0) for simpler math.
/// Converted from Factorio/Lua style of f32 relative to center (3x3 entity has start=1.5,1.5).
#[derive(Serialize, Deserialize)]
pub struct VSurface {
    pixels: VEntityBuffer<VPixel>,
    entities: VEntityBuffer<VEntity>,
}

impl VSurface {
    //<editor-fold desc="io load">

    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityBuffer::new(radius),
            entities: VEntityBuffer::new(radius),
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
        let surface = simd_json::serde::from_reader(BufReader::new(file)).map_err(|e| {
            VError::SimdJsonFail {
                e,
                path: path.into_boxed_path(),
                backtrace: Backtrace::capture(),
            }
        })?;
        Ok(surface)
    }

    fn load_entity_buffers(&mut self, out_dir: &Path) -> VResult<()> {
        let pixel_path = &path_pixel_buffer(out_dir);
        debug!("loading pixel buffer from {}", pixel_path.display());
        self.pixels.load_xy_file(pixel_path)?;

        let entity_path = &path_entity_buffer(out_dir);
        debug!("loading entity buffer from {}", entity_path.display());
        self.entities.load_xy_file(entity_path)?;

        Ok(())
    }

    pub fn load_from_last_step(params: &StepParams) -> VResult<Self> {
        Self::load(params.previous_step_dir())
    }

    //</editor-fold>

    //<editor-fold desc="io save">

    pub fn save(&self, out_dir: &Path) -> VResult<()> {
        self.save_state(out_dir)?;
        self.save_pixel_img_colorized(out_dir)?;
        self.save_entity_buffers(out_dir)?;
        Ok(())
    }

    fn save_state(&self, out_dir: &Path) -> VResult<()> {
        let state_path = out_dir.join("vsurface-state.json");
        debug!("Saving state JSON to {}", state_path.display());
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(state_path.clone())
            .map_err(VError::io_error(&state_path))?;
        let writer = BufWriter::new(file);
        simd_json::to_writer(writer, self).map_err(|e| VError::SimdJsonFail {
            e,
            path: state_path.into_boxed_path(),
            backtrace: Backtrace::capture(),
        })?;

        Ok(())
    }

    fn save_pixel_img_colorized(&self, out_dir: &Path) -> VResult<()> {
        let build_watch = BasicWatch::start();
        let pixel_map_path = out_dir.join("pixel-map.png");
        debug!("Saving RGB dump image to {}", pixel_map_path.display());
        let entities = self.pixels.new_xy_entity_array();
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
        let pixel_path = &path_pixel_buffer(out_dir);
        debug!("writing pixel buffer to {}", pixel_path.display());
        self.pixels.save_xy_file(pixel_path)?;

        let entity_path = &path_entity_buffer(out_dir);
        debug!("writing entity buffer to {}", entity_path.display());
        self.entities.save_xy_file(entity_path)?;

        Ok(())
    }

    //</editor-fold>

    pub fn set_pixel(&mut self, start: VPoint, pixel: Pixel) -> VResult<()> {
        self.pixels.add(VPixel { start, pixel })?;
        Ok(())
    }

    pub fn crop(&mut self, new_radius: u32) {
        info!("Crop from {} to {}", self.entities.radius(), new_radius);
        self.entities.crop(new_radius);
        self.pixels.crop(new_radius);
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

fn path_state(out_dir: &Path) -> PathBuf {
    out_dir.join("vsurface-state.json")
}

fn path_pixel_buffer(out_dir: &Path) -> PathBuf {
    out_dir.join("pixel-buffer.dat")
}

fn path_entity_buffer(out_dir: &Path) -> PathBuf {
    out_dir.join("entity-buffer.dat")
}

//</editor-fold>

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VPixel {
    start: VPoint,
    pixel: Pixel,
}

impl VEntityXY for VPixel {
    fn get_xy(&self) -> Vec<VPoint> {
        vec![self.start]
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VEntity {
    start: VPoint,
    points: Vec<VPoint>,
}

impl VEntityXY for VEntity {
    fn get_xy(&self) -> Vec<VPoint> {
        self.points.clone()
    }
}
