use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_SECTION, VPOINT_SECTION_Y_ONLY, VPoint};
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, SECTION_POINTS_I32};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::HopeSodaLink;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Arguments;
use tracing::{trace, warn};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct MinePath {
    pub mine_base: MineLocation,
    pub links: Vec<HopeLink>,
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
    pub fn from_patch_indexes(surface: &VSurface, patch_indexes: Vec<usize>) -> Option<Self> {
        let patch_corners = surface
            .get_patches_slice()
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
            .normalize_within_radius(surface.get_radius_i32() - 1);
        // -- sanity --
        {
            if !surface.is_point_out_of_bounds(&(area_no_touch.point_top_left() - VPOINT_SECTION))
                && !surface
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
        .normalize_within_radius(surface.get_radius_i32() - 1);

        assert!(area_no_touch.get_points().len() < area_buffered.get_points().len());

        let Some((endpoints, endpoints_adjust_direction)) =
            Self::new_endpoints(surface, &area_no_touch)
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
        surface: &VSurface,
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
        for (cur_endpoint, change_sign) in [
            (destination_top_raw, FacDirectionQuarter::West),
            (destination_bottom_raw, FacDirectionQuarter::East),
        ] {
            // basic pre-filter to not screw up later
            if surface.is_point_out_of_bounds(&cur_endpoint) {
                continue;
            }

            endpoints.push(cur_endpoint);
            endpoints_adjust_direction.push(change_sign);
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

    pub fn revalidate_endpoints_after_no_touch(&mut self, surface: &mut VSurface) {
        assert_eq!(self.endpoints.len(), self.endpoints_adjust_direction.len());
        trace!("start {}", self.area_min().point_center());

        'endpoints: for endpoint_index in (0..self.endpoints.len()).rev() {
            let endpoint = self.endpoints[endpoint_index];
            let adjust_direction = self.endpoints_adjust_direction[endpoint_index];
            trace!("dir {adjust_direction}");

            /// See [crate::navigator::base_source::BaseSourceEighth]
            /// This is always applied vertically
            const MAX_INTRA_OFFSET: VPoint = VPoint::new(0, 4 * 6);

            for adjust_i in 0..2 {
                let mut new_origin = endpoint
                    .move_direction_sideways_int(adjust_direction, adjust_i * SECTION_POINTS_I32);

                match self.adjust_endpoint(
                    &surface,
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
                self.endpoints[endpoint_index] = new_origin;
                trace!("uopdat! {endpoint_index}");

                // now try with intra offset
                new_origin += MAX_INTRA_OFFSET;
                match self.adjust_endpoint(
                    &surface,
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

    fn adjust_endpoint(
        &self,
        scratch_surface: &VSurface,
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
        if !self.is_surface_points_free_excluding_buffered(
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
            if !self.is_surface_points_free_excluding_buffered(
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

    fn is_surface_points_free_excluding_buffered(
        &self,
        surface: &VSurface,
        points: impl IntoIterator<Item = impl Borrow<VPoint>>,
        debug_prefix: &Arguments,
    ) -> bool {
        let mut pixels = points
            .into_iter()
            .filter_map(|p| {
                let p = p.borrow();
                if surface.is_point_out_of_bounds(p) {
                    return None;
                }
                let pixel = surface.get_pixel(p);
                if pixel == Pixel::MineNoTouch
                    && self.area_buffered.contains_point(p)
                    && !self.area_no_touch.contains_point(p)
                {
                    // exclude self
                    None
                } else {
                    Some(pixel)
                }
            })
            .collect_vec();
        pixels.sort();
        pixels.dedup();

        if pixels.iter().all(|p| *p == Pixel::Empty) {
            // good all empty!
            true
        } else if pixels
            .iter()
            .all(|p| matches!(*p, Pixel::Empty | Pixel::MineNoTouch))
        {
            // // Are we touching only our own area_buffered?
            // if self.area_buffered.contains_points_all(
            //     end_link_points
            //         .iter()
            //         .filter(|p| surface.get_pixel(*p) == Pixel::MineNoTouch),
            // ) {
            //     trace!("{debug_prefix} is in free space")
            // } else {
            trace!(
                "{debug_prefix} is not in mine touch, maybe touching another?, remain {}",
                self.endpoints.len() - 1
            );
            false
            // }
            // let mut new_surface = surface.clone();
            // new_surface
            //     .set_pixels(Pixel::Highlighter, end_link_points)
            //     .unwrap();
            // // new_surface.draw_square_area_forced(self.area_no_touch(), Pixel::EdgeWall);
            // new_surface.paint_pixel_colored_entire().save_to_oculante();
            // std::process::exit(1);
        } else if pixels
            .iter()
            .all(|v| Pixel::is_resource(v) || *v == Pixel::Empty)
        {
            // todo: do this ever happen?
            panic!("{debug_prefix} hit another mine");
        } else {
            // panic!("{debug_prefix} is {pixels_debug}");
            panic!("{debug_prefix}");
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

    pub fn draw_area_buffered(&self, surface: &mut VSurface) {
        self.draw_area_buffered_with(surface, Pixel::MineNoTouch)
    }

    pub fn draw_area_buffered_with(&self, surface: &mut VSurface, pixel: Pixel) {
        surface
            .change_pixels(self.area_buffered.get_points())
            .stomp(pixel)
    }

    pub fn draw_area_buffered_to_no_touch(&self, surface: &mut VSurface) {
        // let needle = self.area_buffered.point_top_left();
        // let existing_pixel = surface.get_pixel(needle);
        // assert_eq!(existing_pixel, Pixel::MineNoTouch, "at {needle}");

        let new_points = self
            .area_no_touch
            .get_points()
            .into_iter()
            .filter(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch | Pixel::Empty))
            .collect_vec();
        // surface.set_pixel_entity_swap(surface.get_pixel_entity_id_at(&needle), new_points, false)

        let removed_buffer_pixels = self
            .area_buffered
            .get_points()
            .into_iter()
            .filter(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch))
            .collect_vec();
        surface.change_pixels(removed_buffer_pixels).remove();
        surface.change_pixels(new_points).stomp(Pixel::MineNoTouch);
    }

    pub fn draw_area_buffered_highlight_pixel(&self, surface: &mut VSurface, pixel: Pixel) {
        surface
            .change_pixels(self.area_buffered.get_points())
            .find_into(Pixel::MineNoTouch, pixel)
    }

    /// Don't take self as MineLocation already moved / don't need it
    // pub fn draw_area_no_touch_to_buffered(
    //     surface: &mut VSurface,
    //     mut removed_entity: RemovedEntity,
    // ) -> RemovedEntity {
    //     removed_entity
    //         .points
    //         .retain(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch | Pixel::Empty));
    //     surface.set_pixel_entity_swap(removed_entity.entity_id, removed_entity.points, false)
    // }

    pub fn endpoints(&self) -> &[VPoint] {
        &self.endpoints
    }

    pub fn destinations(&self) -> impl Iterator<Item = VPointDirectionQ> {
        // todo
        self.endpoints
            .iter()
            .map(|v| VPointDirectionQ(*v, FacDirectionQuarter::East))
    }

    pub fn surface_patches_len(&self) -> usize {
        self.patch_indexes.len()
    }

    // pub fn surface_patches<'s>(
    //     &self,
    //     surface: &'s VSurface,
    // ) -> impl IntoIterator<Item = &'s VPatch> {
    //     surface
    //         .get_mine_paths()
    //         .into_iter()
    //         .flat_map(|v| v.mine_base.patch_indexes)
    //         .map(|v| surface.get_patches_slice()[v])
    // }

    // pub fn surface_patches_iter<'s>(
    //     mines: impl IntoIterator<Item = &'s Self>,
    //     surface: &'s VSurface,
    // ) -> impl Iterator<Item = &'s VPatch> {
    //     let patch = surface.get_patches_slice();
    //     mines
    //         .into_iter()
    //         // .map(|v| v.borrow())
    //         .flat_map(|v| v.patch_indexes.as_slice())
    //         .map(|v| &patch[*v])
    // }
}

enum Adjustment {
    AdjustMore,
    BadEndpoint,
    Usable,
}

#[cfg(test)]
mod test {
    use crate::surface::pixel::Pixel;
    use crate::surfacev::mine::MineLocation;
    use crate::surfacev::vpatch::VPatch;
    use crate::surfacev::vsurface::VSurface;
    use facto_loop_miner_fac_engine::common::varea::VArea;
    use facto_loop_miner_fac_engine::common::vpoint::{
        VPOINT_SECTION, VPOINT_SECTION_NEGATIVE, VPoint,
    };
    use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
    use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;

    #[test]
    fn test_destinations() {
        let mut surface = VSurface::new(300);
        surface.add_patches(&[VPatch {
            area: VArea::from_arbitrary_points_pair(VPoint::new(-5, -5), VPoint::new(6, 6)),
            resource: Pixel::CrudeOil,
            pixel_indexes: Vec::new(),
        }]);

        let mine = MineLocation::from_patch_indexes(&surface, vec![0]).unwrap();
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
            mine.destinations,
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
}
