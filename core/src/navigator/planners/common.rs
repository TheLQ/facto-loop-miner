use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionRoute, FailingMeta};
use crate::navigator::mine_permutate::CompletePlan;
use crate::navigator::mine_selector::MineSelectBatch;
use crate::state::tuneables::{ChunkValue, MoriTunables, Tunables};
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{
    VSurfacePixelAsVs, VSurfacePixelAsVsMut, VSurfacePixelMut, VSurfaceRailMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_THREE, VPoint};
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::HopeSodaLink;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, warn};

pub struct PathingTunables {
    base_chunks: ChunkValue,
    mori: MoriTunables,
}

impl PathingTunables {
    pub fn from_tunables(tunables: &Tunables) -> Self {
        Self {
            base_chunks: tunables.base.base_chunks,
            mori: tunables.mori.clone(),
        }
    }

    pub fn base_chunks(&self) -> &ChunkValue {
        &self.base_chunks
    }

    pub fn mori(&self) -> &MoriTunables {
        &self.mori
    }
}

/*
pub(super) fn debug_draw_base_sources(
    surface: &mut VSurface,
    batches: impl IntoIterator<Item = impl Borrow<MineSelectBatch>>,
) {
    let mut pixels = Vec::new();
    for batch in batches {
        let batch = batch.borrow();
        let total_routes = batch.mines.len();

        let mut borrow = batch.base_sources.as_ref().borrow_mut();
        for _ in 0..total_routes {
            // can't consume the iterator with take() :-(
            pixels.push(borrow.next().unwrap().origin.point().clone());
        }
        // pixels.extend(borrow.take(total_routes).map(|v| v.point().clone()));
    }
    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
}
*/

pub(super) fn debug_draw_complete_plan(
    surface: &mut VSurfacePixelMut,
    CompletePlan {
        sequences,
        base_sources,
    }: CompletePlan,
) {
    let mut pixels = Vec::new();
    let route_len = sequences[0].routes.len();
    for sequence in &sequences {
        assert_eq!(sequence.routes.len(), route_len);
    }

    // will dupe
    for sequence in sequences {
        for route in &sequence.routes {
            let VSegment { start, end } = route.segment;
            pixels.push(*start.point());
            pixels.push(*end.point());
        }
    }
    base_sources.borrow_mut().advance_by(route_len).unwrap();

    surface.change_pixels(pixels).stomp(Pixel::Highlighter)
}

pub fn debug_draw_segment(surface: &mut VSurfacePixelMut, segment: VSegment) {
    let VSegment { start, end } = segment;
    let mut positions = Vec::new();
    positions.extend(start.point().get_entity_area_3x3());
    positions.extend((start.point() - &VPOINT_THREE).get_entity_area_3x3());
    positions.extend(end.point().get_entity_area_3x3());
    positions.extend((end.point() - &VPOINT_THREE).get_entity_area_3x3());
    // let positions = vec![*start.point(), *end.point()];
    surface.change_pixels(positions).stomp(Pixel::Highlighter);
}

pub(super) fn debug_draw_failing_mines<'a>(
    surface: &mut VSurfacePixelMut,
    mines: impl IntoIterator<Item = &'a MineLocation>,
) {
    let mut seen_mines: Vec<&VArea> = Vec::new();
    let mut destinations = Vec::new();
    for mine in mines {
        let mine_area = &mine.area_buffered();
        if seen_mines.contains(mine_area) {
            continue;
        }
        surface
            .change_square(mine_area)
            .find_into(Pixel::MineNoTouch, Pixel::Highlighter);
        seen_mines.push(mine_area);

        for destination in mine.destinations() {
            destinations.push(destination.0)
        }
    }
    surface.change_pixels(destinations).stomp(Pixel::EdgeWall);
}

