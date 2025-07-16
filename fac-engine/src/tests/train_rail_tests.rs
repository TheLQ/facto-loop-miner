use crate::common::vpoint::VPOINT_TEN;
use crate::common::vpoint_direction::VPointDirectionQ;
use crate::game_blocks::rail_hope::RailHopeLink;
use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
use crate::game_blocks::rail_hope_soda::{HopeSodaLink, sodas_to_rails};
use crate::game_entities::electric_mini::FacEntElectricMiniType;
use crate::game_entities::lamp::FacEntLamp;
use crate::{
    admiral::err::AdmiralResult,
    blueprint::output::FacItemOutput,
    common::vpoint::{VPOINT_ZERO, VPoint},
    game_blocks::{
        rail_hope::RailHopeAppender, rail_hope_dual::RailHopeDual, rail_hope_single::RailHopeSingle,
    },
    game_entities::direction::FacDirectionQuarter,
};
use itertools::Itertools;
use std::rc::Rc;
use strum::VariantArray;
use tracing::info;

pub fn make_rail_basic_dual(output: Rc<FacItemOutput>) {
    let my_output = output.clone();

    my_output.writei(FacEntLamp::new(), VPOINT_ZERO);

    let mut rail = RailHopeDual::new(VPOINT_ZERO, FacDirectionQuarter::East, output.clone());
    rail.add_straight_section();

    let mut rail = RailHopeDual::new(VPOINT_ZERO, FacDirectionQuarter::East, output.clone());
    rail.add_turn90(true);
    my_output.writei(FacEntLamp::new(), rail.pos_next());
    rail.add_turn90(false);
    my_output.writei(FacEntLamp::new(), rail.pos_next());

    // let mut rail = RailHopeDual::new(VPOINT_ZERO, FacDirectionQuarter::East, output.clone());
    // rail.add_turn90(false);
    // my_output.writei(FacEntLamp::new(), rail.pos_next());
    // rail.add_turn90(true);
    // my_output.writei(FacEntLamp::new(), rail.pos_next());
}

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
        let origin = VPOINT_ZERO.move_direction_usz(direction, 6);

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

pub fn make_rail_step_sketch_single(output: Rc<FacItemOutput>) {
    let mut step_starts = vec![VPOINT_ZERO];
    step_starts.push(*step_starts.last().unwrap() - VPoint::new(0, SECTION_POINTS_I32));
    step_starts.push(*step_starts.last().unwrap() - VPoint::new(0, SECTION_POINTS_I32));

    // let mut rail = RailHopeDual::new(
    //     step_starts.remove(0),
    //     FacDirectionQuarter::East,
    //     output.clone(),
    // );
    // rail.add_straight(STEP);
    // rail.add_turn90(true);
    // rail.add_straight(STEP);
    // rail.add_turn90(true);
    // // rail.add_straight(STEP);

    // let mut rail = RailHopeDual::new(
    //     step_starts.remove(0),
    //     FacDirectionQuarter::East,
    //     output.clone(),
    // );
    // rail.add_straight(STEP * 2);
    // rail.add_turn90(true);
    // rail.add_straight(STEP * 2);
    // rail.add_turn90(true);
    // rail.add_straight(STEP * 2);
    // rail.add_turn90(true);
    // rail.add_turn90(true);
    // rail.add_straight(STEP);

    let offset_start = step_starts.remove(0);
    // make_rail_step_letter_c(offset_start, output);
    make_rail_step_letter_c_with_s(offset_start, output);
}

fn make_rail_step_letter_o(offset_start: VPoint, output: Rc<FacItemOutput>) {
    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::West, output.clone());
    rail.add_straight_section();
    rail.add_turn90(false);
    rail.add_straight_section();
    rail.add_turn90(false);
    rail.add_straight_section();

    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::East, output.clone());
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
}

fn make_rail_step_letter_c(offset_start: VPoint, output: Rc<FacItemOutput>) {
    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::East, output.clone());
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    output.writei(FacEntLamp::new(), offset_start);

    let offset_start = offset_start.move_y(SECTION_POINTS_I32);
    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::East, output.clone());
    rail.add_straight_section();
    rail.add_turn90(false);
    rail.add_straight_section();
    rail.add_turn90(false);
    rail.add_straight_section();
    output.writei(FacEntElectricMiniType::Small.entity(), offset_start);
}

