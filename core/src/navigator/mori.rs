use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::navigator::planners::debug_draw_segment;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_common::duration::{BasicWatch, BasicWatchResult};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::{sodas_to_links, HopeSodaLink};
use num_format::ToFormattedString;
use pathfinding::prelude::astar_mori;
use std::time::Duration;
use tracing::info;

/// Pathfinder v1.2, Mori Calliope💀
///
/// astar powered pathfinding, now powered by fac-engine
///
/// Makes a dual rail + spacing, +6 straight or 90 degree turning, path of rail from start to end.
/// Without collisions into any point on the Surface.
pub fn mori2_start(surface: &VSurface, endpoints: VSegment, finding_limiter: &VArea) -> MoriResult {
    let is_possible = endpoints.end.point() - endpoints.start.point();
    is_possible.assert_step_rail();

    let start_link = new_straight_link_from_vd(&endpoints.start);
    let end_link = new_straight_link_from_vd(&endpoints.end);

    let tunables = &surface.tunables().mori;
    let mut watch_data = WatchData::default();

    let total_watch = BasicWatch::start();
    let mut successor_sum = Duration::default();
    let res_sum = Duration::default();
    let pathfind = astar_mori::<_, _, _, _, _, _, _, 5>(
        start_link.clone(),
        |head| {
            let watch = BasicWatch::start();
            let res = successors(
                surface,
                &endpoints,
                head,
                // processor,
                finding_limiter,
                tunables,
                &mut watch_data,
            );
            successor_sum += watch.duration();
            res
        },
        |_p| 0,
        |p| {
            // let watch = BasicWatch::start();
            let res = p == &end_link;
            // res_sum += watch.duration();
            res
            // p.start.distance_bird(&end_link.start) < 5.0
        },
        |path| {
            // sequential compare
            path.sort_by_key(|v| v.pos_start());
            let mut i = 0;
            while i + 1 < path.len() {
                if path[i].pos_start() == path[i + 1].pos_start() {
                    return false;
                }
                i += 1;
            }
            true
        },
    );

    let success = pathfind.is_ok();

    let depth: String = match &pathfind {
        Err(_) => "FAIL".into(),
        Ok((links, cost)) => format!("{}", links.len()),
    };
    info!(
        " - {:>9} executions {:>9} found {:>8} nexts {:>6} cost {:>6} summed {:>5} res {:>8} total  {success:>5} success {depth:>4} depth",
        watch_data.executions.to_formatted_string(&LOCALE),
        watch_data.found_successors.to_formatted_string(&LOCALE),
        BasicWatchResult(watch_data.nexts),
        BasicWatchResult(watch_data.cost),
        BasicWatchResult(successor_sum),
        BasicWatchResult(res_sum),
        total_watch
    );
    // if let Err(_) = &pathfind {
    //     let mut new_surface = surface.clone();
    //     debug_draw_segment(&mut new_surface, endpoints);
    //     new_surface.save_pixel_to_oculante();
    //
    //     exit(0);
    // }

    match pathfind {
        Ok((path, cost)) => {
            assert!(
                path.first().unwrap() == &start_link,
                "path should start with start link"
            );
            assert!(
                path.last().unwrap() == &end_link,
                "path should ebd with start link"
            );
            MoriResult::Route {
                // path: duals_into_single_vec(path),
                path: sodas_to_links(path).collect(),
                cost,
            }
        }
        Err((_dump, _all)) => MoriResult::FailingDebug(
            // duals_into_single_vec(dump.into_iter().map(|(v, (i, r))| v)),
            // duals_into_single_vec(all),
            Vec::new(),
            Vec::new(),
        ),
    }
}

#[derive(Default)]
struct WatchData {
    nexts: Duration,
    cost: Duration,
    executions: usize,
    found_successors: usize,
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

fn new_straight_link_from_vd(start: &VPointDirectionQ) -> HopeSodaLink {
    HopeSodaLink::new_soda_straight(start.0, start.1)
}

fn successors(
    surface: &VSurface,
    segment_points: &VSegment,
    head: &HopeSodaLink,
    finding_limiter: &VArea,
    tune: &MoriTunables,
    watch_data: &mut WatchData,
) -> Vec<(HopeSodaLink, u32)> {
    watch_data.executions += 1;

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
        let cost = calculate_cost_for_link(&next, segment_points, tune);
        successors.push((next, cost));
    }
    watch_data.cost += watch.duration();

    watch_data.found_successors += successors.len();

    successors
}

fn into_buildable_link(
    surface: &VSurface,
    finding_limiter: &VArea,
    new_link: HopeSodaLink,
) -> Option<HopeSodaLink> {
    // todo: fix the limiter and just check center
    if !new_link
        .corners()
        .iter()
        .all(|v| finding_limiter.contains_point(v))
    {
        return None;
    }
    // new_link.pos_start().assert_step_rail();
    let area = new_link.area_vec();
    assert_eq!(area.len(), 104);
    if surface.is_points_free_unchecked(&area) {
        Some(new_link)
    } else {
        None
    }
}
