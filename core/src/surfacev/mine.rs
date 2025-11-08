use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::{
    VSurfacePatch, VSurfacePixel, VSurfacePixelAsVs, VSurfacePixelMut,
};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{
    VPOINT_SECTION, VPOINT_SECTION_Y_ONLY, VPOINT_TEN, VPoint,
};
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, SECTION_POINTS_I32};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::HopeSodaLink;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Arguments;
use tracing::{trace, warn};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MinePath {
    pub location: MineLocation,
    pub links: Vec<HopeLink>,
    pub sodas: Vec<HopeSodaLink>,
    pub segment: VSegment,
    pub cost: u32,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MineLocation {
    patch_indexes: Vec<usize>,
    area_min: VArea,
    area_no_touch: VArea,
    area_buffered: VArea,
    endpoints: Vec<VPoint>,
    endpoints_adjust_direction: Vec<FacDirectionQuarter>,
}

impl MinePath {
    pub fn total_area(&self) -> Vec<VPoint> {
        let mut new_points: Vec<VPoint> = Vec::new();
        for link in &self.links {
            link.area(&mut new_points);
        }

        let old_len = new_points.len();
        new_points.sort();
        new_points.dedup();
        let new_len = new_points.len();
        if old_len != new_len {
            warn!(
                "dedupe mine path from {} to {}",
                old_len.to_formatted_string(&LOCALE),
                new_len.to_formatted_string(&LOCALE)
            )
        }
        new_points
    }
}

impl MineLocation {
    pub fn from_patch_indexes(surface: VSurfacePatch, patch_indexes: Vec<usize>) -> Option<Self> {
        let patch_corners = surface
            .get_patches()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if patch_indexes.contains(&i) {
                    Some(p)
                } else {
                    None
                }
            })
            .flat_map(|p| p.area.get_corner_points());
        let area_min = VArea::from_arbitrary_points(patch_corners);

        let area_no_touch = area_min
            .normalize_step_rail(0)
            .normalize_within_radius(surface.pixels().get_radius_i32() - 1);
        // -- sanity --
        {
            if !surface
                .pixels()
                .is_point_out_of_bounds(&(area_no_touch.point_top_left() - VPOINT_SECTION))
                && !surface
                    .pixels()
                    .is_point_out_of_bounds(&(area_no_touch.point_bottom_right() + VPOINT_SECTION))
            {
                let size = area_no_touch.as_size();
                assert_eq!(size.x() % SECTION_POINTS_I32, 0, "{size}");
                assert_eq!(size.y() % SECTION_POINTS_I32, 0, "{size}");
            }
        }
        // ^^ sanity ^^

        let area_buffered = VArea::from_arbitrary_points_pair(
            area_no_touch.point_top_left() - VPOINT_SECTION_Y_ONLY,
            area_no_touch.point_bottom_right() + VPOINT_SECTION_Y_ONLY,
        )
        .normalize_within_radius(surface.pixels().get_radius_i32() - 1);

        assert!(area_no_touch.get_points().len() < area_buffered.get_points().len());

        let Some((endpoints, endpoints_adjust_direction)) =
            Self::new_endpoints(surface.pixels(), &area_no_touch)
        else {
            warn!("Excluding mine at {}", area_no_touch);
            return None;
        };

        Some(Self {
            patch_indexes,
            area_min,
            area_no_touch,
            area_buffered,
            endpoints,
            endpoints_adjust_direction,
        })
    }

    fn new_endpoints(
        surface: VSurfacePixel,
        area_min: &VArea,
    ) -> Option<(Vec<VPoint>, Vec<FacDirectionQuarter>)> {
        let centered_rounded = area_min.point_center().move_round_rail_down();

        let destination_top_raw =
            VPoint::new(centered_rounded.x(), area_min.point_top_left().y()).move_round_rail_down();
        destination_top_raw.assert_step_rail();

        let destination_bottom_raw =
            VPoint::new(centered_rounded.x(), area_min.point_bottom_right().y())
                .move_round_rail_up();
        destination_bottom_raw.assert_step_rail();

        let mut endpoints = Vec::with_capacity(2);
        let mut endpoints_adjust_direction = Vec::with_capacity(2);
        for (cur_endpoint, adjust_direction) in [
            (destination_top_raw, FacDirectionQuarter::West),
            (destination_bottom_raw, FacDirectionQuarter::East),
        ] {
            // adjust more to account for link

            // basic pre-filter to not screw up later
            if surface.is_point_out_of_bounds(&cur_endpoint) {
                continue;
            }

            endpoints.push(cur_endpoint);
            endpoints_adjust_direction.push(adjust_direction);
        }
        if endpoints.is_empty() {
            warn!(
                "excluding mine top {destination_top_raw} bottom {destination_bottom_raw} for {area_min}"
            );
            None
        } else {
            Some((endpoints, endpoints_adjust_direction))
        }
    }

    pub fn revalidate_endpoints_after_no_touch(&mut self, surface: VSurfacePixel) {
        assert_eq!(self.endpoints.len(), self.endpoints_adjust_direction.len());
        trace!("start {}", self.area_min().point_center());

        'endpoints: for endpoint_index in (0..self.endpoints.len()).rev() {
            let mut endpoint = self.endpoints[endpoint_index];
            let adjust_direction = self.endpoints_adjust_direction[endpoint_index];
            trace!("dir {adjust_direction}");

            /// See [crate::navigator::base_source::BaseSourceEighth]
            /// This is always applied vertically
            const MAX_INTRA_OFFSET: VPoint = VPoint::new(0, 4 * 6);

            /// Given endpoint is center of dual rail, which always is inside of area
            const DUAL_RAIL_OFFSET: i32 = 4;
            endpoint = endpoint.move_direction_sideways_int(adjust_direction, DUAL_RAIL_OFFSET);

            for adjust_i in 0..3 {
                let mut new_origin = endpoint
                    .move_direction_sideways_int(adjust_direction, adjust_i * SECTION_POINTS_I32);

                match self.is_adjust_endpoint(
                    surface,
                    new_origin,
                    format_args!("mine endpoint {endpoint} at {adjust_i}-natty (cur {new_origin})"),
                ) {
                    Adjustment::Usable => {
                        // more tests below
                    }
                    Adjustment::AdjustMore => {
                        continue;
                    }
                    Adjustment::BadEndpoint => {
                        self.remove_bad_endpoint_index(endpoint_index);
                        continue 'endpoints;
                    }
                }
                // best natty endpoint
                self.endpoints[endpoint_index] =
                    new_origin.move_direction_sideways_int(adjust_direction, -DUAL_RAIL_OFFSET);
                trace!("uopdat! {endpoint_index}");

                // now try with intra offset
                new_origin += MAX_INTRA_OFFSET;
                match self.is_adjust_endpoint(
                    surface,
                    new_origin,
                    format_args!("mine endpoint {endpoint} at {adjust_i}-intra (cur {new_origin})"),
                ) {
                    Adjustment::Usable => {
                        // success!
                        trace!("final good!");
                        continue 'endpoints;
                    }
                    Adjustment::AdjustMore => {
                        // just skip ahead
                        continue;
                    }
                    Adjustment::BadEndpoint => {
                        // maybe the next adjustment is better?
                        continue;
                    }
                }
            }
            trace!("out of adjustment");
            self.remove_bad_endpoint_index(endpoint_index);
        }
    }

    fn remove_bad_endpoint_index(&mut self, i: usize) {
        // trace!("remove {i}");
        self.endpoints.remove(i);
        self.endpoints_adjust_direction.remove(i);
        trace!(
            "remove {i} remain {}",
            self.endpoints.iter().map(|v| v.to_string()).join(",")
        );
    }

    fn is_adjust_endpoint(
        &self,
        scratch_surface: VSurfacePixel,
        new_origin: VPoint,
        debug_prefix: Arguments,
    ) -> Adjustment {
        // todo: Multi-approach
        const ONLY_CURRENT_ENDPOINT_DIRECTION: FacDirectionQuarter = FacDirectionQuarter::East;

        let end_link = HopeSodaLink::new_soda_straight(new_origin, ONLY_CURRENT_ENDPOINT_DIRECTION);
        let end_link_points = end_link.area_vec();

        // does link fit inside the surface?
        if end_link_points
            .iter()
            .any(|v| scratch_surface.is_point_out_of_bounds(v))
        {
            // cannot go further out of bounds
            trace!(
                "{debug_prefix} out of bounds, remain {}",
                self.endpoints.len() - 1
            );
            return Adjustment::BadEndpoint;
        }

        // is link still inside the no-touch zone?
        if self.area_no_touch.contains_points_any(&end_link_points) {
            // try the next one
            trace!("{debug_prefix} inside self no touch");
            return Adjustment::AdjustMore;
        }

        // is link points valid?
        if !self.is_surface_points_free_excluding_self_area(
            scratch_surface,
            end_link_points,
            &debug_prefix,
        ) {
            return Adjustment::BadEndpoint;
        }

        // is the link able to be reached?
        let link_backwards = HopeSodaLink::new_soda_straight_flipped(&end_link);
        for link in [
            link_backwards.add_straight_section(),
            link_backwards.add_turn90(true),
            link_backwards.add_turn90(false),
        ] {
            if link
                .area_vec()
                .iter()
                .any(|v| scratch_surface.is_point_out_of_bounds(v))
            {
                trace!(
                    "{debug_prefix} is out of bounds, remain {}",
                    self.endpoints.len() - 1
                );
                return Adjustment::BadEndpoint;
            }

            if !self.is_surface_points_free_excluding_self_area(
                scratch_surface,
                link.area_vec(),
                &debug_prefix,
            ) {
                trace!(
                    "{debug_prefix} is unreachable, remain {}",
                    self.endpoints.len() - 1
                );
                return Adjustment::BadEndpoint;
            }
        }

        // it's valid!
        trace!("{debug_prefix} is valid");
        Adjustment::Usable
    }

    fn is_surface_points_free_excluding_self_area(
        &self,
        surface: VSurfacePixel,
        points: impl IntoIterator<Item = impl Borrow<VPoint>>,
        debug_prefix: &Arguments,
    ) -> bool {
        let mut pixels: Vec<Pixel> = points
            .into_iter()
            .filter_map(|p| {
                let p = p.borrow();
                if surface.is_point_out_of_bounds(p) {
                    panic!("we already checked this?");
                }
                let pixel = surface.get_pixel(p);
                if pixel == Pixel::MineNoTouch && self.area_buffered.contains_point(p) {
                    // exclude self
                    None
                } else {
                    Some(pixel)
                }
            })
            .collect_vec();
        pixels.sort();
        pixels.dedup();
        let pixels_debug = pixels.iter().map(|v| v.as_ref()).join(",");

        if pixels.iter().all(|p| *p == Pixel::Empty) {
            // good all empty!
            true
        } else if pixels
            .iter()
            .all(|p| matches!(*p, Pixel::Empty | Pixel::MineNoTouch))
        {
            trace!(
                "{debug_prefix} is not in mine touch, maybe touching another?, remain {}",
                self.endpoints.len() - 1
            );
            false
        } else if pixels
            .iter()
            .all(|p| Pixel::is_resource(p) || matches!(*p, Pixel::Empty | Pixel::MineNoTouch))
        {
            // todo: do this ever happen?
            trace!("{debug_prefix} hit another mine");
            false
        } else {
            // panic!("{debug_prefix} is {pixels_debug}");
            panic!("{debug_prefix} is {pixels_debug}");
        }
    }

    pub fn area_min(&self) -> &VArea {
        &self.area_min
    }

    pub fn area_no_touch(&self) -> &VArea {
        &self.area_no_touch
    }

    pub fn area_buffered(&self) -> &VArea {
        &self.area_buffered
    }

    pub fn draw_area_buffered(&self, surface: &mut VSurfacePixelMut) {
        self.draw_area_buffered_with(surface, Pixel::MineNoTouch)
    }

    pub fn draw_area_buffered_with(&self, surface: &mut VSurfacePixelMut, pixel: Pixel) {
        surface
            .change_pixels(self.area_buffered.get_points())
            .find_empty_into(pixel)
    }

    pub fn draw_area_buffered_to_no_touch(&self, surface: &mut VSurfacePixelMut) {
        // let needle = self.area_buffered.point_top_left();
        // let existing_pixel = surface.get_pixel(needle);
        // assert_eq!(existing_pixel, Pixel::MineNoTouch, "at {needle}");

        // --sanity--
        for point in self
            .area_buffered
            .get_points()
            .into_iter()
            .filter(|v| !self.area_no_touch.contains_point(v))
        {
            // assert_eq!(surface.get_pixel(point), Pixel::MineNoTouch);
            let pixel = surface.pixels().get_pixel(point);
            if !matches!(pixel, Pixel::MineNoTouch | Pixel::Empty | Pixel::UraniumOre) {
                surface
                    .change_square(&VArea::from_arbitrary_points_pair(
                        point,
                        point + VPOINT_TEN,
                    ))
                    .stomp(Pixel::Highlighter);
                surface
                    .pixels()
                    .paint_pixel_colored_zoomed()
                    .save_to_oculante();
                panic!("for {point} is {pixel:?}")
            }
        }

        surface
            .change_pixels(
                self.area_buffered
                    .get_points()
                    .into_iter()
                    .filter(|v| !self.area_no_touch.contains_point(v)),
            )
            .remove();
    }

    pub fn draw_area_buffered_highlight_pixel(&self, surface: &mut VSurfacePixelMut, pixel: Pixel) {
        surface
            .change_pixels(self.area_buffered.get_points())
            .find_into(Pixel::MineNoTouch, pixel)
    }

    pub fn restore_area_buffered(
        mines: &[impl Borrow<Self>],
        surface: &mut VSurfacePixelMut,
        removed_rail: Vec<VPoint>,
    ) {
        let mut intersected_mines = HashSet::new();
        for point in removed_rail {
            for mine in mines {
                let mine = mine.borrow();
                if mine.area_buffered().contains_point(&point) {
                    intersected_mines.insert(mine);
                    break;
                }
            }
        }

        for mine in intersected_mines {
            surface
                .change_pixels(mine.area_buffered().get_points())
                .find_empty_into(Pixel::MineNoTouch)
        }
    }

    // pub fn endpoints(&self) -> &[VPoint] {
    //     &self.endpoints
    // }

    pub fn destinations(&self) -> impl Iterator<Item = VPointDirectionQ> {
        // todo
        self.endpoints
            .iter()
            .map(|v| VPointDirectionQ(*v, FacDirectionQuarter::East))
    }

    pub(super) fn patch_indexes(&self) -> &[usize] {
        self.patch_indexes.as_slice()
    }
}

