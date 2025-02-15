use std::rc::Rc;

use crate::common::vpoint::VPOINT_TEN;
use crate::{
    admiral::err::AdmiralResult,
    blueprint::output::FacItemOutput,
    common::vpoint::{VPOINT_ZERO, VPoint},
    game_blocks::{
        rail_hope::RailHopeAppender, rail_hope_dual::RailHopeDual, rail_hope_single::RailHopeSingle,
    },
    game_entities::direction::FacDirectionQuarter,
};

pub fn make_rail_spiral_90(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let origin: VPoint = VPOINT_ZERO;
    for clockwise in [
        true,  //
        false, //
    ] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North, output.clone());
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South, output.clone());
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East, output.clone());
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West, output.clone());
        for mut hope in [
            hope1, //
            hope2, //
            hope3, //
            hope4, //
        ] {
            hope.add_straight(2);
            hope.add_turn90(clockwise);
            hope.add_straight(2);
            hope.add_turn90(clockwise);
            hope.add_straight(2);
        }
    }

    Ok(())
}

pub fn make_rail_shift_45(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    let origin = VPOINT_ZERO;
    for clockwise in [false /*, false*/] {
        let hope1 = RailHopeSingle::new(origin, FacDirectionQuarter::North, output.clone());
        let hope2 = RailHopeSingle::new(origin, FacDirectionQuarter::South, output.clone());
        let hope3 = RailHopeSingle::new(origin, FacDirectionQuarter::East, output.clone());
        let hope4 = RailHopeSingle::new(origin, FacDirectionQuarter::West, output.clone());
        for mut hope in [hope1, hope2, hope3, hope4] {
            hope.add_straight(2);
            hope.add_shift45(clockwise, 3);
            hope.add_straight(2)
        }
    }

    Ok(())
}

pub fn make_rail_dual_turning(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    for clockwise in [true, false] {
        for direction in [
            FacDirectionQuarter::North,
            FacDirectionQuarter::East,
            FacDirectionQuarter::South,
            FacDirectionQuarter::West,
        ] {
            let mut hope = RailHopeDual::new(VPOINT_ZERO, direction, output.clone());
            hope.add_straight(5);
            hope.add_turn90(clockwise);
            hope.add_straight(5);
            hope.add_straight(5);
        }
    }

    Ok(())
}

pub fn make_rail_dual_powered(output: Rc<FacItemOutput>) -> AdmiralResult<()> {
    for direction in [
        FacDirectionQuarter::North,
        // FacDirectionQuarter::East,
        // FacDirectionQuarter::South,
        // FacDirectionQuarter::West,
    ] {
        let origin = VPOINT_ZERO.move_direction_usz(&direction, 6);

        let mut hope = RailHopeDual::new(origin, direction, output.clone());
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();
        hope.add_turn90(true);
        hope.add_straight_section();
        // for entity in hope.to_fac() {
        //     let bpfac = entity.to_blueprint();
        //     // let bppos = &bpfac.position;
        //     // if existing_points.contains(bppos) {
        //     //     continue;
        //     // } else {
        //     //     existing_points.push(bppos.clone());
        //     // }
        //     admiral.execute_checked_command(bpfac.to_lua().into_boxed())?;
        // }
    }

    Ok(())
}

/// do the electric poles line up?
pub fn make_rail_gee_for_power(output: Rc<FacItemOutput>) {
    let mut rail = RailHopeDual::new(VPOINT_TEN, FacDirectionQuarter::East, output);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
}
