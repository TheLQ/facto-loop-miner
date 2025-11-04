use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MinePath;
use crate::surfacev::ventity_map::{VEntityMap, VPixel};
use crate::surfacev::vsurface::{
    VSurface, VSurfacePixel, VSurfacePixelMut, VSurfaceRails, VSurfaceRailsMut,
};
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use tracing::trace;

pub struct PlugMut<'s> {
    pub(super) rails: &'s mut Vec<MinePath>,
    pub(super) pixels: &'s mut VEntityMap<VPixel>,
}

impl<'s> PlugMut<'s> {
    pub fn pixels_mut(&mut self) -> VSurfacePixelMut {
        VSurfacePixelMut::new(&mut self.pixels)
    }

    pub fn pixels(&self) -> VSurfacePixel {
        VSurfacePixel::new(&self.pixels)
    }

    pub fn new(rails: &'s mut Vec<MinePath>, pixels: &'s mut VEntityMap<VPixel>) -> Self {
        Self { rails, pixels }
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
        self.pixels_mut().change_pixels(new_points).stomp(pixel);

        // todo
        // // add markers for start points
        // let start_points: Vec<VPoint> = mine_path.links.iter().map(|v| v.start).collect_vec();
        // self.set_pixels(Pixel::EdgeWall, start_points)?;

        self.rails.push(mine_path);
    }

    pub fn remove_mine_path_at(&mut self, index: usize) -> Option<(MinePath, Vec<VPoint>)> {
        let mine_path = self.rails.remove(index);
        trace!(
            "{} at {index} total {} - {}",
            nu_ansi_term::Color::Red.paint("mine remove"),
            self.rails.len(),
            mine_path.segment,
        );

        let removed_points = self.remove_mine_path_cleanup(&mine_path);
        Some((mine_path, removed_points))
    }

    pub fn remove_mine_path_pop(&mut self) -> Option<(MinePath, Vec<VPoint>)> {
        trace!(
            "{} pop total {}",
            nu_ansi_term::Color::Red.paint("mine remove"),
            self.rails.len()
        );
        let mine_path = self.rails.pop()?;
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
}

pub struct Plug<'s> {
    rails: &'s [MinePath],
}

impl<'s> Plug<'s> {
    pub fn new(rails: &'s [MinePath]) -> Self {
        Self { rails }
    }

    pub fn get_mine_paths(&self) -> &[MinePath] {
        &self.rails
    }
}

//

impl VSurface {
    pub fn rails(&self) -> VSurfaceRails {
        VSurfaceRails::new(&self.rail_paths)
    }

    pub fn rails_mut(&mut self) -> VSurfaceRailsMut {
        let Self { rails, pixels, .. } = self;
        PlugMut { rails, pixels }
    }
}
