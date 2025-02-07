use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::{RailHopeAppender, RailHopeAppenderExt};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, RailHopeSingle};
use num_format::ToFormattedString;
use pathfinding::prelude::{astar, astar_mori};
use std::time::Duration;
use tracing::{info, warn};

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
    info!("tunables {:?}", tunables);

    let mut watch_data = WatchData::default();

    let pathfind = astar_mori(
        &start_link,
        |_redundant_head, path, cost| {
            successors(surface, endpoints, &path, &tunables, &mut watch_data)
        },
        |_p| 1,
        |p| p == &end_link,
    );

    warn!(
        "executions {} found {} nexts {}ms cost {}ms",
        watch_data.executions.to_formatted_string(&LOCALE),
        watch_data.found_successors.to_formatted_string(&LOCALE),
        watch_data.nexts.as_millis().to_formatted_string(&LOCALE),
        watch_data.cost.as_millis().to_formatted_string(&LOCALE)
    );

    match pathfind {
        Some((path, cost)) => MoriResult::Route { path, cost },
        None => MoriResult::FailingDebug(Vec::new()),
        // Err((inner_map, parents)) => {
        //     let entries = parents.into_iter().map(|(node, _v)| node).collect();
        //     MoriResult::FailingDebug(entries)
        // }
    }
}

#[derive(Default)]
struct WatchData {
    nexts: Duration,
    cost: Duration,
    executions: usize,
    found_successors: usize,
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
    path: &[&HopeLink],
    tune: &MoriTunables,
    watch_data: &mut WatchData,
) -> Vec<(HopeLink, u32)> {
    watch_data.executions += 1;
    let head = path.first().unwrap();

    let watch = BasicWatch::start();
    let nexts = [
        into_buildable_link(surface, head.add_straight(STRAIGHT_STEP_SIZE)),
        into_buildable_link(surface, head.add_turn90(false)),
        into_buildable_link(surface, head.add_turn90(true)),
    ];
    watch_data.nexts += watch.duration();

    let watch = BasicWatch::start();
    let mut successors = Vec::with_capacity(3);
    for next in nexts.into_iter().flatten() {
        let cost = calculate_cost_for_link(&next, segment_points, path, tune);
        successors.push((next, cost));
    }
    watch_data.cost += watch.duration();

    watch_data.found_successors += successors.len();

    successors
}

fn into_buildable_link(surface: &VSurface, new_link: HopeLink) -> Option<HopeLink> {
    if true {
        return Some(new_link);
    }
    let area = new_link.area();
    if surface.is_points_free_unchecked(&area) {
        Some(new_link)
    } else {
        None
    }
}