pub fn debug_failing(
    surface: &mut VSurfaceRailMut,
    FailingMeta {
        found_paths,
        mut all_routes,
        astar_err,
    }: FailingMeta,
) {
    // draw all endpoints
    surface
        .pixels_mut()
        .change_pixels(
            all_routes
                .iter()
                .flat_map(|v| [v.segment.start, v.segment.end])
                .map(|v| *v.point())
                .collect(),
        )
        .stomp(Pixel::Highlighter);

    // split all_routes
    let routes_found: Vec<ExecutionRoute> = all_routes
        .extract_if(.., |v| {
            found_paths
                .iter()
                .any(|found_path| found_path.location == v.location)
        })
        .collect();
    let routes_notfound = all_routes;

    // draw paths (now that we don't need it anymore)
    error!(
        "failed to pathfind but writing {} paths anyway",
        found_paths.len()
    );
    for path in found_paths {
        // path.
        surface.add_mine_path_with_pixel(path, Pixel::Water);
    }

    warn!(
        "Found {} notfound {}",
        routes_found.len(),
        routes_notfound.len()
    );
    // draw found
    for route in routes_found {
        route
            .location
            .draw_area_buffered_highlight_pixel(&mut surface.pixels_mut(), Pixel::Stone);
    }
    // draw not found
    for route in routes_notfound {
        warn!("failing at {:?}", route.location.area_buffered());
        route
            .location
            .draw_area_buffered_highlight_pixel(&mut surface.pixels_mut(), Pixel::SteelChest);
    }
}

pub(super) fn draw_prep(surface: &mut VSurfacePixelMut, batches: &[MineSelectBatch]) {
    draw_prep_mines(
        surface,
        batches.iter().flat_map(|v| &v.mines),
        &batches[0].base_sources,
    )
}

pub(super) fn draw_prep_mines(
    surface: &mut VSurfacePixelMut,
    mines: impl IntoIterator<Item = impl Borrow<MineLocation>>,
    base_sources: &Rc<RefCell<BaseSourceEighth>>,
) {
    for mine in mines {
        mine.borrow().draw_area_buffered(surface);
        // mine.borrow().draw_area_buffered_to_no_touch(surface);
    }

    // stop routes going backwards right behind the start
    let radius = surface.pixels().get_radius_i32();

    let base_sources = base_sources.as_ref().borrow();
    let anti_backside_x = base_sources.peek_single().origin.point().x() - 1;
    let anti_backside_points = (-(radius - 1)..radius)
        .map(|i| VPoint::new(anti_backside_x, i))
        .collect_vec();
    surface
        .change_pixels(anti_backside_points)
        .stomp(Pixel::MineNoTouch)
}

pub fn debug_draw_mine_index_labels(
    surface: &mut VSurfacePixelMut,
    mines: impl IntoIterator<Item = impl Borrow<MineLocation>>,
) {
    for (i, mine) in mines.into_iter().enumerate() {
        let mine = mine.borrow();
        surface.draw_text_at(mine.area_min().point_center(), &i.to_string());
    }
}

pub fn debug_draw_mine_links(
    surface: &mut VSurfacePixelMut,
    mines: impl IntoIterator<Item = impl Borrow<MineLocation>>,
) {
    for mine in mines {
        let mine = mine.borrow();
        for destination in mine.destinations() {
            let link = HopeSodaLink::new_soda_straight(destination.0, destination.1);
            surface
                .change_pixels(
                    link.area_vec()
                        .into_iter()
                        .filter(|p| !surface.pixels().is_point_out_of_bounds(p))
                        .collect(),
                )
                .stomp(Pixel::Rail);
        }
    }
}

/*
pub fn debug_conflict_no_touching(
    surface: &mut VSurface,
    batches: &[MineSelectBatch],
) -> Result<(), ()> {
    let mut seen_points: Vec<(&MineLocation, VArea)> = Vec::new();
    let mut fail = false;
    for batch in batches {
        for mine in &batch.mines {
            let area = max_no_touching_zone(surface, &mine.area);
            for point in area.get_points() {
                if let Some((loc, _)) = seen_points.iter().find(|(_, v)| v.contains_point(&point)) {
                    surface.draw_square_area_replacing(
                        &max_no_touching_zone(surface, &area),
                        Pixel::MineNoTouch,
                        Pixel::Highlighter,
                    );
                    surface.draw_square_area_replacing(
                        &max_no_touching_zone(surface, &loc.area),
                        Pixel::MineNoTouch,
                        Pixel::EdgeWall,
                    );
                }
                fail = true;
            }
            seen_points.push((mine, area));
        }
    }
    if fail {
        Err(())
    } else {
        Ok(())
    }
}
*/
