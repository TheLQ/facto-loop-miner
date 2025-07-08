use crate::surface::pixel::Pixel;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::{RemovedEntity, VSurface};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::{VArea, VAreaSugar};
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPOINT_SECTION};
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use itertools::Itertools;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::{hint, ptr, slice};
use tracing::warn;

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
    destinations: Vec<VPointDirectionQ>,
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
        let area_buffered = VArea::from_arbitrary_points_pair(
            area_no_touch.point_top_left() - VPOINT_SECTION,
            area_no_touch.point_bottom_right() + VPOINT_SECTION,
        )
        .normalize_within_radius(surface.get_radius_i32() - 1);

        assert!(area_no_touch.get_points().len() < area_buffered.get_points().len());

        let Some(endpoints) = Self::new_endpoints(surface, &area_no_touch) else {
            warn!("Excluding mine at {}", area_no_touch);
            return None;
        };
        let destinations = endpoints
            .iter()
            .map(|p| VPointDirectionQ(*p, FacDirectionQuarter::East))
            .collect();

        // fn expanded_mine_no_touching_zone(surface: &VSurface, mine: &MineLocation) -> VArea {
        // const MINE_RAIL_BUFFER_PIXELS: i32 = RAIL_STRAIGHT_DIAMETER_I32 * 2 * 2;
        // VArea::from_arbitrary_points_pair(
        //     area.point_top_left()
        //         .move_xy(-MINE_RAIL_BUFFER_PIXELS, -MINE_RAIL_BUFFER_PIXELS),
        //     area.point_bottom_right()
        //         .move_xy(MINE_RAIL_BUFFER_PIXELS, MINE_RAIL_BUFFER_PIXELS),
        // )
        //     .normalize_within_radius(surface.get_radius_i32())

        Some(Self {
            patch_indexes,
            area_min,
            area_no_touch,
            area_buffered,
            endpoints,
            destinations,
        })
    }

    fn new_endpoints(surface: &VSurface, area: &VArea) -> Option<Vec<VPoint>> {
        let centered_rounded = area.point_center().move_round_rail_down();
        let destination_top =
            VPoint::new(centered_rounded.x(), area.point_top_left().y()).move_round_rail_down();
        destination_top.assert_step_rail();
        let destination_bottom =
            VPoint::new(centered_rounded.x(), area.point_bottom_right().y()).move_round_rail_up();
        destination_bottom.assert_step_rail();

        let endpoints: Vec<VPoint> = [destination_top, destination_bottom]
            .into_iter()
            .filter(|destination| !surface.is_point_out_of_bounds(destination))
            .collect();
        // assert_ne!(
        //     endpoints.len(),
        //     0,
        //     "stripped all possible destinations from {area_min} in {surface}"
        // );
        if endpoints.is_empty() {
            warn!("excluding mine top {destination_top} bottom {destination_bottom} for {area}");
            None
        } else {
            Some(endpoints)
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
        surface.draw_square_area(&self.area_buffered, pixel)
    }

    pub fn draw_area_buffered_to_no_touch(&self, surface: &mut VSurface) {
        let needle = self.area_buffered.point_top_left();
        let existing_pixel = surface.get_pixel(needle);
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
            .collect();
        surface
            .set_pixels(Pixel::Empty, removed_buffer_pixels)
            .unwrap();
        surface.set_pixels(Pixel::MineNoTouch, new_points).unwrap();
    }

    pub fn draw_area_buffered_replacing(&self, surface: &mut VSurface, pixel: Pixel) {
        surface.draw_square_area_replacing(&self.area_buffered, Pixel::MineNoTouch, pixel)
    }

    /// Don't take self as MineLocation already moved / don't need it
    pub fn draw_area_no_touch_to_buffered(
        surface: &mut VSurface,
        mut removed_entity: RemovedEntity,
    ) -> RemovedEntity {
        removed_entity
            .points
            .retain(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch | Pixel::Empty));
        surface.set_pixel_entity_swap(removed_entity.entity_id, removed_entity.points, false)
    }

    pub fn endpoints(&self) -> &[VPoint] {
        &self.endpoints
    }

    pub fn destinations(&self) -> &[VPointDirectionQ] {
        &self.destinations
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

#[cfg(test)]
mod test {
    use crate::surface::pixel::Pixel;
    use crate::surfacev::mine::MineLocation;
    use crate::surfacev::vpatch::VPatch;
    use crate::surfacev::vsurface::VSurface;
    use facto_loop_miner_fac_engine::common::varea::VArea;
    use facto_loop_miner_fac_engine::common::vpoint::{
        VPoint, VPOINT_SECTION, VPOINT_SECTION_NEGATIVE,
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
