use std::rc::Rc;

use tracing::warn;

use crate::blueprint::bpitem::BlueprintItem;
use crate::blueprint::output::FacItemOutput;
use crate::common::entity::FacEntity;
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_entities::direction::{FacDirectionEighth, FacDirectionQuarter};
use crate::game_entities::rail::{FacEntRailStraight, RAIL_STRAIGHT_DIAMETER};
use crate::game_entities::rail_curved::FacEntRailCurved;

/// Rail Pathing v10.999?, "Irys💎 Hope"
///
/// Describe Rail as a self-contained sequence of links,
/// without significant Pathfinding-specific code overhead
pub struct RailHopeSingle {
    links: Vec<RailHopeLink>,
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
    output: Rc<FacItemOutput>,
}

pub struct RailHopeLink {
    start_pos: VPoint,
    rtype: RailHopeLinkType,
    link_direction: FacDirectionQuarter,
}

#[derive(Debug, Clone)]
pub enum FacEntRailType {
    Straight,
    Curved,
}

pub struct RailHopeLinkRail {
    direction: FacDirectionEighth,
    rtype: FacEntRailType,
    position: VPoint,
}

pub enum RailHopeLinkType {
    Straight { length: usize },
    Turn90 { clockwise: bool },
    Shift45 { clockwise: bool, length: usize },
}

impl RailHopeSingle {
    pub fn new(
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
        output: Rc<FacItemOutput>,
    ) -> Self {
        origin.assert_even_position();
        Self {
            links: Vec::new(),
            origin,
            origin_direction,
            output,
        }
    }

    // pub fn compress_straight(&mut self) {}

    fn add_straight_line_raw(
        &mut self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        length: usize,
    ) {
        warn!("writing direction {}", direction);
        for i in 0..length {
            self.write_link_rail(RailHopeLinkRail {
                position: origin.move_direction(&direction, i * RAIL_STRAIGHT_DIAMETER),
                direction: direction.to_direction_eighth(),
                rtype: FacEntRailType::Straight,
            })
        }
        self.links.push(RailHopeLink {
            start_pos: origin,
            link_direction: direction,
            rtype: RailHopeLinkType::Straight { length },
        })
    }

    // fn last_link(&self) -> &RailHopeLink {
    //     // we always should have a link
    //     self.links.last().unwrap()
    // }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        self.links
            .last()
            .map(|v| &v.link_direction)
            .unwrap_or(&self.origin_direction)
    }

    pub(crate) fn current_next_pos(&self) -> VPoint {
        self.links
            .last()
            .map(|v| v.next_straight_position())
            .unwrap_or(self.origin)
    }

    fn write_link_rail(&mut self, link: RailHopeLinkRail) {
        link.to_fac(&self.output);
    }
}

