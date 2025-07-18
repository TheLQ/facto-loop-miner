use crate::opencv::GeneratedMat;
use crate::state::machine::StepParams;
use crate::state::tuneables::Tunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{CoreConvertPathResult, VResult};
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use crate::surfacev::mine::MinePath;
use crate::surfacev::ventity_map::{VEntityMap, VMapChange};
use crate::surfacev::vpatch::VPatch;
use colorgrad::Gradient;
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ONE, VPoint};
use facto_loop_miner_io::{read_entire_file, write_entire_file};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;
use std::thread::JoinHandle;
use tracing::{debug, info, trace};

/// A map of background pixels (eg resources, water) and the large entities on top
///
/// Entity Position is i32 relative to top left (3x3 entity has start=0,0) for simpler math.
/// Converted from Factorio style of f32 relative to center (3x3 entity has start=1.5,1.5).
#[derive(Serialize, Deserialize, Clone)]
pub struct VSurface {
    pixels: VEntityMap<VPixel>,
    // entities: VEntityMap<VEntity>,
    patches: Vec<VPatch>,
    #[serde(default)]
    rail_paths: Vec<MinePath>,
    #[serde(skip, default = "Tunables::new")]
    tunables: Tunables,
}

impl VSurface {
    //<editor-fold desc="io load">

    pub fn new(radius: u32) -> Self {
        VSurface {
            pixels: VEntityMap::new(radius),
            // entities: VEntityMap::new(radius),
            patches: Vec::new(),
            rail_paths: Vec::new(),
            tunables: Tunables::new(),
        }
    }

    pub fn load(out_dir: &Path) -> VResult<Self> {
        info!("+++ Loading VSurface from {}", out_dir.display());
        let load_time = BasicWatch::start();

        let surface_out_dir = out_dir.to_path_buf();
        let surface_thread = thread::spawn(move || Self::load_state(&surface_out_dir));
        // let (pixel_thread, entity_thread) = Self::new(1).load_entity_buffers(out_dir);
        let (pixel_thread,) = Self::new(1).load_entity_buffers(out_dir);

        let mut new_surface = surface_thread.join().expect("surface join failed")?;

        // new_surface
        //     .entities
        //     .load_xy_from_other(entity_thread.join().expect("entity thread failed")?);
        new_surface
            .pixels
            .load_xy_from_other(pixel_thread.join().expect("pixel thread failed")?);

        // todo: error check
        // new_surface.pixels.assert_no_empty_pixels();

        info!("Loaded {}", new_surface);
        new_surface.log_pixel_stats("vsurface load");
        info!("+++ Loaded in {} from {}", load_time, out_dir.display());
        Ok(new_surface)
    }

    // pub fn assert_no_empty(&self) {
    //     self.pixels.assert_no_empty_pixels();
    // }

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
        let mut data = read_entire_file(&path, true).convert(&path)?;
        read_watch.stop();

        let load_watch = BasicWatch::start();
        let surface = simd_json::serde::from_slice(&mut data).convert(&path)?;
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
        let reader = BufReader::new(File::open(&path).convert(&path)?);

