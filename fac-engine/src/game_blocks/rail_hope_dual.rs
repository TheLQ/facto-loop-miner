use std::rc::Rc;

use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_blocks::rail_hope_single::RailHopeSingle;
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::electric_large::{FacEntElectricLarge, FacEntElectricLargeType};
use crate::game_entities::lamp::FacEntLamp;
use crate::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER;

/// A 4 way intersection is 13 rails wide square.  
pub const DUAL_RAIL_STEP: usize = STRAIGHT_RAIL_STEP * 2;
const STRAIGHT_RAIL_STEP: usize = 13;

/// The dreamed Side-by-side rail generator
pub struct RailHopeDual {
    hopes: [RailHopeSingle; 2],
    output: Rc<FacItemOutput>,
}

impl RailHopeDual {
    pub fn new(
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
        output: Rc<FacItemOutput>,
    ) -> Self {
        // move on axis, not rotation, to give every direction the same starting point
        // and maintain intersection
        let next_origin = origin.move_direction_sideways_axis_int(
            origin_direction,
            -((RAIL_STRAIGHT_DIAMETER * 2) as i32),
        );
        let mut hopes = [
            RailHopeSingle::new(origin, origin_direction, output.clone()),
            RailHopeSingle::new(next_origin, origin_direction, output.clone()),
        ];

        match origin_direction {
            FacDirectionQuarter::East | FacDirectionQuarter::North => {}
            FacDirectionQuarter::West | FacDirectionQuarter::South => {
                // maintain order expected by turn90
                hopes.swap(0, 1);
                // maintain
            }
        }
        Self {
            output: output.clone(),
            hopes,
        }
    }

    pub fn add_straight_section(&mut self) {
        self.add_straight(STRAIGHT_RAIL_STEP);
        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "👐Ruby".into());
            self.add_electric_next();
        }
    }

    pub fn add_electric_next(&mut self) {
        let last_link = self.hopes[0].appender_link();
        self.add_electric_next_for_link(
            last_link.next_direction,
            last_link.next_straight_position(),
        );
    }

    pub fn add_electric_next_for_link(&mut self, direction: FacDirectionQuarter, pos: VPoint) {
        // must use next pos, because last start link might be part of a turn90
        let electric_large_pos = pos.move_direction_sideways_int(direction, -2);
        self.output.writei(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big),
            electric_large_pos,
        );

        self.output.writei(
            FacEntLamp::new(),
            (electric_large_pos + VPOINT_ONE).move_factorio_style_direction(direction, 1.5),
        );
    }

    pub(crate) fn next_buildable_point(&self) -> VPoint {
        self.hopes[0].next_pos()
    }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        &self.hopes[0].last_link().next_direction
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        for (i, rail) in &mut self.hopes.iter_mut().enumerate() {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("👐Dual-{}", i));
            rail.add_straight(length);
        }
    }

    fn add_turn90(&mut self, clockwise: bool) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "👐Dual-Turn".into());
        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }

        for rail in &mut self.hopes {
            rail.add_turn90(clockwise);
        }

        if clockwise {
            self.hopes[1].add_straight(2);
        } else {
            self.hopes[0].add_straight(2);
        }
        self.add_electric_next();
    }

    fn add_shift45(&mut self, _clockwise: bool, _length: usize) {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::blueprint::bpitem::BlueprintItem;
    use crate::blueprint::output::FacItemOutput;
    use crate::common::vpoint::{VPOINT_ZERO, VPoint};
    use crate::common::vpoint_direction::VPointDirectionQ;
    use crate::game_blocks::rail_hope::RailHopeAppender;
    use crate::game_blocks::rail_hope_dual::{DUAL_RAIL_STEP, RailHopeDual};
    use crate::game_entities::direction::FacDirectionQuarter;
    use crate::game_entities::rail_straight::{RAIL_STRAIGHT_DIAMETER, RAIL_STRAIGHT_DIAMETER_I32};

    #[test]
    fn congruent_line() {
        // let output = FacItemOutput::new_null().into_rc();

        let mut a = dual_gen((VPOINT_ZERO, FacDirectionQuarter::East), |rail| {
            rail.add_straight(4);
        });
        a.sort();

        let mut b = dual_gen(
            (
                VPOINT_ZERO.move_x(3 * RAIL_STRAIGHT_DIAMETER_I32),
                FacDirectionQuarter::West,
            ),
            |rail| {
                rail.add_straight(4);
            },
        );
        b.sort();

        compare_points(&a, &b);
    }

    #[test]
    fn congruent_turn_step() {
        // let output = FacItemOutput::new_null().into_rc();

        let mut a = dual_gen((VPOINT_ZERO, FacDirectionQuarter::East), |rail| {
            rail.add_straight(DUAL_RAIL_STEP);
            rail.add_turn90(true);
            rail.add_straight(DUAL_RAIL_STEP);
        });
        a.sort();

        let mut b = dual_gen(
            (
                VPOINT_ZERO.move_y_usize(DUAL_RAIL_STEP * 2),
                FacDirectionQuarter::East,
            ),
            |rail| {
                rail.add_straight(DUAL_RAIL_STEP);
                rail.add_turn90(false);
                rail.add_straight(DUAL_RAIL_STEP);
            },
        );
        b.sort();

        compare_points(&a, &b);
    }

    fn dual_gen(
        origin: impl Into<VPointDirectionQ>,
        work: impl Fn(&mut RailHopeDual),
    ) -> Vec<VPoint> {
        let origin = origin.into();
        let output = FacItemOutput::new_blueprint().into_rc();
        let mut rail = RailHopeDual::new(origin.0, origin.1, output.clone());
        work(&mut rail);
        drop(rail);

        output.flush();
        let items: Vec<BlueprintItem> = output.consume_rc().into_blueprint_contents().consume().0;
        items.into_iter().map(|v| *v.position()).collect()
    }

    fn compare_points(a: &[VPoint], b: &[VPoint]) {
        let mut success = true;
        for i in 0..a.len() {
            let e_a = a.get(i).unwrap();
            let e_b = b.get(i).unwrap();
            if e_a == e_b {
                println!("{e_a} > {e_b}")
            } else {
                success = false;
                println!("{e_a} > {e_b} !!!")
            }
        }
        assert!(success);

        assert!(!a.is_empty());
        assert!(!b.is_empty());
        assert_eq!(a.len(), b.len());
    }
}
