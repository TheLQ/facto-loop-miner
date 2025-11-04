use crate::opencv::{GeneratedMat, draw_text_cv, draw_text_size, mat_into_points};
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{CoreConvertPathResult, VResult};
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use crate::surfacev::ventity_map::{VEntityMap, VMapChange};
use crate::surfacev::vsurface::VPixel;
use colorgrad::Gradient;
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ONE, VPoint};
use facto_loop_miner_fac_engine::opencv_re::core::{CV_8U, Mat, MatTrait, Point, Scalar};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use num_format::ToFormattedString;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use tracing::{debug, info, trace};

impl VEntityMap<VPixel> {
    fn as_surface_pixel(&self) -> VSurfacePixel {
        VSurfacePixel { pixels: self }
    }
}

struct VSurfacePixelMut<'s> {
    pixels: &'s mut VEntityMap<VPixel>,
}

impl<'s> VSurfacePixelMut<'s> {
    fn get(&self) -> VSurfacePixel<'s> {
        VSurfacePixel::new(&self.pixels)
    }

    pub fn crop(&mut self, new_radius: u32) {
        let old_radius = self.get_radius();
        info!("Crop from {} to {}", old_radius, new_radius);
        assert!(old_radius > new_radius);
        // self.entities.crop(new_radius);
        self.pixels.crop(new_radius);
    }

    pub fn set_entity_replace(&mut self, pos: VPoint, expected: Pixel, new: Pixel) {
        let entity = self
            .pixels
            .get_entity_by_point_mut(&pos)
            .unwrap_or_else(|| panic!("must exist {pos}"));
        if entity.pixel == expected {
            entity.pixel = new;
        } else {
            panic!(
                "at {pos} expected {} found {}",
                expected.as_ref(),
                entity.pixel.as_ref()
            )
        }
    }

    #[must_use]
    pub fn change_square(&mut self, area: &VArea) -> VMapChange<'_, VPixel, Vec<VPoint>> {
        assert!(
            !self.pixels.is_point_out_of_bounds(&area.point_top_left()),
            "Area is out of bounds {area}",
        );
        assert!(
            !self
                .pixels
                .is_point_out_of_bounds(&area.point_bottom_right()),
            "Area is out of bounds {area}",
        );
        self.pixels.change(area.get_points())
    }

    #[must_use]
    pub fn change_pixels<I>(&mut self, positions: I) -> VMapChange<'_, VPixel, I>
    where
        I: IntoIterator<Item = VPoint>,
    {
        self.pixels.change(positions)
    }

    pub fn draw_text_at(&mut self, pos: VPoint, text: &str) {
        let watch = BasicWatch::start();

        let text_height = 25;
        let text_thickness = 3;
        let text_size = draw_text_size(text, text_height, text_thickness);
        // TIL: new_size/new_rows_cols will reuse allocations!
        let mut mat = unsafe { Mat::new_size(text_size.to_cv_size(), CV_8U).unwrap() };
        mat.set_scalar(Scalar::all(0.0)).unwrap();

        let color = u8::MAX;
        draw_text_cv(
            &mut mat,
            text,
            Point {
                x: 0,
                // draw_text_size adds thickness we must remove
                y: text_size.y() - text_thickness,
            },
            Scalar::all(color.into()),
            text_height,
            text_thickness,
        );
        // imwrite("out.png", &mat, &Vector::new()).unwrap();
        let new_points = mat_into_points(mat, color, pos)
            .into_iter()
            .filter(|v| !self.is_point_out_of_bounds(v))
            .collect();
        trace!("Text \"{text}\" at {pos} generated in {watch}");

        // self.change_square(&VArea::from_arbitrary_points_pair(pos, pos + text_size))
        //     .stomp(Pixel::EdgeWall);

        // let gen_area = VArea::from_arbitrary_points(&new_points);
        // trace!("from area {gen_area}");

        // let watch = BasicWatch::start();
        // let new_points_len = new_points.len();
        self.change_pixels(new_points).stomp(Pixel::Highlighter);
        // trace!("set {new_points_len} points in {watch}");
    }
}

struct VSurfacePixel<'s> {
    pixels: &'s VEntityMap<VPixel>,
}

impl<'s> VSurfacePixel<'s> {
    pub fn new(pixels: &'s VEntityMap<VPixel>) -> Self {
        Self { pixels }
    }