impl<'o> RailHopeAppender for RailHopeSingle {
    fn add_straight(&mut self, length: usize) {
        self.add_straight_line_raw(
            self.current_next_pos(),
            self.current_direction().clone(),
            length,
        );
    }

    fn add_turn90(&mut self, clockwise: bool) {
        let _ = self
            .output
            .context_handle(format!("Turn90-{}", if clockwise { "clw" } else { "ccw" }));
        // warn!("turn 90 start---- clockwise {}", clockwise);
        /*
        Factorio 1 Rails are really complicated
        This is version 3544579 💎

        Order: Curve > Straight 45 > Curve

        In X, steps are 3 > 3 > 3
        In Y, steps are 1 > 3 > 3
        (signs and axis depend on direction)

        Compass rotations have no apparent pattern but stable in all turn directions
        "Clockwise" is 1 > -2 > -1
        "Counter-Clockwise" is 0 > 1 > 2

        Directions appear arbitraty
        eg. curved rail from North to NorthWest in Factorio is... curved-rail North?
        */
        let cur_direction = self.current_direction().clone();
        warn!("cur direction {}", cur_direction);
        // 1,1 to cancel RailStraight's to_fac offset
        let origin_fac = self.current_next_pos() + VPOINT_ONE;

        // curve 1
        let first_curve_pos = origin_fac
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 1));
        let first_curve_direction = cur_direction.to_direction_eighth();
        let first_curve_direction = if clockwise {
            first_curve_direction.rotate_once()
        } else {
            first_curve_direction
        };
        self.write_link_rail(RailHopeLinkRail {
            position: first_curve_pos,
            direction: first_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });
        warn!("first curve {:?}", first_curve_direction);

        // middle
        let middle_straight_pos = first_curve_pos
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 3));
        let middle_straight_direction = if clockwise {
            first_curve_direction.rotate_opposite().rotate_opposite()
        } else {
            first_curve_direction.rotate_once()
        };
        warn!("middle straight {:?}", middle_straight_direction);
        self.write_link_rail(RailHopeLinkRail {
            // -1,-1 to cancel RailStraight's to_fac offset
            position: middle_straight_pos - VPOINT_ONE,
            direction: middle_straight_direction.clone(),
            rtype: FacEntRailType::Straight,
        });

        // curve 2
        let last_curve_pos = middle_straight_pos
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 3));
        let last_curve_direction = if clockwise {
            middle_straight_direction.rotate_opposite()
        } else {
            middle_straight_direction.rotate_once().rotate_once()
        };
        warn!("last curve {:?}", middle_straight_direction);
        self.write_link_rail(RailHopeLinkRail {
            position: last_curve_pos,
            direction: last_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        // where to go next
        let link_direction = if clockwise {
            cur_direction.rotate_once()
        } else {
            cur_direction.rotate_opposite()
        };
        warn!(
            "from start direction {} to end direction {}",
            cur_direction, link_direction
        );
        self.links.push(RailHopeLink {
            start_pos: self.current_next_pos(),
            link_direction,
            rtype: RailHopeLinkType::Turn90 { clockwise },
        })
    }

    fn add_shift45(&mut self, clockwise: bool, length: usize) {
        /*
        Factorio 1 Rails at 45 degrees are still really complicated

        Order: Curve > 2x 45 straights > Curve back

        Middle rail is in pairs of 2 on the same X axis.
        In game, in preview-item-place-view they're stacked on top of eachother.
        Curves start only on these pairs,
        if only 1 the game rail planner inserts other 45

        Between 2x pairs, the middle 2 rails are on the same Y axis
        */

        let cur_direction = self.current_direction().clone();
        // 1,1 to cancel RailStraight's to_fac offset
        let origin_fac = self.current_next_pos() + VPOINT_ONE;

        // curve 1 (copy of above turn90)
        let first_curve_pos = origin_fac
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 1));
        let first_curve_direction = cur_direction.to_direction_eighth();
        let first_curve_direction = if clockwise {
            first_curve_direction.rotate_once()
        } else {
            first_curve_direction
        };
        self.write_link_rail(RailHopeLinkRail {
            position: first_curve_pos,
            direction: first_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        // middle
        let middle_straight_pos = first_curve_pos
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 3));
        let middle_a_direction = if clockwise {
            first_curve_direction.rotate_opposite().rotate_opposite()
        } else {
            first_curve_direction.rotate_once()
        };
        let middle_b_direction = middle_a_direction.rotate_flip();

        let mut next_a_pos = middle_straight_pos;
        let mut last_b_pos = middle_straight_pos
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, -2));
        for _ in 0..length {
            self.write_link_rail(RailHopeLinkRail {
                // -1,-1 to cancel RailStraight's to_fac offset
                position: next_a_pos - VPOINT_ONE,
                direction: middle_a_direction.clone(),
                rtype: FacEntRailType::Straight,
            });
            last_b_pos = next_a_pos.move_direction(&cur_direction, 2);
            self.write_link_rail(RailHopeLinkRail {
                // -1,-1 to cancel RailStraight's to_fac offset
                position: last_b_pos - VPOINT_ONE,
                direction: middle_b_direction.clone(),
                rtype: FacEntRailType::Straight,
            });
            next_a_pos =
                last_b_pos.move_direction_sideways(&cur_direction, neg_if_false(clockwise, 2))
        }

        // curve 2 back
        let last_curve_pos = last_b_pos
            .move_direction(&cur_direction, 3)
            .move_direction_sideways(&cur_direction, neg_if_false(clockwise, 3));
        let last_curve_direction = if clockwise {
            first_curve_direction
                .rotate_once()
                .rotate_once()
                .rotate_once()
                .rotate_once()
        } else {
            first_curve_direction
                .rotate_opposite()
                .rotate_opposite()
                .rotate_opposite()
                .rotate_opposite()
        };
        self.write_link_rail(RailHopeLinkRail {
            position: last_curve_pos,
            direction: last_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        self.links.push(RailHopeLink {
            start_pos: self.current_next_pos(),
            link_direction: cur_direction.clone(),
            rtype: RailHopeLinkType::Shift45 { clockwise, length },
        })
    }
}

