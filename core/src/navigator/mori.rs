use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::state::tuneables::MoriTunables;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::{BasicWatch, BasicWatchResult};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPoint, VPointSugar, VPOINT_ZERO};
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{HopeLink, SECTION_POINTS_I32};
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::{sodas_to_links, HopeSodaLink};
use facto_loop_miner_fac_engine::game_entities::direction::FacDirectionQuarter;
use num_format::ToFormattedString;
use pathfinding::prelude::{astar_mori, astar_mori2};
use std::time::Duration;
use tracing::{info, trace};

/// Pathfinder v1.2, Mori Calliope💀
///
/// astar powered pathfinding, now powered by fac-engine
///
/// Makes a dual rail + spacing, +6 straight or 90 degree turning, path of rail from start to end.
/// Without collisions into any point on the Surface.
pub fn mori2_start(surface: &VSurface, endpoints: VSegment, finding_limiter: &VArea) -> MoriResult {
    let start_link = new_straight_link_from_vd(&endpoints.start);
    let end_link = new_straight_link_from_vd(&endpoints.end);

    let tunables = &surface.tunables().mori;
    let mut watch_data = WatchData::default();

    let dummy_processor = ParentProcessor::default();

    let total_watch = BasicWatch::start();
    let mut successor_sum = Duration::default();
    let res_sum = Duration::default();
    // ::<_, _, _, _, _, _, _, ParentProcessor>
    let pathfind = astar_mori2(
        &start_link.pos_next(),
        |head, parent| {
            let watch = BasicWatch::start();
            let res = successors(
                surface,
                &endpoints,
                head,
                parent,
                &dummy_processor,
                finding_limiter,
                tunables,
                &mut watch_data,
            );
            trace!("writing {}", res.len());
            successor_sum += watch.duration();
            res
        },
        |_p| 0,
        |p| {
            // let watch = BasicWatch::start();
            let res = p == &end_link.pos_next();
            // res_sum += watch.duration();
            res
            // p.start.distance_bird(&end_link.start) < 5.0
        },
        // |processor, _cur_link| {
        //     processor.total_links += 1;
        // },
    );

    let success = true; //pathfind.is_ok();

    info!(
        " - {:>9} executions {:>9} found {:>8} nexts {:>6} cost {:>6} summed {:>5} res {:>8} total  {success} success",
        watch_data.executions.to_formatted_string(&LOCALE),
        watch_data.found_successors.to_formatted_string(&LOCALE),
        BasicWatchResult(watch_data.nexts),
        BasicWatchResult(watch_data.cost),
        BasicWatchResult(successor_sum),
        BasicWatchResult(res_sum),
        total_watch
    );

    todo!("why you exit")
    // match pathfind {
    //     Ok((path, cost)) => MoriResult::Route {
    //         // path: duals_into_single_vec(path),
    //         path: sodas_to_links(path).collect(),
    //         cost,
    //     },
    //     Err((_dump, _all)) => MoriResult::FailingDebug(
    //         // duals_into_single_vec(dump.into_iter().map(|(v, (i, r))| v)),
    //         // duals_into_single_vec(all),
    //         Vec::new(),
    //         Vec::new(),
    //     ),
    // }
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
    head_pos: &VPoint,
    parent_pos: &VPoint,
    processor: &ParentProcessor,
    finding_limiter: &VArea,
    tune: &MoriTunables,
    watch_data: &mut WatchData,
) -> Vec<(VPoint, u32)> {
    info!("successor for {head_pos} parent {parent_pos}");
    watch_data.executions += 1;

    // match can't take expression
    const SECTION_NEGATIVE: i32 = -SECTION_POINTS_I32;
    const SECTION_POSITIVE: i32 = SECTION_POINTS_I32;

    let pos = (head_pos - parent_pos).sugar();
    let head: HopeSodaLink;
    match pos {
        VPointSugar(0, 0) => {
            // note: assumes starting easy
            head = HopeSodaLink::new_soda_straight(*head_pos, FacDirectionQuarter::East);
            trace!("start");
        }
        VPointSugar(SECTION_POSITIVE, 0) => {
            head = HopeSodaLink::new_soda_straight(*head_pos, FacDirectionQuarter::East);
            trace!("forward east {pos}");
        }
        unknown => unimplemented!("what? {unknown}"),
    };

    // let head = path.first().unwrap();

    // if processor.total_links > 200 {
    //     // warn!("too many links");
    //     return Vec::new();
    // }

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
        successors.push((next.pos_next(), cost));
        trace!("next {}", next.my_q());
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
    let area = new_link.area();
    if surface.is_points_free_unchecked(&area) {
        Some(new_link)
    } else {
        None
    }
}
