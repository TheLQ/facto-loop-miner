use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::ExecutionRoute;
use crate::navigator::mine_permutate::CompletePlan;
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
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

pub(super) fn debug_draw_complete_plans(
    surface: &mut VSurface,
    plans: impl IntoIterator<Item = impl Borrow<CompletePlan>>,
) {
    let mut pixels = Vec::new();
    for plan in plans {
        let CompletePlan {
            sequences,
            base_sources,
        } = plan.borrow();
        // will dupe
        for sequence in sequences {
            for route in &sequence.routes {
                let VSegment { start, end } = route.segment;
                pixels.push(*start.point());
                pixels.push(*end.point());
                // pixels.push(*route.destination.point());
            }
        }
    }

    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
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