enum Adjustment {
    AdjustMore,
    BadEndpoint,
    Usable,
}

#[derive(Serialize, Deserialize)]
pub struct DebugMinePatch {
    pub pixel: Pixel,
    pub points: Vec<VPoint>,
}

#[cfg(test)]
mod test {
    use crate::navigator::planners::{debug_draw_mine_links, debug_draw_segment};
    use crate::surface::pixel::Pixel;
    use crate::surfacev::mine::{DebugMinePatch, MineLocation};
    use crate::surfacev::vpatch::VPatch;
    use crate::surfacev::vsurface::{
        VSurface, VSurfacePatchAsVs, VSurfacePatchAsVsMut, VSurfacePixelAsVs, VSurfacePixelAsVsMut,
    };
    use facto_loop_miner_common::duration::BasicWatch;
    use facto_loop_miner_common::log_init_trace;
    use facto_loop_miner_fac_engine::common::varea::VArea;
    use facto_loop_miner_fac_engine::common::vpoint::{
        VPOINT_SECTION, VPOINT_SECTION_NEGATIVE, VPoint,
    };
    use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
    use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
    use itertools::Itertools;
    use simd_json::prelude::ArrayTrait;

    #[test]
    fn test_destinations() {
        let mut surface = VSurface::new(300);
        surface.patches_mut().add_patches([VPatch {
            area: VArea::from_arbitrary_points_pair(VPoint::new(-5, -5), VPoint::new(6, 6)),
            resource: Pixel::CrudeOil,
            pixel_indexes: Vec::new(),
        }]);

        let mine = MineLocation::from_patch_indexes(surface.patches(), vec![0]).unwrap();
        assert_eq!(mine.area_min.point_top_left(), VPoint::new(-5, -5));
        assert_eq!(mine.area_min.point_bottom_right(), VPoint::new(6, 6));

        assert_eq!(mine.area_no_touch.point_top_left(), VPOINT_SECTION_NEGATIVE);
        assert_eq!(mine.area_no_touch.point_bottom_right(), VPOINT_SECTION);

        assert_eq!(
            mine.area_buffered.point_top_left(),
            VPOINT_SECTION_NEGATIVE + VPOINT_SECTION_NEGATIVE
        );
        assert_eq!(
            mine.area_buffered.point_bottom_right(),
            VPOINT_SECTION + VPOINT_SECTION
        );

        assert_eq!(
            mine.destinations().collect_vec(),
            [
                VPointDirectionQ(
                    VPoint::new(0, -SECTION_POINTS_I32),
                    FacDirectionQuarter::East
                ),
                VPointDirectionQ(
                    VPoint::new(0, SECTION_POINTS_I32),
                    FacDirectionQuarter::East
                )
            ]
        );
    }

