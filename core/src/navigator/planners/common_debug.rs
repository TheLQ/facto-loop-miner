use crate::navigator::base_source::BaseSourceEighth;
use crate::navigator::mine_executor::{ExecutionSequence, ExecutionSequenceParts, FailingMeta};
use crate::opencv::TextSize;
use crate::state::tuneables::{
    AltareTunables, ChunkValue, MoriTunables, PathCommonTunables, Tunables,
};
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::{MineDraw, MineLocation, MineLocationResolver};
use crate::surfacev::vsurface::{
    MineRef, VSurfaceMineAsVs, VSurfaceMineAsVsMut, VSurfaceMineMut, VSurfacePixelAsVs, VSurfacePixelAsVsMut, VSurfacePixelMut,
    VSurfaceRailAsVsMut,
};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_THREE, VPoint};
use facto_loop_miner_fac_engine::common::vpoint_direction::VSegment;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::HashMap;
use tracing::{error, warn};

pub struct PathingTunables {
    base_chunks: ChunkValue,
    mori: MoriTunables,
    altare: AltareTunables,
    common: PathCommonTunables,
}

impl PathingTunables {
    pub fn from_tunables(tunables: &Tunables) -> Self {
        Self {
            base_chunks: tunables.base.base_chunks,
            mori: tunables.mori.clone(),
            altare: tunables.altare.clone(),
            common: tunables.path_common.clone(),
        }
    }

    pub fn base_chunks(&self) -> &ChunkValue {
        &self.base_chunks
    }

    pub fn mori(&self) -> &MoriTunables {
        &self.mori
    }

    pub fn altare(&self) -> &AltareTunables {
        &self.altare
    }

