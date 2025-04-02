use crate::navigator::mine_permutate::CompletePlan;
use crate::navigator::mine_selector::MineSelectBatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use itertools::Itertools;
use std::borrow::Borrow;

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

pub(super) fn debug_draw_planned_destinations(
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
                pixels.push(*route.destination.point());
            }
            // pixels.push(*route.base_source.point());
        }
    }

    surface.set_pixels(Pixel::Highlighter, pixels).unwrap();
}

pub(super) fn draw_no_touching_zone(surface: &mut VSurface, batches: &[MineSelectBatch]) {
    for batch in batches {
        for mine in &batch.mines {
            surface.draw_square_area(&mine.area, Pixel::MineNoTouch);
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