    #[must_use]
    pub fn paint_pixel_graduated(&self, compressed: HashMap<VPoint, u32>) -> SurfacePainting {
        assert!(!compressed.is_empty());
        let build_watch = BasicWatch::start();

        let max_count = *compressed.values().max().unwrap() as f32;
        let index_to_compressed: HashMap<usize, (VPoint, f32)> = compressed
            .into_iter()
            //todo: filter shouldn't be needed
            .filter(|(pos, _)| pos.is_within_center_radius(self.get_radius()))
            .flat_map(|(pos, count)| {
                pos.get_entity_area_square(10)
                    .into_iter()
                    .map(move |v| (v, count))
            })
            .map(|(pos, count)| {
                (
                    self.pixels.point_to_index_unchecked(&pos),
                    (pos, count as f32 / max_count),
                )
            })
            .collect();
        let colorgrad = colorgrad::preset::spectral();

        let entities = self.pixels.iter_xy_pixels();
        let mut output: Vec<u8> = vec![0; self.pixels.xy_array_length_from_radius() * 4];
        for (i, pixel) in entities.enumerate() {
            let color = if let Some((_, count)) = index_to_compressed.get(&i) {
                colorgrad.at(*count).to_rgba8()
            } else {
                let raw = pixel.color();
                [raw[0], raw[1], raw[2], 0xFF]
            };
            let start = i * color.len();
            output[start..(start + 4)].copy_from_slice(&color);
        }

        let debug_description = format!(
            "graduated ({} in {})",
            output.len().to_formatted_string(&LOCALE),
            build_watch
        );
        SurfacePainting {
            output,
            diameter: self.pixels.diameter() as u32,
            color_type: ExtendedColorType::Rgba8,
            file_name: "pixel-map-grad.png",
            debug_description,
        }
    }

    /// https://github.com/woelper/oculante
    #[must_use]
    pub fn paint_pixel_colored_zoomed(&self) -> SurfacePainting {
        let crop_circle = VArea::from_arbitrary_points_pair(
            VPoint::new(0, 0),
            VPoint::new(self.get_radius_i32() - 1, self.get_radius_i32() - 1),
        );
        self.paint_pixel_colored(Some(crop_circle))
    }

    #[must_use]
    pub fn paint_pixel_colored_entire(&self) -> SurfacePainting {
        self.paint_pixel_colored(None)
    }

    #[must_use]
    fn paint_pixel_colored(&self, crop: Option<VArea>) -> SurfacePainting {
        const FILENAME: &str = "pixel-map.png";
        let build_watch = BasicWatch::start();
        if let Some(crop_circle) = crop {
            // let crop_circle: VArea = self
            //     .dummy_area_entire_surface()
            //     .normalize_within_radius(self.get_radius_i32() - 5);
            let crop_size = crop_circle.as_size() + VPOINT_ONE;
            assert_eq!(crop_size.x(), crop_size.y());
            let output_size = (crop_size.x() * crop_size.y() * 3) as usize;

            let entities = self.pixels.iter_xy_pixels();
            let mut output: Vec<u8> = Vec::with_capacity(output_size);
            // trace!(
            //     "area {crop_circle} size {} output {} width {}",
            //     output.len(),
            //     crop_size,
            //     output.len() / 3 / self.get_radius() as usize
            // );
            for (i, pixel) in entities.enumerate() {
                if !crop_circle.contains_point(&self.pixels.index_to_xy(i)) {
                    continue;
                }
                let color = &pixel.color();
                output.extend(color);
            }
            assert_eq!(output.len(), output_size);

            let debug_description = format!(
                "colored cropped ({} in {})",
                output.len().to_formatted_string(&LOCALE),
                build_watch
            );
            SurfacePainting {
                output,
                diameter: crop_size.x().try_into().unwrap(),
                color_type: ExtendedColorType::Rgb8,
                file_name: FILENAME,
                debug_description,
            }
        } else {
            let entities = self.pixels.iter_xy_pixels();
            // trace!("built entity array of {}", entities.len());
            let mut output: Vec<u8> = vec![0; self.pixels.xy_array_length_from_radius() * 3];
            for (i, pixel) in entities.enumerate() {
                let color = &pixel.color();
                let start = i * color.len();
                output[start..(start + 3)].copy_from_slice(color);
            }

            let debug_description = format!(
                "colored entire ({} in {})",
                output.len().to_formatted_string(&LOCALE),
                build_watch
            );
            SurfacePainting {
                output,
                diameter: self.get_radius() * 2,
                color_type: ExtendedColorType::Rgb8,
                file_name: FILENAME,
                debug_description,
            }
        }
    }

