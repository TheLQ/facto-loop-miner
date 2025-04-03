use crate::navigator::base_source::BaseSourceEntry;
use crate::navigator::mine_executor::assert_pos_valid;
use crate::navigator::mine_permutate::{
    get_possible_routes_for_batch, CompletePlan, PlannedRoute, PlannedSequence,
};
use crate::navigator::mine_selector::{select_mines_and_sources, MineSelectBatch};
use crate::navigator::mori::{mori2_start, MoriResult};
use crate::navigator::planners::common::draw_no_touching_zone;
use crate::state::machine::StepParams;
use crate::surface::pixel::Pixel;
use crate::surfacev::mine::MinePath;
use crate::surfacev::vsurface::VSurface;
use itertools::Itertools;
use rayon::prelude::*;
use rayon::ThreadPool;
use simd_json::prelude::ArrayTrait;
use std::cmp::PartialEq;
use tracing::{error, info};

/// Planner v2 "Regis Altare 🎇"
///
/// Perfecting retry algorithm
pub fn start_altare_planner(surface: &mut VSurface, params: &StepParams) {
    let exe_pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|i| format!("exe{i:02}"))
        .num_threads(2)
        .build()
        .unwrap();

    let mut winder = Winder::new(surface);

    draw_no_touching_zone(surface, &winder.mines);

    while !winder.is_complete() {
        let select = winder.next_select();

        match process_select(surface, select, &exe_pool) {
            Ok(best_path) => surface.add_mine_path(best_path).unwrap(),
            Err(routes) => {
                let trigger_mine = routes
                    .iter()
                    .map(|v| &v.location)
                    .reduce(|acc, next| {
                        assert_eq!(acc, next);
                        acc
                    })
                    .unwrap();
                surface.draw_square_area_replacing(
                    &trigger_mine.area,
                    Pixel::MineNoTouch,
                    Pixel::Highlighter,
                );

                error!("failed!");
                break;
            }
        }
    }
}

fn process_select(
    surface: &VSurface,
    select: &MineSelectBatch,
    exe_pool: &ThreadPool,
) -> Result<MinePath, Vec<PlannedRoute>> {
    assert_eq!(select.mines.len(), 1);
    info!("processing {:?}", select.mines[0]);

    let CompletePlan {
        sequences,
        base_sources,
    } = get_possible_routes_for_batch(surface, select.clone());
    // assert!(
    //     sequences.len() <= 2,
    //     "too many destinations {}?",
    //     sequences.len()
    // );
    // assert_ne!(sequences.len(), 0, "no destinations found?");
    assert_eq!(sequences.len(), 2, "not enough destinations found?");

    let actual_base_source = base_sources.borrow().peek_single();
    let mut results: Vec<(MoriResult, PlannedRoute)> = exe_pool.install(|| {
        sequences
            .into_par_iter()
            .map(|PlannedSequence { mut routes }| {
                assert_eq!(routes.len(), 1, "sequence should go to 1 route");
                let route = routes.remove(0);
                let res = execute_route(surface, &route, &actual_base_source);
                (res, route)
            })
            .collect()
    });

    results
        .into_iter()
        // Find best path OR collect all the targeted MineLocation's
        .fold(Err(Vec::new()), |best, (res, route)| match (best, res) {
            (Err(_), MoriResult::Route { path, cost }) => Ok((path, cost, route)),
            (Ok(best), MoriResult::Route { path, cost }) => {
                if best.1 < cost {
                    Ok(best)
                } else {
                    Ok((path, cost, route))
                }
            }
            //
            (Err(mut total), MoriResult::FailingDebug(_, _)) => {
                total.push(route);
                Err(total)
            }
            (Ok(best), MoriResult::FailingDebug(_, _)) => Ok(best),
        })
        .map(|(links, cost, route)| {
            base_sources.borrow_mut().next().unwrap();

            MinePath {
                links,
                cost,
                mine_base: route.location,
            }
        })
        .inspect_err(|routes| assert!(!routes.is_empty()))
}

fn execute_route(
    surface: &VSurface,
    route: &PlannedRoute,
    base_source_entry: &BaseSourceEntry,
) -> MoriResult {
    // let mut working_surface = (*surface).clone();
    let working_surface = surface;

    assert_pos_valid(&base_source_entry, base_source_entry.origin, "origin");

    let adjusted_destination = base_source_entry.apply_intra_offset_to(route.destination);
    assert_pos_valid(&base_source_entry, adjusted_destination, "destination");
    let route_result = mori2_start(
        &working_surface,
        base_source_entry.origin,
        adjusted_destination,
        &route.finding_limiter,
    );
    route_result
}

/// Wind and Re-Wind state
struct Winder {
    cursor: usize,
    mines: Vec<MineSelectBatch>,
    routes: Vec<MinePath>,
}

impl Winder {
    fn new(surface: &VSurface) -> Self {
        Self {
            cursor: 0,
            mines: select_mines_and_sources(&surface, 1)
                .into_success()
                .unwrap(),
            routes: Vec::new(),
        }
    }

    fn is_complete(&self) -> bool {
        self.cursor == self.mines.len()
    }

    fn next_select(&mut self) -> &MineSelectBatch {
        let res = &self.mines[self.cursor];
        self.cursor += 1;
        res
    }
}
