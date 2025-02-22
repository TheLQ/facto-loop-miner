use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::{BasicWatch, BasicWatchResult};
use crate::LOCALE;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::{RailHopeAppender, RailHopeLink};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_dual::{
    duals_into_single_vec, HopeDualLink, RailHopeDual, DUAL_RAIL_STEP, DUAL_RAIL_STEP_I32,
};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use num_format::ToFormattedString;
use pathfinding::prelude::astar_mori;
use std::time::Duration;
use tracing::warn;

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

    let total_watch = BasicWatch::start();
    let mut successor_sum = Duration::default();
    let mut res_sum = Duration::default();
    // ::<_, _, _, _, _, _, _, ParentProcessor>
    let pathfind = astar_mori(
        start_link,
        |head, processor, cost| {
            let watch = BasicWatch::start();
            let res = successors(
                surface,
                endpoints,
                head,
                processor,
                finding_limiter,
                tunables,
                &mut watch_data,
            );
            successor_sum += watch.duration();
            res
        },
        |_p| 1,
        |p| {
            // let watch = BasicWatch::start();
            let res = p == &end_link;
            // res_sum += watch.duration();
            res
            // p.start.distance_bird(&end_link.start) < 5.0
        },
        |processor, cur_link| {
            processor.total_links += 1;
        },
    );

    let success = pathfind.is_ok();

    warn!(
        "executions {} found {} nexts {} cost {} summed {} res {} total {} success {success}",
        watch_data.executions.to_formatted_string(&LOCALE),
        watch_data.found_successors.to_formatted_string(&LOCALE),
        BasicWatchResult(watch_data.nexts),
        BasicWatchResult(watch_data.cost),
        BasicWatchResult(successor_sum),
        BasicWatchResult(res_sum),
        total_watch
    );

    match pathfind {
        Ok((path, cost)) => MoriResult::Route {
            path: duals_into_single_vec(path),
            cost,
        },
        Err((dump, all)) => MoriResult::FailingDebug(
            duals_into_single_vec(dump.into_iter().map(|(v, (i, r))| v)),
            duals_into_single_vec(all),
        ),
    }
}

#[derive(Default)]
pub(crate) struct ParentProcessor {
    // parent_turns: usize,
    total_links: usize,
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
    FailingDebug(Vec<HopeLink>, Vec<HopeLink>),
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
        self.start.point().assert_step_rail();
        self.end.point().assert_step_rail();
    }
}

fn new_straight_link_from_vd(start: &VPointDirectionQ) -> HopeDualLink {
    let mut hope = RailHopeDual::new(
        start
            .point()
            .move_direction_int(start.direction(), -DUAL_RAIL_STEP_I32),
        *start.direction(),
        FacItemOutput::new_null().into_rc(),
    );
    hope.add_straight(DUAL_RAIL_STEP);
    let links = hope.into_links();
    let link = links.into_iter().next().unwrap();
    link.pos_next().assert_step_rail();
    link
}

fn successors(
    surface: &VSurface,
    segment_points: &PathSegmentPoints,
    head: &HopeDualLink,
    // path: &[&HopeLink],
    processor: &ParentProcessor,
    finding_limiter: &VArea,
    tune: &MoriTunables,
    watch_data: &mut WatchData,
) -> Vec<(HopeDualLink, u32)> {
    watch_data.executions += 1;
    // let head = path.first().unwrap();

    if processor.total_links > 500 {
        // warn!("too many links");
        return Vec::new();
    }

    let watch = BasicWatch::start();
    let nexts = [
        into_buildable_link(surface, finding_limiter, head.add_straight_section()),
        into_buildable_link(surface, finding_limiter, head.add_turn90(false)),
        into_buildable_link(surface, finding_limiter, head.add_turn90(true)),
    ];
    watch_data.nexts += watch.duration();

    let watch = BasicWatch::start();
    let mut successors = Vec::with_capacity(3);
    for next in nexts.into_iter().flatten() {
        let cost = calculate_cost_for_link(&next, segment_points, processor, tune);
        successors.push((next, cost));
    }
    watch_data.cost += watch.duration();

    watch_data.found_successors += successors.len();

    successors
}

fn into_buildable_link(
    surface: &VSurface,
    finding_limiter: &VArea,
    new_link: HopeDualLink,
) -> Option<HopeDualLink> {
    if !finding_limiter.contains_point(&new_link.pos_next()) {
        return None;
    }
    new_link.pos_start().assert_step_rail();
    let area = new_link.area();
    if surface.is_points_free_unchecked(&area) {
        Some(new_link)
    } else {
        None
    }
}
