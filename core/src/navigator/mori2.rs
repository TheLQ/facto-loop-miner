use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::{RailHopeAppender, RailHopeAppenderExt};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, RailHopeSingle};
use pathfinding::prelude::astar_mori;

const STRAIGHT_STEP_SIZE: usize = 1;

/// Pathfinder v1.2, Mori Calliope💀
///
/// astar powered pathfinding, now powered by fac-engine
///
/// Makes a dual rail + spacing, +6 straight or 90 degree turning, path of rail from start to end.
/// Without collisions into any point on the Surface.
pub fn mori2_start(
    surface: &VSurface,
    start: VPointDirectionQ,
    end: VPointDirectionQ,
) -> MoriResult {
    let pathfind_watch = BasicWatch::start();

    let endpoints = &PathSegmentPoints { start, end };
    endpoints.validate_positions();
    let start_link = new_straight_link_from_vd(&endpoints.start);
    let end_link = new_straight_link_from_vd(&endpoints.end);

    let tunables = MoriTunables::default();

    let pathfind = astar_mori(
        &start_link,
        |(successor_rail, path, _total_cost)| {
            // let (next, parents) = parents.split_last().unwrap();
            // assert_eq!(successor_rail, next);
            successors(surface, endpoints, &path, &tunables)
        },
        |_p| 1,
        |p| p == &end_link,
    );

    match pathfind {
        Ok((path, cost)) => MoriResult::Route { path, cost },
        Err((inner_map, parents)) => {
            let entries = parents.into_iter().map(|(node, _v)| node).collect();
            MoriResult::FailingDebug(entries)
        }
    }
}

pub struct PathSegmentPoints {
    start: VPointDirectionQ,
    pub(crate) end: VPointDirectionQ,
}

pub enum MoriResult {
    Route { path: Vec<HopeLink>, cost: u32 },
    FailingDebug(Vec<HopeLink>),
}

impl MoriResult {
    pub fn is_route(&self) -> bool {
        match &self {
            MoriResult::Route { .. } => true,
            MoriResult::FailingDebug(..) => false,
        }
    }
}

impl PathSegmentPoints {
    fn validate_positions(&self) {
        // self.start.point().assert_odd_16x16_position();
        // self.end.point().assert_odd_16x16_position();
    }
}

fn new_straight_link_from_vd(start: &VPointDirectionQ) -> HopeLink {
    let mut hope = RailHopeSingle::new(
        *start.point(),
        *start.direction(),
        FacItemOutput::new_null().into_rc(),
    );
    hope.add_straight(STRAIGHT_STEP_SIZE);
    let links = hope.into_links();
    links.into_iter().next().unwrap()
}

fn successors(
    surface: &VSurface,
    segment_points: &PathSegmentPoints,
    path: &[HopeLink],
    tune: &MoriTunables,
) -> Vec<(HopeLink, u32)> {
    let mut successors = Vec::new();
    let last = path.last().unwrap();

    let nexts = [
        into_buildable_link(surface, last.add_straight(STRAIGHT_STEP_SIZE)),
        into_buildable_link(surface, last.add_turn90(false)),
        into_buildable_link(surface, last.add_turn90(true)),
    ];
    for next in nexts {
        if let Some(next) = next {
            let cost = calculate_cost_for_link(&next, segment_points, path, tune);
            successors.push((next, cost));
        }
    }

    successors
}

fn into_buildable_link(surface: &VSurface, new_link: HopeLink) -> Option<HopeLink> {
    let area = link_area(surface, &new_link);
    if surface.is_points_free_unchecked(&area) {
        Some(new_link)
    } else {
        None
    }
}

fn link_area(surface: &VSurface, new_link: &HopeLink) -> Vec<VPoint> {
    Vec::new()
}

// struct LinkArea<'l>(&'l HopeSingleLink);
//
// impl LinkArea {
//     fn is_valid()
// }