fn make_rail_step_letter_c_with_s(offset_start: VPoint, output: Rc<FacItemOutput>) {
    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::East, output.clone());
    rail.add_straight_section();
    let mut next = rail.pos_next();
    info!("next {}", next);
    let mut last = next;

    rail.add_turn90(true);
    next = rail.pos_next();
    info!("next {} diff {}", next, last - next);
    last = next;

    rail.add_straight_section();
    next = rail.pos_next();
    info!("next {} diff {}", next, last - next);
    last = next;

    rail.add_turn90(true);
    next = rail.pos_next();
    info!("next {} diff {}", next, last - next);
    last = next;

    rail.add_straight_section();
    next = rail.pos_next();
    info!("next {} diff {}", next, last - next);
    last = next;

    output.writei(FacEntLamp::new(), offset_start);

    let offset_start = offset_start.move_y(SECTION_POINTS_I32);
    let mut rail = RailHopeDual::new(offset_start, FacDirectionQuarter::East, output.clone());
    rail.add_turn90(false);
    rail.add_turn90(true);
    rail.add_turn90(true);
    rail.add_turn90(false);
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_turn90(true);
    rail.add_straight_section();
    rail.add_straight_section();
    rail.add_turn90(true);
    output.writei(FacEntElectricMiniType::Small.entity(), offset_start);
}

pub fn make_soda(output: Rc<FacItemOutput>) {
    let source = HopeSodaLink::new_soda_straight(VPOINT_TEN, FacDirectionQuarter::East);
    let then = source.add_turn90(true);
    let after = then.add_straight_section();
    let sodas = [source, then, after];
    // let sodas = [then];

    for soda in &sodas {
        output.writei(FacEntLamp::new(), soda.pos_start());
    }

    for rail in sodas_to_rails(sodas) {
        rail.write_output(&output);
    }
}

pub fn make_soda_plus(output: Rc<FacItemOutput>) {
    for direction in FacDirectionQuarter::VARIANTS {
        // for direction in &[FacDirectionQuarter::West] {
        for clockwise in [true, false] {
            info!("direction {direction} clockwise {clockwise}");
            let source = HopeSodaLink::new_soda_turn(VPOINT_TEN, *direction, clockwise);
            for rail in sodas_to_rails([source]) {
                rail.write_output(&output);
            }
        }
    }
}

pub fn make_base_source_rails(output: Rc<FacItemOutput>) {
    // let primary_link = HopeSodaLink::new_soda_straight(VPOINT_ZERO, FacDirectionQuarter::East);
    // primary_link.pos_start().assert_step_rail();
    //
    // let end_link = HopeSodaLink::new_soda_straight(
    //     VPoint::new(0, SECTION_POINTS_I32),
    //     FacDirectionQuarter::East,
    // );
    // end_link.pos_start().assert_step_rail();
    //
    // for rail in sodas_to_rails([primary_link, end_link]) {
    //     rail.write_output(&output);
    // }
    let mut source =
        BaseSourceEighth::new(VPointDirectionQ(VPOINT_ZERO, FacDirectionQuarter::East), 1);

    for _ in 0..6 {
        let new_source = source.next().unwrap();
        let link = HopeSodaLink::new_soda_straight(*new_source.point(), FacDirectionQuarter::East);

        for rail in sodas_to_rails([link]) {
            rail.write_output(&output);
        }
    }
}

/// Generator
#[derive(Debug)]
struct BaseSourceEighth {
    origin: VPointDirectionQ,
    sign: i32,
    next: i32,
}

impl BaseSourceEighth {
    fn new(origin: VPointDirectionQ, sign: i32) -> Self {
        // Must start at 1 due to conflict at 0!
        Self {
            origin,
            sign,
            next: 1,
        }
    }

    fn get_for_index(&self, index: i32) -> VPointDirectionQ {
        const INTRA_COUNT: i32 = 4;
        const LOOP_STEP: i32 = SECTION_POINTS_I32;

        let pos = self.origin.point().move_direction_sideways_int(
            self.origin.direction(),
            self.sign * LOOP_STEP * (index / INTRA_COUNT) + ((index % INTRA_COUNT) * 6),
        );
        tracing::trace!("working with {} from {}", pos, self.origin);
        // pos.assert_step_rail();
        VPointDirectionQ(pos, *self.origin.direction())
    }

    fn peek_single(&self) -> VPointDirectionQ {
        self.get_for_index(self.next)
    }

    fn peek_multiple(&self, size: usize) -> Vec<VPointDirectionQ> {
        let res = (self.next..(self.next + size as i32))
            .map(|i| self.get_for_index(i))
            .collect_vec();
        assert_eq!(res.len(), size);
        res
    }
}

impl Iterator for BaseSourceEighth {
    type Item = VPointDirectionQ;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_for_index(self.next);
        self.next += 1;
        Some(result)
    }
}

struct BaseSourceEntry {
    origin: VPointDirectionQ,
    applied_intra_offset: VPoint,
}