    pub fn path_common(&self) -> &PathCommonTunables {
        &self.common
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

pub struct Debugger<'s, S>(pub &'s mut S, pub &'static str);

impl<'s, S: VSurfacePixelAsVsMut + VSurfaceMineAsVs> Debugger<'s, S> {
    pub fn sequences(
        &mut self,
        sequences: Vec<ExecutionSequence>,
        base_source: &BaseSourceEighth,
    ) -> &mut Self {
        // will dupe
        let mut pixels = Vec::new();
        for sequence in sequences {
            for (i, route) in sequence.routes().iter().enumerate() {
                let source = base_source.peek_after(i);
                let VSegment { start, end } = route
                    .segment_for_source(&source, MineLocationResolver::Surface(self.0.mines()));
                pixels.push(start.point());
                pixels.push(end.point());
            }
        }
        self.0
            .pixels_mut_fn(|mut s| s.change_pixels(pixels).stomp(Pixel::Highlighter));
        self
    }

    pub fn starts_numbered(&mut self, start_points: &BaseSourceEighth, amount: usize) -> &mut Self {
        let surface = &mut self.0.pixels_mut_ref();
        let mut pixels = Vec::new();
        for (i, base_source) in (0..amount).map(|v| start_points.peek_after(v)).enumerate() {
            let point = base_source.origin.point();
            surface.draw_text_at(point, &i.to_string(), TextSize::small(), Pixel::SteelChest);
            pixels.push(point);
        }
        surface.change_pixels(pixels).stomp(Pixel::Highlighter);
        self
    }

    pub fn wasteds(&mut self, wasteds: HashMap<Vec<VPoint>, usize>) -> &mut Self {
        let surface = &mut self.0.pixels_mut_ref();
        for (wasted, count) in wasteds {
            let center = VArea::from_arbitrary_points(&wasted).point_center();
            surface.change_pixels(wasted).stomp(Pixel::Highlighter);
            surface.draw_text_at(
                center,
                &count.to_string(),
                TextSize::small(),
                Pixel::EdgeWall,
            );
        }
        self
    }
}

impl<'s, S: VSurfaceMineAsVs + VSurfacePixelAsVsMut> Debugger<'s, S> {
    pub fn mines(
        &mut self,
        mines: impl IntoIterator<Item = MineRef>,
        base_source: &BaseSourceEighth,
    ) -> &mut Self {
        let mut seen_mines: Vec<VArea> = Vec::new();
        let mut destinations = Vec::new();
        for mine in mines {
            let mine_area = {
                let mine = mine.resolve_mine_surface(self.0.mines());
                let mine_area = mine.area_buffered().clone();
                if seen_mines.contains(&mine_area) {
                    continue;
                }
                mine_area
            };
            self.0
                .pixels_mut_ref()
                .change_square(&mine_area)
                .find_into(Pixel::MineNoTouch, Pixel::Highlighter);
            seen_mines.push(mine_area);

            for destination in mine.resolve_mine_surface(self.0.mines()).destinations() {
                tracing::trace!("destination {:?}", destination);
                // destinations.push(destination.0)
                let endpoint = destination.for_level(&base_source.intra_level_at_index(0));
                destinations.extend(VArea::from_radius(endpoint.point(), 3).get_points());
            }
        }
        self.0
            .pixels_mut_ref()
            .change_pixels(destinations)
            .stomp(Pixel::EdgeWall);
        self
    }
}

impl<'s, S: VSurfacePixelAsVsMut + VSurfaceRailAsVsMut + VSurfaceMineAsVsMut> Debugger<'s, S> {
    pub fn fail_mine_color_and_best_routes(
        &mut self,
        FailingMeta {
            sequence,
            failing_sequence,
            found_paths,
            cause: _,
        }: FailingMeta,
    ) -> &mut Self {
        warn!("debug routes_found_notfound for {}", self.1);

        error!(
            "failed to pathfind but writing {} paths anyway",
            found_paths.len()
        );
        self.0.rails_mut_fn(|mut s| {
            for found_path in found_paths {
                s.add_mine_path_with_pixel(found_path, Pixel::Water);
            }
        });

        let ExecutionSequenceParts { pass, fail } = sequence.split_routes_from(failing_sequence);
        warn!("pass {} fail {}", pass.len(), fail.len());
        for route in pass {
            self.0.mines_mut_ref().draw_mine(
                route.destination.mine_ref(),
                MineDraw::HighlightBufferedMain,
            );

            let mine = route
                .destination
                .mine_ref()
                .resolve_mine_surface(self.0.mines());
            warn!("pass at {:?}", mine.area_buffered());
        }
        for route in fail {
            self.0
                .mines_mut_ref()
                .draw_mine(route.destination.mine_ref(), MineDraw::HighlightBufferedAlt);

            let mine = route
                .destination
                .mine_ref()
                .resolve_mine_surface(self.0.mines());
            warn!("fail at {:?}", mine.area_buffered());
        }
        self
    }
}

fn debug_draw_segment(surface: &mut VSurfacePixelMut, segment: VSegment) {
    let VSegment { start, end } = segment;
    let mut positions = Vec::new();
    positions.extend(start.point().get_entity_area_3x3());
    positions.extend((start.point() - VPOINT_THREE).get_entity_area_3x3());
    positions.extend(end.point().get_entity_area_3x3());
    positions.extend((end.point() - VPOINT_THREE).get_entity_area_3x3());
    // let positions = vec![*start.point(), *end.point()];
    surface.change_pixels(positions).stomp(Pixel::Highlighter);
}

// pub(super) fn draw_prep(
//     surface: &mut VSurfacePatchMut,
//     batches: &[MineSelectBatch],
//     base_sources: &BaseSourceEighth,
// ) {
//     todo!("batches?");
//     draw_prep_mines(surface, base_sources)
// }

pub(super) fn draw_prep_mines<'plan_mine>(
    mut surface: VSurfaceMineMut,
    base_sources: &BaseSourceEighth,
) {
    for mine in surface.mines().all_mines_refs_iter() {
        surface.draw_mine(mine, MineDraw::ChangeBuffered);
    }

    // stop routes going backwards right behind the start
    let radius = surface.pixels().get_radius_i32();

    let anti_backside_x = base_sources.peek_single().origin.point().x() - (SECTION_POINTS_I32 / 2);
    let anti_backside_points = (-(radius - 1)..radius)
        .map(|i| VPoint::new(anti_backside_x, i))
        .collect_vec();
    surface.pixels_mut_fn(|mut s| {
        s.change_pixels(anti_backside_points)
            .stomp(Pixel::MineNoTouch)
    })
}

pub fn debug_draw_mine_index_labels(
    surface: &mut VSurfacePixelMut,
    mines: impl IntoIterator<Item = impl Borrow<MineLocation>>,
) {
    for (i, mine) in mines.into_iter().enumerate() {
        let mine = mine.borrow();
        surface.draw_text_at(
            mine.area_min().point_center(),
            &i.to_string(),
            TextSize::default(),
            Pixel::Highlighter,
        );
    }
}

/*
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
*/

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
