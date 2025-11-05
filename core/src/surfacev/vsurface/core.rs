use crate::opencv::{draw_text_size, mat_into_points};
use crate::state::machine::StepParams;
use crate::state::tuneables::Tunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::{CoreConvertPathResult, VResult};
use crate::surfacev::mine::MinePath;
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::pixel::AsVs;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_io::{read_entire_file, write_entire_file};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
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
    pub(crate) pixels: VEntityMap<VPixel>,
    // entities: VEntityMap<VEntity>,
    pub(crate) patches: Vec<VPatch>,
    #[serde(default)]
    pub(crate) rails: Vec<MinePath>,
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
            rails: Vec::new(),
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
        new_surface.pixels().log_pixel_stats("vsurface load");
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

    //</editor-fold>

    //<editor-fold desc="io save">

    pub fn save(&self, out_dir: &Path) -> VResult<()> {
        info!("+++ Saving to {} {}", out_dir.display(), self);
        self.pixels().log_pixel_stats("vsurface save");
        let total_save_watch = BasicWatch::start();
        self.save_state(out_dir)?;

        self.pixels()
            .paint_pixel_colored_entire()
            .save_to_file(out_dir)?;
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

    pub fn tunables(&self) -> &Tunables {
        &self.tunables
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

pub(super) fn path_pixel_xy_indexes_clone() -> PathBuf {
    Path::new("/tmp/pixel-xy-indexes-clone.dat").into()
}

// fn path_entity_xy_indexes(out_dir: &Path) -> PathBuf {
//     out_dir.join("entity-xy-indexes.dat")
// }

fn path_state(out_dir: &Path) -> PathBuf {
    out_dir.join("vsurface-state.json")
}

//</editor-fold>

#[cfg(test)]
mod test {
    use crate::surface::pixel::Pixel;
    use crate::surfacev::vsurface::core::VSurface;
    use facto_loop_miner_common::log_init_trace;
    use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
    use facto_loop_miner_fac_engine::common::varea::VArea;
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
    fn text_test() {
        log_init_trace();

        let mut surface = VSurface::new(500);
        surface
            .change_square(&VArea::from_radius(VPOINT_ZERO, 4))
            .stomp(Pixel::EdgeWall);

        surface.draw_text_at(VPOINT_ZERO, "1234");

        surface.paint_pixel_colored_entire().save_to_oculante();
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
