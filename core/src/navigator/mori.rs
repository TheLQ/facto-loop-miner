use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use crate::LOCALE;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::{RailHopeAppender, RailHopeAppenderExt};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, RailHopeSingle};
use facto_loop_miner_fac_engine::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;
use num_format::ToFormattedString;
use pathfinding::prelude::astar_mori;
use std::time::Duration;
use tracing::{info, warn};

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
    finding_limiter: &VArea,
) -> MoriResult {
    let pathfind_watch = BasicWatch::start();

    let endpoints = &PathSegmentPoints { start, end };
    endpoints.validate_positions();
    let start_link = new_straight_link_from_vd(&endpoints.start);
    let end_link = new_straight_link_from_vd(&endpoints.end);

    let tunables = &surface.tunables().mori;
    // info!("tunables {:?}", tunables);

    let mut watch_data = WatchData::default();

    let pathfind = astar_mori(
        &start_link,
        |_redundant_head, path, cost| {
            successors(
                surface,
                endpoints,
                &path,
                finding_limiter,
                tunables,
                &mut watch_data,
            )
        },
        |_p| 1,
        |p| p == &end_link,
    );

    let success = pathfind.is_some();
    warn!(
        "executions {} found {} nexts {}ms cost {}ms success {success}",
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
        start
            .point()
            .move_direction_int(start.direction(), -RAIL_STRAIGHT_DIAMETER_I32),
        *start.direction(),
        FacItemOutput::new_null().into_rc(),
    );
    hope.add_straight(1);
    let links = hope.into_links();
    links.into_iter().next().unwrap()
}

fn successors(
    surface: &VSurface,
    segment_points: &PathSegmentPoints,
    path: &[&HopeLink],
    finding_limiter: &VArea,
    tune: &MoriTunables,
    watch_data: &mut WatchData,
) -> Vec<(HopeLink, u32)> {
    watch_data.executions += 1;
    let head = path.first().unwrap();

    let watch = BasicWatch::start();
    let nexts = [
        into_buildable_link(
            surface,
            &finding_limiter,
            head.add_straight(tune.straight_section_size),
        ),
        into_buildable_link(surface, &finding_limiter, head.add_turn90(false)),
        into_buildable_link(surface, &finding_limiter, head.add_turn90(true)),
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

fn into_buildable_link(
    surface: &VSurface,
    finding_limiter: &VArea,
    new_link: HopeLink,
) -> Option<HopeLink> {
    if !finding_limiter.contains_point(&new_link.next_straight_position()) {
        return None;
    }
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