impl RailHopeLink {
    fn next_straight_position(&self) -> VPoint {
        match &self.rtype {
            RailHopeLinkType::Straight { length } => self
                .start_pos
                .move_direction(&self.link_direction, length * RAIL_STRAIGHT_DIAMETER),
            RailHopeLinkType::Turn90 { clockwise } => {
                let unrotated = if *clockwise {
                    self.link_direction.rotate_opposite()
                } else {
                    self.link_direction.rotate_once()
                };
                warn!("unrotated {}", unrotated);
                self.start_pos
                    .move_direction(&unrotated, 10)
                    .move_direction_sideways(&unrotated, neg_if_false(*clockwise, 12))
            }
            RailHopeLinkType::Shift45 { clockwise, length } => self
                .start_pos
                .move_direction(&self.link_direction, 14 + (*length * 2))
                .move_direction_sideways(
                    &self.link_direction,
                    neg_if_false(*clockwise, 6 + (*length as i32 * 2)),
                ),
        }
    }
}

impl RailHopeLinkRail {
    fn to_fac(&self, res: &FacItemOutput) {
        match self.rtype {
            FacEntRailType::Straight => res.write(BlueprintItem::new(
                FacEntRailStraight::new(self.direction.clone()).into_boxed(),
                self.position,
            )),
            FacEntRailType::Curved => res.write(BlueprintItem::new(
                FacEntRailCurved::new(self.direction.clone()).into_boxed(),
                self.position,
            )),
        }
    }
}

fn neg_if_false(flag: bool, value: i32) -> i32 {
    if flag { value } else { -value }
}

#[cfg(test)]
mod test {
    use crate::{
        blueprint::output::FacItemOutput, common::vpoint::VPoint,
        game_blocks::rail_hope::RailHopeAppender, game_entities::direction::FacDirectionQuarter,
    };

    use super::RailHopeSingle;

    #[test]
    fn test_straight_chain() {
        let hope_long_output = FacItemOutput::new_blueprint().into_rc();
        let hope_long_next_pos = {
            let mut hope_long = RailHopeSingle::new(
                VPoint::zero(),
                FacDirectionQuarter::North,
                hope_long_output.clone(),
            );
            hope_long.add_straight(2);
            hope_long.add_straight(3);
            hope_long.add_straight(6);

            hope_long.current_next_pos()
        };

        let hope_long_bp = hope_long_output.consume_rc().into_blueprint_contents();

        let hope_short_output = FacItemOutput::new_blueprint().into_rc();
        let hope_short_next_pos = {
            let mut hope_short = RailHopeSingle::new(
                VPoint::zero(),
                FacDirectionQuarter::North,
                hope_short_output.clone(),
            );
            hope_short.add_straight(11);

            hope_short.current_next_pos()
        };

        let hope_short_bp = hope_short_output.consume_rc().into_blueprint_contents();

        assert_eq!(hope_long_bp.fac_entities(), hope_short_bp.fac_entities(),);
        assert_eq!(hope_long_next_pos, hope_short_next_pos);
    }
}
