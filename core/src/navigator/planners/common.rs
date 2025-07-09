use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionRoute, FailingMeta};
use crate::navigator::mine_permutate::CompletePlan;
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::err::VResult;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, warn};
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
    surface: &mut VSurface,
    CompletePlan {
        sequences,
        base_sources,
    }: CompletePlan,
) -> VResult<()> {
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

    surface.set_pixels(Pixel::Highlighter, pixels)
}

pub fn debug_draw_segment(surface: &mut VSurface, segment: VSegment) {
    let VSegment { start, end } = segment;
    surface
        .set_pixels(Pixel::Highlighter, vec![*start.point(), *end.point()])
        .unwrap();
}

pub(super) fn debug_draw_failing_mines<'a>(
    surface: &mut VSurface,
    routes: impl IntoIterator<Item = &'a ExecutionRoute>,
) {
    let mut seen_mines: Vec<&VArea> = Vec::new();
    let mut destinations = Vec::new();
    for route in routes {
        let mine_area = &route.location.area_buffered();
        if seen_mines.contains(mine_area) {
            continue;
        }
        surface.draw_square_area_replacing(mine_area, Pixel::MineNoTouch, Pixel::Highlighter);
        seen_mines.push(mine_area);

        let destination = *route.segment.end.point();
        if !destinations.contains(&destination) {
            destinations.push(destination);
        }
    }
    surface.set_pixels(Pixel::EdgeWall, destinations).unwrap();
}

pub fn debug_failing(
    surface: &mut VSurface,
    FailingMeta {
        found_paths,
        mut all_routes,
        debug_tree,
    }: FailingMeta,
) {
    // draw all endpoints
    surface
        .set_pixels(
            Pixel::Highlighter,
            all_routes
                .iter()
                .flat_map(|v| [v.segment.start, v.segment.end])
                .map(|v| *v.point())
                .collect(),
        )
        .unwrap();

    // split all_routes
    let routes_found: Vec<ExecutionRoute> = all_routes
        .extract_if(.., |v| {
            found_paths
                .iter()
                .any(|found_path| found_path.mine_base == v.location)
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
        surface
            .add_mine_path_with_pixel(path, Pixel::Water)
            .unwrap();
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
            .draw_area_buffered_replacing(surface, Pixel::Stone);
    }
    // draw not found
    for route in routes_notfound {
        warn!("failing at {:?}", route.location.area_buffered());
        route
            .location
            .draw_area_buffered_replacing(surface, Pixel::SteelChest);
    }
}

pub(super) fn draw_prep(surface: &mut VSurface, batches: &[MineSelectBatch]) {
    draw_prep_mines(
        surface,
        batches.into_iter().flat_map(|v| &v.mines),
        &batches[0].base_sources,
    )
}

pub(super) fn draw_prep_mines(
    surface: &mut VSurface,
    mines: impl IntoIterator<Item = impl Borrow<MineLocation>>,
    base_sources: &Rc<RefCell<BaseSourceEighth>>,
) {
    for mine in mines {
        mine.borrow().draw_area_buffered(surface);
        // mine.borrow().draw_area_buffered_to_no_touch(surface);
    }

    // stop routes going backwards right behind the start
    let radius = surface.get_radius_i32();

    let base_sources = base_sources.as_ref().borrow();
    let anti_backside_x = base_sources.peek_single().origin.point().x() - 1;
    let anti_backside_points = (-(radius - 1)..radius)
        .map(|i| VPoint::new(anti_backside_x, i))
        .collect_vec();
    surface
        .set_pixels(Pixel::MineNoTouch, anti_backside_points)
        .unwrap()
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
