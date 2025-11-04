use crate::navigator::mine_executor::FailingMeta;
use crate::navigator::mori_cost::calculate_cost_for_link;
use crate::navigator::planners::debug_draw_segment;
use crate::state::tuneables::MoriTunables;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_common::duration::{BasicWatch, BasicWatchResult};
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::{VPointDirectionQ, VSegment};
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::HopeLink;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_soda::{HopeSodaLink, sodas_to_links};
use itertools::Itertools;
use num_format::ToFormattedString;
use pathfinding::prelude::{AStarErr, astar_mori};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::{info, trace, warn};

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

    if !surface.is_points_free_unchecked(&end_link.area_vec()) {
        // // todo: lock?
        // error!("endpoint {}", endpoints.end);
        // let new_surface = crude_dump_on_failure(surface, end_link, endpoints);
        // new_surface.paint_pixel_colored_entire().save_to_oculante();
        // // new_surface
        // //     .paint_pixel_colored_entire()
        // //     .save_to_file(Path::new("work/out0"))
        // //     .unwrap();
        // new_surface.assert_no_empty();
        //
        // panic!("waste of time")

        let mut founds = HashSet::new();
        for pos in end_link.area_vec() {
            founds.insert(surface.get_pixel(pos));
        }
        // let founds_txt = founds.iter().map(|v| v.as_ref()).join(",");
        let founds_txt = "";

        // we need this when 100% blocked
        warn!("waste of time {endpoints} {founds_txt}");
        return MoriResult::FailingDebug {
            err: FailingMeta::default().astar_err,
        };
    }

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
        Ok((links, _)) => format!("{}", links.len()),
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
    // if let Err(err) = &pathfind
    //     && err.parents.len() > 0
    // {
    //     let new_surface = crude_dump_on_failure(surface, end_link, endpoints);
    //
    //     new_surface
    //         //.paint_pixel_graduated(watch_data.was_unfree_check)
    //         .paint_pixel_graduated(count_link_origins(&err.seen))
    //         .save_to_oculante();
    //     std::process::exit(0)
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
                path: sodas_to_links(&path).collect(),
                sodas: path,
                cost,
            }
        }
        Err(err) => MoriResult::FailingDebug { err },
    }
}

fn crude_dump_on_failure(
    surface: &VSurface,
    end_link: HopeSodaLink,
    endpoints: VSegment,
) -> VSurface {
    let mut new_surface = surface.clone();

    let links = sodas_to_links([
        end_link.add_turn90(true),
        HopeSodaLink::new_soda_straight_flipped(&end_link).add_turn90(true),
    ])
    .collect_vec();

    let debug_free = links
        .iter()
        .map(|v| {
            let area = v.area_vec();
            if new_surface.is_points_free_truncating(&area) {
                "free".into()
            } else {
                let mut points = area
                    .into_iter()
                    .map(|v| new_surface.get_pixel(v))
                    .collect_vec();
                points.sort();
                points.dedup();
                format!(
                    "({})",
                    points.into_iter().map(|v| v.as_ref().to_string()).join(",")
                )
            }
        })
        .join(",");
    trace!("debug free {debug_free}");

    new_surface
        .change_pixels(links.into_iter().flat_map(|v| v.area_vec()).collect())
        .stomp(Pixel::Highlighter);
    debug_draw_segment(&mut new_surface, endpoints);

    new_surface
}

#[derive(Default)]
struct WatchData {
    nexts: Duration,
    cost: Duration,
    executions: usize,
    found_successors: usize,
    // was_unfree_check: HashMap<VPoint, u32>,
}

pub enum MoriResult {
    Route {
        path: Vec<HopeLink>,
        sodas: Vec<HopeSodaLink>,
        cost: u32,
    },
    FailingDebug {
        err: AStarErr<HopeSodaLink, u32>,
    },
}

impl MoriResult {
    // pub fn is_route(&self) -> bool {
    //     match &self {
    //         MoriResult::Route { .. } => true,
    //         MoriResult::FailingDebug { .. } => false,
    //     }
    // }
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
        into_buildable_link(
            surface,
            finding_limiter,
            head.add_straight_section(),
            watch_data,
        ),
        into_buildable_link(surface, finding_limiter, head.add_turn90(false), watch_data),
        into_buildable_link(surface, finding_limiter, head.add_turn90(true), watch_data),
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
    watch_data: &mut WatchData,
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
        // for point in area {
        //     watch_data
        //         .was_unfree_check
        //         .entry(point)
        //         .and_modify(|v| *v += 1)
        //         .or_default();
        // }
        None
    }
}

/// Process AStarErr into graduated image
pub fn count_link_origins(links: &[HopeSodaLink]) -> HashMap<VPoint, u32> {
    let mut compressed = HashMap::new();
    for link in links {
        let val = compressed.entry(link.pos_next()).or_default();
        *val += 1;
    }
    compressed
}
