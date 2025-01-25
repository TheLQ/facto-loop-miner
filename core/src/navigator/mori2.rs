use crate::navigator::rail_point_compare::RailPointCompare;
use crate::surfacev::vsurface::VSurface;
use crate::util::duration::BasicWatch;
use facto_loop_miner_fac_engine::blueprint::output::FacItemOutput;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::common::vpoint_direction::VPointDirectionQ;
use facto_loop_miner_fac_engine::game_blocks::rail_hope::RailHopeAppender;
use facto_loop_miner_fac_engine::game_blocks::rail_hope_single::{
    HopeFactoRail, HopeLink, RailHopeSingle,
};
use pathfinding::prelude::astar_mori;

const STRAIGHT_STEP_SIZE: usize = 1;

pub fn mori2_start(surface: &VSurface, start: VPointDirectionQ, end: VPointDirectionQ) {
    let pathfind_watch = BasicWatch::start();

    validate_positions(&start, &end);
    let start_link = new_straight_link_from_vd(&start);
    let end_link = new_straight_link_from_vd(&start);

    let pathfind = astar_mori(
        &start_link,
        |(successor_rail, parents, _total_cost)| {
            let (next, parents) = parents.split_last().unwrap();
            assert_eq!(successor_rail, next);
            successors(parents, next)
        },
        |_p| 1,
        |p| p == &end_link,
    );
}

fn validate_positions(start: &VPointDirectionQ, end: &VPointDirectionQ) {
    start.point().assert_odd_16x16_position();
    end.point().assert_odd_16x16_position();
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

fn successors(parents: &[HopeLink], next: &HopeLink) -> Vec<(RailPointCompare, u32)> {}
