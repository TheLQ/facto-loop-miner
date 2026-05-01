use crate::navigator::planners::common_util::move_further_by_section;
use crate::navigator::{BaseSourceEighth, IntraLevel};
use crate::opencv::TextSize;
use crate::state::tuneables::PathCommonTunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::{
    MineDestinationRef, MineRef, PatchRef, VSurfaceMine, VSurfacePatch, VSurfacePixel,
    VSurfacePixelAsVs, VSurfacePixelAsVsMut, VSurfacePixelMut,
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
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use simd_json::prelude::ArrayTrait;
use tracing::{error, warn};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MinePath {
    pub destination: MineDestinationRef,
    pub links: Vec<HopeLink>,
    pub sodas: Vec<HopeSodaLink>,
    pub segment: VSegment,
    pub cost: u32,
}

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MineLocation {
    patch_indexes: Vec<PatchRef>,
    area_min: VArea,
    area_no_touch: VArea,
    area_buffered: VArea,
    destinations: Vec<MineDestination>,
}

impl MinePath {
    pub fn total_area(&self) -> Vec<VPoint> {
        let mut new_points: Vec<VPoint> = Vec::new();
        for link in &self.links {
            // link.area(&mut new_points);
            let link_area = link.link_area_slow();
            new_points.extend(link_area);
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
    pub fn from_patch_indexes(
        surface: VSurfacePatch,
        patch_indexes: Vec<PatchRef>,
        base_source: &BaseSourceEighth,
        tunables: &PathCommonTunables,
    ) -> Option<Self> {
        let area_min = VArea::from_arbitrary_points(
            patch_indexes
                .iter()
                .flat_map(|v| v.get_patch(surface).area.get_corner_points()),
        );

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

        let destinations = MineDestination::find_all_destinations(
            surface.pixels(),
            &area_no_touch,
            base_source,
            tunables,
            // todo: only finds links ending going east
            FacDirectionQuarter::East,
        );
        if destinations.is_empty() {
            warn!("Excluding mine at {}", area_no_touch);
            return None;
        };

        Some(Self {
            patch_indexes,
            area_min,
            area_no_touch,
            area_buffered,
            destinations,
        })
    }

    pub fn actually_clone(&self) -> Self {
        let Self {
            patch_indexes,
            area_min,
            area_no_touch,
            area_buffered,
            destinations,
        } = self;
        Self {
            patch_indexes: patch_indexes.clone(),
            area_min: area_min.clone(),
            area_no_touch: area_no_touch.clone(),
            area_buffered: area_buffered.clone(),
            destinations: destinations.to_vec(),
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

    pub fn destinations(&self) -> &[MineDestination] {
        self.destinations.as_slice()
    }

    pub fn destination_refs_iter(&self, mine: MineRef) -> impl Iterator<Item = MineDestinationRef> {
        (0..self.destinations.len()).map(move |i| MineDestinationRef(mine, i))
    }

    pub fn destinations_with_refs(
        &self,
        self_ref: MineRef,
    ) -> impl Iterator<Item = (MineDestinationRef, &MineDestination)> {
        self.destinations
            .iter()
            .enumerate()
            .map(move |(i, dest)| (MineDestinationRef(self_ref, i), dest))
    }
}

pub enum MineDraw {
    InitBuffered,
    ChangeNoTouch,
    ChangeBuffered,
    HighlightBufferedMain,
    HighlightBufferedAlt,
}

impl MineDraw {
    pub fn draw_mine(&self, surface: &mut VSurfacePixelMut, mine: &MineLocation) {
        match self {
            MineDraw::InitBuffered => Self::draw_buffered_with(surface, mine, Pixel::MineNoTouch),
            MineDraw::ChangeNoTouch => {
                // --sanity--
                for point in mine
                    .area_buffered
                    .get_points()
                    .into_iter()
                    .filter(|v| !mine.area_no_touch.contains_point(v))
                {
                    // assert_eq!(surface.get_pixel(point), Pixel::MineNoTouch);
                    let pixel = surface.pixels().get_pixel(point);
                    if !matches!(pixel, Pixel::MineNoTouch | Pixel::Empty | Pixel::Rail) {
                        surface
                            .change_pixels(mine.area_buffered.get_points())
                            .stomp(Pixel::Highlighter);

                        surface
                            .change_square(&VArea::from_arbitrary_points_pair(
                                point,
                                point + VPOINT_TEN,
                            ))
                            .stomp(Pixel::Highlighter);
                        // surface
                        //     .pixels()
                        //     .paint_pixel_colored_entire()
                        //     .save_to_oculante();
                        error!("[sanity] for {point} is {pixel:?}")
                    }
                }

                surface
                    .change_pixels(
                        mine.area_buffered
                            .get_points()
                            .into_iter()
                            .filter(|v| !mine.area_no_touch.contains_point(v)),
                    )
                    .remove();
            }
            MineDraw::ChangeBuffered => Self::draw_buffered_with(surface, mine, Pixel::MineNoTouch),
            MineDraw::HighlightBufferedMain => {
                Self::draw_buffered_with(surface, mine, Pixel::Stone)
            }
            MineDraw::HighlightBufferedAlt => {
                Self::draw_buffered_with(surface, mine, Pixel::SteelChest)
            }
        }
    }

    fn draw_buffered_with(surface: &mut VSurfacePixelMut, mine: &MineLocation, pixel: Pixel) {
        surface
            .change_pixels(mine.area_buffered.get_points())
            .find_empty_into(pixel)
    }
}

pub enum MineLocationResolver<'l, 'plan_mine> {
    Lookup(&'l [(MineRef, &'plan_mine MineLocation)]),
    Surface(VSurfaceMine<'plan_mine>),
}

impl<'plan_mine> MineLocationResolver<'_, 'plan_mine> {
    pub fn resolve_mine(&self, mine_ref: MineRef) -> &'plan_mine MineLocation {
        match self {
            Self::Lookup(lookup) => mine_ref.resolve_mine_lookup(lookup),
            Self::Surface(surface) => mine_ref.resolve_mine_surface(*surface),
        }
    }
    pub fn resolve_destination(&self, mine_ref: MineDestinationRef) -> &'plan_mine MineDestination {
        match self {
            Self::Lookup(lookup) => mine_ref.resolve_destination_lookup(lookup),
            Self::Surface(surface) => mine_ref.resolve_destination_surface(*surface),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct MineDestination(Vec<MineDestinationLevel>);

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct MineDestinationLevel {
    level: IntraLevel,
    target: VPointDirectionQ,
}

impl MineDestination {
    fn find_all_destinations(
        surface: VSurfacePixel,
        area_min: &VArea,
        base_source: &BaseSourceEighth,
        tunables: &PathCommonTunables,
        direction: FacDirectionQuarter,
    ) -> Vec<Self> {
        let centered_rounded = area_min.point_center().move_round_rail_down();

        let destination_top_raw =
            VPoint::new(centered_rounded.x(), area_min.point_top_left().y()).move_round_rail_down();
        destination_top_raw.assert_step_rail();

        let destination_bottom_raw =
            VPoint::new(centered_rounded.x(), area_min.point_bottom_right().y())
                .move_round_rail_up();
        destination_bottom_raw.assert_step_rail();

        let gen_link = |origin: VPoint| {
            HopeSodaLink::new_soda_straight_q(&VPointDirectionQ(origin, direction))
        };
        let test = |cur: &VPoint| {
            let end_link = gen_link(*cur);
            let mut conflict_links = Vec::new();

            // is the link able to be reached?
            let link_backwards = HopeSodaLink::new_soda_straight_flipped(&end_link);
            let mut is_best = true;
            let mut failed_turns = 0;
            for (is_turn, link) in [
                (false, end_link.clone()),
                (false, link_backwards.add_straight_section()),
                (true, link_backwards.add_turn90(true)),
                (true, link_backwards.add_turn90(false)),
            ] {
                let points = link.soda_area();
                if points.iter().any(|v| surface.is_point_out_of_bounds(v)) {
                    return None;
                } else {
                    let is_free = surface.is_points_free_slice(&points);
                    if !is_free {
                        conflict_links.push(link);
                    }
                    is_best = is_best && is_free;

                    if is_turn {
                        let is_too_close = area_min.contains_points_any(points);
                        if is_too_close {
                            failed_turns += 1;
                        }
                    }
                }
            }
            is_best = is_best && failed_turns != 2;
            Some((is_best, conflict_links))
        };

        let mut destination_levels = Vec::new();
        'destinations: for origin in [destination_top_raw, destination_bottom_raw] {
            let mut level_map = Vec::new();
            'levels: for level_i in 0..tunables.base_source_intra_rails {
                let level = base_source.intra_level_at_index(level_i);
                let mut attempts = vec![level.apply(origin)];
                for _ in 0..tunables.mine_further_attempts {
                    let further_endpoint = move_further_by_section(
                        area_min.point_center(),
                        *attempts.last().unwrap(),
                        true,
                        tunables,
                    );
                    attempts.push(further_endpoint);
                }

                let mut conflict_links = Vec::new();
                for attempt in &attempts {
                    match test(attempt) {
                        None => {
                            // endpoint is out of bounds, abandon the entire destination
                            continue 'destinations;
                        }
                        Some((false, new_conflict_links)) => {
                            conflict_links.extend(new_conflict_links);
                        }
                        Some((true, new_conflict_links)) => {
                            assert!(new_conflict_links.is_empty());
                            level_map.push(MineDestinationLevel {
                                level,
                                target: VPointDirectionQ(*attempt, direction),
                            });
                            continue 'levels;
                        }
                    }
                }

                // we just gathered conflicts
                if true {
                    // ignore?
                    warn!("Failing mine after {} attempts", attempts.len());
                    continue 'destinations;
                } else {
                    let mut debug_surface = surface.surface_copy();

                    // mega highlighter
                    debug_surface.pixels_mut_fn(|mut s| {
                        for bad in conflict_links {
                            s.change_pixels(bad.soda_area())
                                .find_empty_into(Pixel::Highlighter);
                        }

                        for (i, endpoint) in attempts.iter().enumerate() {
                            s.draw_text_at(
                                *endpoint,
                                &format!("d{i}"),
                                TextSize::small(),
                                Pixel::EdgeWall,
                            );
                        }

                        s
                            // .change_square(&VArea::from_radius(attempts[0], 200))
                            .change_square(area_min)
                            .find_empty_into(Pixel::SteelChest);

                        attempts.push(area_min.point_center());
                        s.change_pixels(attempts).stomp(Pixel::Water);

                        s.pixels().paint_pixel_colored_entire().save_to_oculante();
                    });

                    panic!("the further away pos doesn't work either?")
                }
            }
            assert!(!level_map.is_empty());

            let destination_level = MineDestination(level_map);

            destination_levels.push(destination_level);
        }
        destination_levels
    }

    pub fn for_level(&self, needle: &IntraLevel) -> VPointDirectionQ {
        self.0
            .iter()
            .find(|level| level.level == *needle)
            .unwrap_or_else(|| {
                for level in &self.0 {
                    error!("level {:?}", level.level);
                }
                panic!("level not found {needle:?}")
            })
            .target
    }
}

// enum Adjustment {
//     AdjustMore,
//     BadEndpoint,
//     Usable,
// }

#[derive(Serialize, Deserialize)]
pub struct DebugMinePatch {
    pub pixel: Pixel,
    pub points: Vec<VPoint>,
}

#[cfg(test)]
mod test {
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

        if 1 + 1 == 2 {
            // debug_draw_mine_links(&mut surface.pixels_mut(), [&mine]);
            panic!("uhh todo")
        }

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