    pub fn dump_pixels_xy(&self) -> impl Iterator<Item = &Pixel> {
        self.pixels.iter_xy_pixels()
    }

    //</editor-fold>

    pub fn to_pixel_cv_image(&self, filter: Option<Pixel>) -> GeneratedMat {
        self.pixels.map_pixel_xy_to_cv(filter)
    }

    pub fn get_radius(&self) -> u32 {
        self.pixels.radius()
    }

    pub fn get_radius_i32(&self) -> i32 {
        self.pixels.radius() as i32
    }

    pub fn get_diameter(&self) -> usize {
        self.pixels.diameter()
    }

    pub fn point_top_left(&self) -> VPoint {
        let radius = self.get_radius_i32();
        VPoint::new(-radius, -radius)
    }

    pub fn point_bottom_right(&self) -> VPoint {
        let radius = self.get_radius_i32();
        VPoint::new(radius, radius)
    }

    pub fn get_pixel(&self, point: impl Borrow<VPoint>) -> Pixel {
        match self.pixels.get_entity_by_point(point.borrow()) {
            Some(e) => e.pixel,
            None => Pixel::Empty,
        }
    }

    pub fn get_pixels_all(&self) -> impl Iterator<Item = (VPoint, Pixel)> + '_ {
        self.pixels
            .iter_xy_entities_and_points()
            .map(|(point, maybe_vpixel)| {
                (
                    point,
                    maybe_vpixel
                        .map(|vpixel| vpixel.pixel)
                        .unwrap_or(Pixel::Empty),
                )
            })
    }

    pub fn is_point_out_of_bounds(&self, point: &VPoint) -> bool {
        self.pixels.is_point_out_of_bounds(point)
    }

    pub fn is_points_free_truncating(&self, points: &[VPoint]) -> bool {
        self.pixels.is_points_free_safe(points)
    }

    pub fn is_points_free_unchecked(&self, points: &[VPoint]) -> bool {
        self.pixels.is_points_free_unchecked_iter(points)
    }

    pub fn get_pixel_entity_id_at(&self, point: &VPoint) -> usize {
        self.pixels.get_entity_id_at(point)
    }

    pub fn log_pixel_stats(&self, debug_message: &str) {
        let mut metrics = FastMetrics::new(format!("log_pixel_counts Entities {}", debug_message));
        for entity in self.pixels.iter_xy_pixels() {
            metrics.increment(FastMetric::VSurface_Pixel(*entity));
        }
        metrics.log_final();
    }
}

/// Generated image with various save outputs
pub struct SurfacePainting {
    output: Vec<u8>,
    diameter: u32,
    color_type: ExtendedColorType,
    file_name: &'static str,
    debug_description: String,
}

impl SurfacePainting {
    fn encoder<W: Write>(writer: W) -> PngEncoder<W> {
        // For input 2000x2000 image:
        // Custom takes 0.121 seconds to save
        // Default takes 2.4 seconds to save
        PngEncoder::new_with_quality(writer, CompressionType::Fast, FilterType::NoFilter)
    }

    pub fn save_to_oculante(self) {
        let Self {
            output,
            diameter,
            color_type,
            file_name: _,
            debug_description,
        } = self;
        const ADDR: (&str, u16) = ("peko.g.xana.sh", 5689);
        // const ADDR: (&str, u16) = ("127.0.0.1", 5689);
        let address: SocketAddr = ADDR.to_socket_addrs().unwrap().next().unwrap();
        debug!("Painting {debug_description} to oculante {address}");
        let stream = TcpStream::connect(address).unwrap();

        Self::encoder(stream)
            .write_image(&output, diameter, diameter, color_type)
            .unwrap();
    }

    pub fn save_to_file(self, dir: &Path) -> VResult<()> {
        let Self {
            output,
            diameter,
            color_type,
            file_name,
            debug_description,
        } = self;

        let watch = BasicWatch::start();
        let path = dir.join(file_name);
        debug!("Painting {debug_description} to file {}", path.display());

        let file = File::create(&path).convert(&path)?;
        let writer = BufWriter::new(&file);
        Self::encoder(writer)
            .write_image(&output, diameter, diameter, color_type)
            .convert(&path)?;

        let size = file.metadata().convert(&path)?.len();
        debug!(
            "Saved {} byte image to {} in {}",
            size.to_formatted_string(&LOCALE),
            path.display(),
            watch
        );
        Ok(())
    }
}
