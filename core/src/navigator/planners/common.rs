use crate::navigator::mine_permutate::CompletePlan;
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
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

fn max_no_touching_zone(surface: &VSurface, area: &VArea) -> VArea {
    if 1 + 1 == 2 {
        return area.clone();
    }
    VArea::from_arbitrary_points_pair(
        area.point_top_left().move_round_rail_down(),
        area.point_bottom_right().move_round_rail_up(),
    )
    .normalize_within_radius(surface.get_radius_i32() - 1)
}