        let surface = simd_json::serde::from_reader(reader).convert(&path)?;
        info!(
            "Loading state JSON in {} from {}",
            total_watch,
            path.display(),
        );
        Ok(surface)
    }

    #[allow(clippy::type_complexity)]
    // fn load_entity_buffers(
    //     &mut self,
    //     out_dir: &Path,
    // ) -> (
    //     JoinHandle<VResult<VEntityMap<VPixel>>>,
    //     JoinHandle<VResult<VEntityMap<VEntity>>>,
    // ) {
    fn load_entity_buffers(
        &mut self,
        out_dir: &Path,
    ) -> (JoinHandle<VResult<VEntityMap<VPixel>>>,) {
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

        // let out_dir_buf = out_dir.to_path_buf();
        // let entity_thread = thread::Builder::new()
        //     .name("entity-loader".to_string())
        //     .spawn(move || {
        //         trace!("start entity thread");
        //         let entity_path = &path_entity_xy_indexes(&out_dir_buf);
        //         let mut buffer = VEntityMap::<VEntity>::new(0);
        //         buffer.load_xy_file(entity_path).map(|_| buffer)
        //     })
        //     .unwrap();

        // (pixel_thread, entity_thread)
        (pixel_thread,)
    }

    pub fn load_from_last_step(params: &StepParams) -> VResult<Self> {
        Self::load(params.previous_step_dir())
    }

    pub fn path_pixel_buffer_from_last_step(params: &StepParams) -> PathBuf {
        path_pixel_xy_indexes(params.previous_step_dir())
    }

    pub fn load_clone_prep(&mut self) -> VResult<()> {
        self.pixels.load_clone_prep(&path_pixel_xy_indexes_clone())
    }

    //</editor-fold>

    //<editor-fold desc="io save">

    pub fn save(&self, out_dir: &Path) -> VResult<()> {
        info!("+++ Saving to {} {}", out_dir.display(), self);
        self.log_pixel_stats("vsurface save");
        let total_save_watch = BasicWatch::start();
        self.save_state(out_dir)?;

        self.paint_pixel_colored_entire().save_to_file(out_dir)?;
        self.save_entity_buffers(out_dir)?;
        self.save_tuning_parameters(out_dir)?;
        info!("+++ Saved in {} to {}", total_save_watch, out_dir.display());
        Ok(())
    }

    fn save_state(&self, out_dir: &Path) -> VResult<()> {
        let state_path = out_dir.join("vsurface-state.json");

        let mut serialize_watch = BasicWatch::start();
        let data = simd_json::to_vec(self).convert(&state_path)?;
        serialize_watch.stop();

        let save_watch = BasicWatch::start();
        write_entire_file(&state_path, &data).convert(&state_path)?;

        debug!(
            "Saving state JSON serialize {} save {} to {}",
            serialize_watch,
            save_watch,
            state_path.display(),
        );

        Ok(())
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

    fn save_entity_buffers(&self, out_dir: &Path) -> VResult<()> {
        let pixel_path = path_pixel_xy_indexes(out_dir);
        self.pixels.save_xy_file(&pixel_path)?;

        // let entity_path = path_entity_xy_indexes(out_dir);
        // self.entities.save_xy_file(&entity_path)?;

        Ok(())
    }

    /// Created after loosing so much run data
    fn save_tuning_parameters(&self, out_dir: &Path) -> VResult<()> {
        let tuning_path = out_dir.join("tuning-params.json");
        let output = simd_json::to_vec_pretty(&self.tunables).convert(&tuning_path)?;
        std::fs::write(&tuning_path, &output).convert(&tuning_path)?;

        Ok(())
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

    #[must_use]
    pub fn change_pixels<I>(&mut self, positions: I) -> VMapChange<'_, VPixel, I>
    where
        I: IntoIterator<Item = VPoint>,
    {
        self.pixels.change(positions)
    }

    pub fn add_patches(&mut self, patches: &[VPatch]) {
        self.patches.extend_from_slice(patches)
    }

    pub fn get_patches_slice(&self) -> &[VPatch] {
        &self.patches
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

    pub fn crop(&mut self, new_radius: u32) {
        info!("Crop from {} to {}", self.pixels.radius(), new_radius);
        // self.entities.crop(new_radius);
        self.pixels.crop(new_radius);
    }

    pub fn remove_patches_within_radius(&mut self, radius: u32) {
        let mut removed_points: Vec<VPoint> = Vec::new();
        let mut patches_to_remove = Vec::new();
        for (patch_index, patch) in self.patches.iter().enumerate() {
            if !patch.area.point_center().is_within_center_radius(radius) {
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
        self.pixels.change(removed_points).remove();

        patches_to_remove.reverse();
        for patch_index in patches_to_remove {
            self.patches.remove(patch_index);
        }
    }

    pub fn remove_patches_in_column(&mut self, radius: u32) {
        let mut removed_points: Vec<VPoint> = Vec::new();
        let mut patches_to_remove = Vec::new();
        let radius = radius as i32;
        for (patch_index, patch) in self.patches.iter().enumerate() {
            if (-radius..radius).contains(&patch.area.point_center().x()) {
                removed_points.extend_from_slice(&patch.pixel_indexes);
                patches_to_remove.push(patch_index);
            }
        }
        info!(
            "removing {} patches with {} entities within {} radius",
            patches_to_remove.len(),
            removed_points.len(),
            radius
        );
        self.pixels.change(removed_points).remove();

        patches_to_remove.reverse();
        for patch_index in patches_to_remove {
            self.patches.remove(patch_index);
        }
    }

    pub fn get_pixel_entity_id_at(&self, point: &VPoint) -> usize {
        self.pixels.get_entity_id_at(point)
    }

    // pub fn set_pixel_entity_swap(
    //     &mut self,
    //     entity_id: usize,
    //     mut points: Vec<VPoint>,
    //     overwrite_non_empty: bool,
    // ) -> RemovedEntity {
    //     self.pixels
    //         .set_entity_points_swap(entity_id, &mut points, overwrite_non_empty);
    //     RemovedEntity { entity_id, points }
    // }

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

    pub fn log_pixel_stats(&self, debug_message: &str) {
        // let mut metrics = FastMetrics::new(format!("log_pixel_stats Entities {}", debug_message));
        // for entity in self.pixels.iter_entities() {
        //     metrics.increment(FastMetric::VSurface_Pixel(*entity.pixel()));
        // }

        let mut metrics = FastMetrics::new(format!("log_pixel_counts Entities {}", debug_message));
        for entity in self.pixels.iter_xy_pixels() {
            metrics.increment(FastMetric::VSurface_Pixel(*entity));
        }
        metrics.log_final();
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

    pub fn add_mine_path(&mut self, mine_path: MinePath) {
        self.add_mine_path_with_pixel(mine_path, Pixel::Rail)
    }

    pub fn add_mine_path_with_pixel(&mut self, mine_path: MinePath, pixel: Pixel) {
        trace!(
            "{} {}",
            nu_ansi_term::Color::Red.paint("mine add"),
            mine_path.segment
        );
        let new_points = mine_path.total_area();
        self.change_pixels(new_points).stomp(pixel);

        // todo
        // // add markers for start points
        // let start_points: Vec<VPoint> = mine_path.links.iter().map(|v| v.start).collect_vec();
        // self.set_pixels(Pixel::EdgeWall, start_points)?;

        self.rail_paths.push(mine_path);
    }

    pub fn remove_mine_path_at(&mut self, index: usize) -> Option<(MinePath, Vec<VPoint>)> {
        let mine_path = self.rail_paths.remove(index);
        trace!(
            "{} at {index} total {} - {}",
            nu_ansi_term::Color::Red.paint("mine remove"),
            self.rail_paths.len(),
            mine_path.segment,
        );

        let removed_points = self.remove_mine_path_cleanup(&mine_path);
        Some((mine_path, removed_points))
    }

    pub fn remove_mine_path_pop(&mut self) -> Option<(MinePath, Vec<VPoint>)> {
        trace!(
            "{} pop total {}",
            nu_ansi_term::Color::Red.paint("mine remove"),
            self.rail_paths.len()
        );
        let mine_path = self.rail_paths.pop()?;
        let removed_points = self.remove_mine_path_cleanup(&mine_path);
        Some((mine_path, removed_points))
    }

    fn remove_mine_path_cleanup(&mut self, mine_path: &MinePath) -> Vec<VPoint> {
        let removed_points = mine_path.total_area();
        for point in &removed_points {
            let existing = self.get_pixel(point);
            if existing != Pixel::Rail {
                panic!("existing {existing:?} is not Rail")
            }
        }
        self.pixels.change(removed_points.clone()).remove();
        removed_points
    }

    //
    // pub fn get_rail_TODO(&self) -> impl Iterator<Item = &Rail> {
    //     self.rail_paths.iter().flat_map(|v| &v.rail)
    // }
    //
    pub fn get_mine_paths(&self) -> &[MinePath] {
        &self.rail_paths
    }
    //
    // pub fn get_mines_mut(&mut self) -> &mut [MinePath] {
    //     &mut self.rail_paths
    // }

    pub fn dump_pixels_xy(&self) -> impl Iterator<Item = &Pixel> {
        self.pixels.iter_xy_pixels()
    }

    pub fn dummy_area_entire_surface(&self) -> VArea {
        let radius = self.get_radius_i32();
        VArea::from_arbitrary_points_pair(
            VPoint::new(-radius, -radius),
            VPoint::new(radius, radius),
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

    pub fn tunables(&self) -> &Tunables {
        &self.tunables
    }

    /// Anti-entropy
    pub fn validate(&self) {
        self.pixels.validate();
        self.validate_patches();
    }

    fn validate_patches(&self) {
        if self.patches.is_empty() {
            panic!("no patches to validate")
        }
        let mut checks = 0;
        let mut points_history: Vec<&VPoint> = Vec::new();
        for patch in &self.patches {
            for point in &patch.pixel_indexes {
                if points_history.contains(&point) {
                    panic!("dupe {patch:?}");
                }
                points_history.push(point);

                let pixel = self.pixels.get_entity_by_point(point).unwrap();
                assert_eq!(pixel.pixel, patch.resource);
                checks += 1;
            }
        }
        debug!("validate {checks} checks");
    }
}

impl Display for VSurface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VSurface pixels {{ {} }} patches {{ {} }}",
            self.pixels,
            display_patches(&self.patches)
        )
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

fn path_pixel_xy_indexes(out_dir: &Path) -> PathBuf {
    out_dir.join("pixel-xy-indexes.dat")
}

fn path_pixel_xy_indexes_clone() -> PathBuf {
    Path::new("/tmp/pixel-xy-indexes-clone.dat").into()
}

// fn path_entity_xy_indexes(out_dir: &Path) -> PathBuf {
//     out_dir.join("entity-xy-indexes.dat")
// }

fn path_state(out_dir: &Path) -> PathBuf {
    out_dir.join("vsurface-state.json")
}

//</editor-fold>

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VPixel {
    pub(super) pixel: Pixel,
}

impl VPixel {
    pub fn pixel(&self) -> &Pixel {
        &self.pixel
    }
}

#[cfg(test)]
mod test {
    use crate::surface::pixel::Pixel;
    use crate::surfacev::vsurface::VSurface;
    use facto_loop_miner_common::log_init_trace;
    use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
    use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ZERO, VPoint};
    use facto_loop_miner_fac_engine::game_blocks::rail_hope::{RailHopeAppender, RailHopeLink};
    use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, RailHopeSingle};
    use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;

    fn test_basic_surface() {
        log_init_trace();
        let mut surface = VSurface::new(50);

        let dummy_link: HopeLink = {
            let mut hope = RailHopeSingle::new(
                VPOINT_ZERO,
                FacDirectionQuarter::North,
                FacItemOutput::new_null().into_rc(),
            );
            hope.add_straight(5);
            hope.into_links().into_iter().next().unwrap()
        };
        surface
            .change_pixels(dummy_link.area_vec())
            .stomp(Pixel::Rail);

        // test overwrite
        surface
            .change_pixels(dummy_link.area_vec())
            .stomp(Pixel::EdgeWall);

        // let test_output_dir = Path::new("work/test-output");
        // info!("writing to {}", test_output_dir.display());
        // surface.save_pixel_img_colorized(&test_output_dir).unwrap()
    }

    #[test]
    fn radius_checks() {
        let surface = VSurface::new(50);

        let extreme_top_left = VPoint::new(-50, -50);
        let extreme_bottom_right = VPoint::new(50, 50);

        assert_eq!(surface.get_pixel(extreme_top_left), Pixel::Empty);
        assert_eq!(surface.get_pixel(extreme_bottom_right), Pixel::Empty);

        // let new = extreme_top_left - VPOINT_ONE;
        // // assert_eq!(
        // //     surface
        // //         .pixels
        // //         .xy_to_index_unchecked(new.x(), new.y()),
        // //         // .map(|v| v.pixel),
        // //     Some(Pixel::Empty)
        // // );
        // assert_eq!(surface.pixels.xy_to_index_unchecked(new.x(), new.y()), 5);

        assert!(!surface.is_point_out_of_bounds(&extreme_top_left));
        assert!(!surface.is_point_out_of_bounds(&extreme_bottom_right));
    }
}