    #[test]
    fn test() {
        log_init_trace();

        let surface = &mut VSurface::new(550);

        let patches = load_mine_patch();
        for patch in &patches {
            let area = VArea::from_arbitrary_points(&patch.points);
            println!("area {area}");
        }
        surface
            .patches_mut()
            .add_patches(patches.iter().map(|v| VPatch {
                pixel_indexes: v.points.clone(),
                resource: v.pixel,
                area: VArea::from_arbitrary_points(&v.points),
            }));
        // blank surface doesn't have pixels
        for patch in &patches {
            surface
                .pixels_mut()
                .change_pixels(patch.points.clone())
                .stomp(patch.pixel);
        }

        let mut mine = MineLocation::from_patch_indexes(
            surface.patches(),
            (0..surface.patches().get_patches().len()).collect(),
        )
        .unwrap();
        mine.draw_area_buffered(&mut surface.pixels_mut());

        // debug_draw_mine_links(surface, [&mine]);

        // <<<
        mine.revalidate_endpoints_after_no_touch(surface.pixels());
        assert_ne!(mine.destinations().next(), None);

        debug_draw_mine_links(&mut surface.pixels_mut(), [&mine]);

        let watch = BasicWatch::start();
        let mut grid = Vec::new();
        for x in 0..surface.pixels().get_radius_i32() {
            for y in 0..surface.pixels().get_radius_i32() {
                if x % SECTION_POINTS_I32 == 0 || y % SECTION_POINTS_I32 == 0 {
                    grid.push(VPoint::new(x, y));
                }
            }
        }
        println!("gen in {watch} total {}", grid.len());
        let watch = BasicWatch::start();
        surface
            .pixels_mut()
            .change_pixels(grid)
            .stomp(Pixel::Highlighter);
        println!("stomp in {watch}");

        surface
            .pixels()
            .paint_pixel_colored_entire()
            .save_to_oculante();
    }

    fn load_mine_patch() -> Vec<DebugMinePatch> {
        const INPUT: &str = include_str!("example_mine.json");
        let mut input = Vec::from(INPUT.as_bytes());
        let mut patches: Vec<DebugMinePatch> = simd_json::from_slice(&mut input).unwrap();

        let area = VArea::from_arbitrary_points(patches.iter().flat_map(|v| &v.points));
        let top_left = area.point_top_left();
        let area_offset = VPoint::new(
            (top_left.x() - (SECTION_POINTS_I32 * 2)).next_multiple_of(SECTION_POINTS_I32),
            (top_left.y() - (SECTION_POINTS_I32 * 2)).next_multiple_of(SECTION_POINTS_I32),
        );

        for patch in &mut patches {
            for point in &mut patch.points {
                *point -= area_offset;
            }
        }
        patches
    }
}
