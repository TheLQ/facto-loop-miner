use crate::surface::pixel::Pixel;
use crate::surfacev::err::{VError, VResult};
use crate::surfacev::ventity_buffer::{VEntityBuffer, VEntityXY};
use crate::surfacev::vpoint::VPoint;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;
use tracing::debug;

/// A collection of background/pixels and the large entities on top
///
/// Position is i32 relative to the bottom right (3x3 entity has start=0,0)
/// Converted from Factorio/Lua style of f32 relative to center (3x3 entity has start=1.5,1.5).
/// i32 is much saner math.
#[derive(Serialize, Deserialize)]
pub struct VSurface {
    pixels: VEntityBuffer<VPixel>,
    entities: VEntityBuffer<VEntity>,
}

impl VSurface {
    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityBuffer::new(radius),
            entities: VEntityBuffer::new(radius),
        }
    }

    pub fn set_pixel(&mut self, start: VPoint, pixel: Pixel) -> VResult<()> {
        self.pixels.add(VPixel { start, pixel })?;
        Ok(())
    }

    pub fn save(&self, out_dir: &Path) -> VResult<()> {
        if !out_dir.is_dir() {
            Err(VError::NotADirectory {
                path: format!("dir does not exist {}", out_dir.display()),
                backtrace: Backtrace::force_capture(),
            })
        } else {
            self.save_state(out_dir)?;
            self.save_pixel_img_colorized(out_dir)?;

            // self.save_raw(out_dir);
            // self.save_colorized(out_dir, NAME_PREFIX);

            Ok(())
        }
    }

    fn save_state(&self, out_dir: &Path) -> VResult<()> {
        let state_path = out_dir.join("vsurface-state.json");
        debug!("Saving state JSON to {}", state_path.display());
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(state_path.clone())
            .map_err(|e| VError::IoError {
                e,
                path: state_path.to_string_lossy().to_string(),
                backtrace: Backtrace::capture(),
            })?;
        let writer = BufWriter::new(file);
        simd_json::to_writer(writer, self).map_err(|e| VError::SimdJsonFail {
            e,
            backtrace: Backtrace::force_capture(),
        })?;

        Ok(())
    }

    fn save_pixel_img_colorized(&self, out_dir: &Path) -> VResult<()> {
        let pixel_map_path = out_dir.join("pixel-map.png");
        debug!("Saving RGB dump image to {}", pixel_map_path.display());
        let entities = self.pixels.new_entity_array();
        let mut output: Vec<u8> = vec![0; entities.len() * 3];
        for (i, pixel) in entities.iter().enumerate() {
            let color = &pixel.pixel.color();
            let start = i * color.len();
            output[start] = color[0];
            output[start + 1] = color[1];
            output[start + 2] = color[2];
        }

        // &out_dir.join(format!("{}full.png", name_prefix))
        let size = self.entities.diameter();
        save_png(
            &pixel_map_path,
            &output,
            self.pixels.diameter() as u32,
            self.pixels.diameter() as u32,
        );
        Ok(())
    }
}

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