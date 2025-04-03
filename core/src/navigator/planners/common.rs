use crate::navigator::mine_permutate::{CompletePlan, PlannedRoute};
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MineLocation;
use crate::surfacev::vsurface::{RemovedEntity, VSurface};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use itertools::Itertools;
use std::borrow::Borrow;
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
                let base_source_entry = base_sources.borrow_mut().next().unwrap();
                let VSegment { start, end } = base_source_entry.route_to_segment(route);
                pixels.push(*start.point());
                pixels.push(*end.point());
            }
        }
    }

    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
}

pub(super) fn debug_draw_failing_mines(
    surface: &mut VSurface,
    routes: impl IntoIterator<Item = impl Borrow<PlannedRoute>>,
) {
    let mut seen_mines = Vec::new();
    let mut destinations = Vec::new();
    for route in routes {
        let route = route.borrow();

        let mine_area = &route.location.area;
        if seen_mines.contains(mine_area) {
            continue;
        }
        surface.draw_square_area_replacing(mine_area, Pixel::MineNoTouch, Pixel::Highlighter);
        seen_mines.push(mine_area.clone());

        let destination = *route.destination.point();
        if !destinations.contains(&destination) {
            destinations.push(destination);
        }
    }
    surface.set_pixels(Pixel::EdgeWall, destinations).unwrap();
}

pub(super) fn draw_no_touching_zone(surface: &mut VSurface, batches: &[MineSelectBatch]) {
    for batch in batches {
        for mine in &batch.mines {
            surface.draw_square_area(
                &max_no_touching_zone(surface, &mine.area),
                Pixel::MineNoTouch,
            );
        }
    }

    // stop routes going backwards right behind the start
    let radius = surface.get_radius_i32();

    let base_sources = batches[0].base_sources.as_ref().borrow();
    let anti_backside_x = base_sources.peek_single().origin.point().x() - 1;
    let anti_backside_points = (-(radius - 1)..radius)
        .map(|i| VPoint::new(anti_backside_x, i))
        .collect_vec();
    surface
        .set_pixels(Pixel::MineNoTouch, anti_backside_points)
        .unwrap()
}

pub(super) fn draw_active_no_touching_zone(
    surface: &mut VSurface,
    location: &MineLocation,
) -> RemovedEntity {
    let needle = location.area.point_top_left();
    let existing_pixel = surface.get_pixel(needle);
    assert_eq!(existing_pixel, Pixel::MineNoTouch, "at {needle}");

    let new_points = location
        .area
        .get_points()
        .into_iter()
        .filter(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch | Pixel::Empty))
        .collect_vec();
    surface.set_pixel_entity_swap(surface.get_pixel_entity_id_at(&needle), new_points, false)
}

pub(super) fn draw_restored_no_touching_zone(
    surface: &mut VSurface,
    mut removed_entity: RemovedEntity,
) -> RemovedEntity {
    removed_entity
        .points
        .retain(|p| matches!(surface.get_pixel(p), Pixel::MineNoTouch | Pixel::Empty));
    surface.set_pixel_entity_swap(removed_entity.entity_id, removed_entity.points, false)
}

fn max_no_touching_zone(surface: &VSurface, area: &VArea) -> VArea {
    // if 1 + 1 == 2 {
    //     return area.clone();
    // }
    VArea::from_arbitrary_points_pair(
        area.point_top_left().move_round_rail_down(),
        area.point_bottom_right().move_round_rail_up(),
    )
    .normalize_within_radius(surface.get_radius_i32() - 1)
}
