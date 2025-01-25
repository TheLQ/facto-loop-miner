use std::mem;
use std::rc::Rc;

use tracing::trace;

use crate::blueprint::bpitem::BlueprintItem;
use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::common::entity::FacEntity;
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_entities::direction::{FacDirectionEighth, FacDirectionQuarter};
use crate::game_entities::rail_curved::FacEntRailCurved;
use crate::game_entities::rail_straight::{FacEntRailStraight, RAIL_STRAIGHT_DIAMETER};

/// Rail Pathing v10.999?, "Irys💎 Hope"
///
/// Describe Rail as a self-contained sequence of links,
/// powered by the vastly better fac-engine API,
/// without significant Pathfinding-specific code overhead.
pub struct RailHopeSingle {
    links: Vec<HopeLink>,
    rail_cache: Vec<HopeFactoRail>,
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
    output: Rc<FacItemOutput>,
}

#[derive(Debug, Clone)]
pub struct HopeLink {
    pub start: VPoint,
    pub rtype: RailHopeLinkType,
    pub next_direction: FacDirectionQuarter,
    pub rails: Vec<HopeFactoRail>,
}

#[derive(Debug, Clone)]
pub enum FacEntRailType {
    Straight,
    Curved,
}

/// Everything needed to create a BlueprintItem rail
#[derive(Debug, Clone)]
pub struct HopeFactoRail {
    pub direction: FacDirectionEighth,
    pub rtype: FacEntRailType,
    pub position: VPoint,
}

#[derive(Debug, Clone)]
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
            rail_cache: Vec::new(),
            origin,
            origin_direction,
            output,
        }
    }

    pub fn links(&self) -> &[HopeLink] {
        &self.links
    }

    // pub fn compress_straight(&mut self) {}

    fn add_straight_line_raw(
        &mut self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        length: usize,
    ) {
        trace!("writing direction {}", direction);
        for i in 0..length {
            self.write_link_rail(HopeFactoRail {
                position: origin.move_direction_usz(direction, i * RAIL_STRAIGHT_DIAMETER),
                direction: direction.to_direction_eighth(),
                rtype: FacEntRailType::Straight,
            })
        }
        let rails = self.drain_rails_cache();
        self.links.push(HopeLink {
            start: origin,
            next_direction: direction,
            rtype: RailHopeLinkType::Straight { length },
            rails,
        })
    }

    // fn last_link(&self) -> &RailHopeLink {
    //     // we always should have a link
    //     self.links.last().unwrap()
    // }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        self.links
            .last()
            .map(|v| &v.next_direction)
            .unwrap_or(&self.origin_direction)
    }

    pub(crate) fn current_next_pos(&self) -> VPoint {
        self.links
            .last()
            .map(|v| v.next_straight_position())
            .unwrap_or(self.origin)
    }

    fn write_link_rail(&mut self, link: HopeFactoRail) {
        link.to_fac(&self.output);
        self.rail_cache.push(link)
    }

    fn drain_rails_cache(&mut self) -> Vec<HopeFactoRail> {
        let mut new = Vec::new();
        mem::swap(&mut self.rail_cache, &mut new);
        assert!(!new.is_empty(), "len {}", new.len());
        new
    }
}

impl<'o> RailHopeAppender for RailHopeSingle {
    fn add_straight(&mut self, length: usize) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, format!("👉Straight-{}", length));
        self.add_straight_line_raw(self.current_next_pos(), *self.current_direction(), length);
    }

    fn add_turn90(&mut self, clockwise: bool) {
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
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("👉Turn90-{}", if clockwise { "clw" } else { "ccw" }),
        );

        let cur_direction = self.current_direction().clone();
        trace!("cur direction {}", cur_direction);
        // 1,1 to cancel RailStraight's to_fac offset
        let origin_fac = self.current_next_pos() + VPOINT_ONE;

        // curve 1
        let first_curve_pos = origin_fac
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 1));
        let first_curve_direction = cur_direction.to_direction_eighth();
        let first_curve_direction = if clockwise {
            first_curve_direction.rotate_once()
        } else {
            first_curve_direction
        };
        self.write_link_rail(HopeFactoRail {
            position: first_curve_pos,
            direction: first_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });
        trace!("first curve {:?}", first_curve_direction);

        // middle
        let middle_straight_pos = first_curve_pos
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 3));
        let middle_straight_direction = if clockwise {
            first_curve_direction.rotate_opposite().rotate_opposite()
        } else {
            first_curve_direction.rotate_once()
        };
        trace!("middle straight {:?}", middle_straight_direction);
        self.write_link_rail(HopeFactoRail {
            // -1,-1 to cancel RailStraight's to_fac offset
            position: middle_straight_pos - VPOINT_ONE,
            direction: middle_straight_direction.clone(),
            rtype: FacEntRailType::Straight,
        });

        // curve 2
        let last_curve_pos = middle_straight_pos
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 3));
        let last_curve_direction = if clockwise {
            middle_straight_direction.rotate_opposite()
        } else {
            middle_straight_direction.rotate_once().rotate_once()
        };
        trace!("last curve {:?}", middle_straight_direction);
        self.write_link_rail(HopeFactoRail {
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
        trace!(
            "from start direction {} to end direction {}",
            cur_direction, link_direction
        );
        let rails = self.drain_rails_cache();
        self.links.push(HopeLink {
            start: self.current_next_pos(),
            next_direction: link_direction,
            rtype: RailHopeLinkType::Turn90 { clockwise },
            rails,
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
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("👉Shift45-{}", if clockwise { "clw" } else { "ccw" }),
        );

        let cur_direction = self.current_direction().clone();
        // 1,1 to cancel RailStraight's to_fac offset
        let origin_fac = self.current_next_pos() + VPOINT_ONE;

        // curve 1 (copy of above turn90)
        let first_curve_pos = origin_fac
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 1));
        let first_curve_direction = cur_direction.to_direction_eighth();
        let first_curve_direction = if clockwise {
            first_curve_direction.rotate_once()
        } else {
            first_curve_direction
        };
        self.write_link_rail(HopeFactoRail {
            position: first_curve_pos,
            direction: first_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        // middle
        let middle_straight_pos = first_curve_pos
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 3));
        let middle_a_direction = if clockwise {
            first_curve_direction.rotate_opposite().rotate_opposite()
        } else {
            first_curve_direction.rotate_once()
        };
        let middle_b_direction = middle_a_direction.rotate_flip();

        let mut next_a_pos = middle_straight_pos;
        let mut last_b_pos = middle_straight_pos
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, -2));
        for _ in 0..length {
            self.write_link_rail(HopeFactoRail {
                // -1,-1 to cancel RailStraight's to_fac offset
                position: next_a_pos - VPOINT_ONE,
                direction: middle_a_direction.clone(),
                rtype: FacEntRailType::Straight,
            });
            last_b_pos = next_a_pos.move_direction_usz(&cur_direction, 2);
            self.write_link_rail(HopeFactoRail {
                // -1,-1 to cancel RailStraight's to_fac offset
                position: last_b_pos - VPOINT_ONE,
                direction: middle_b_direction.clone(),
                rtype: FacEntRailType::Straight,
            });
            next_a_pos =
                last_b_pos.move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 2))
        }

        // curve 2 back
        let last_curve_pos = last_b_pos
            .move_direction_usz(&cur_direction, 3)
            .move_direction_sideways_int(&cur_direction, neg_if_false(clockwise, 3));
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
        self.write_link_rail(HopeFactoRail {
            position: last_curve_pos,
            direction: last_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });
        let rails = self.drain_rails_cache();
        self.links.push(HopeLink {
            start: self.current_next_pos(),
            next_direction: cur_direction.clone(),
            rtype: RailHopeLinkType::Shift45 { clockwise, length },
            rails,
        })
    }
}

impl HopeLink {
    fn next_straight_position(&self) -> VPoint {
        match &self.rtype {
            RailHopeLinkType::Straight { length } => self
                .start
                .move_direction_usz(&self.next_direction, length * RAIL_STRAIGHT_DIAMETER),
            RailHopeLinkType::Turn90 { clockwise } => {
                let unrotated = if *clockwise {
                    self.next_direction.rotate_opposite()
                } else {
                    self.next_direction.rotate_once()
                };
                trace!("unrotated {}", unrotated);
                self.start
                    .move_direction_usz(&unrotated, 10)
                    .move_direction_sideways_int(&unrotated, neg_if_false(*clockwise, 12))
            }
            RailHopeLinkType::Shift45 { clockwise, length } => self
                .start
                .move_direction_usz(&self.next_direction, 14 + (*length * 2))
                .move_direction_sideways_int(
                    &self.next_direction,
                    neg_if_false(*clockwise, 6 + (*length as i32 * 2)),
                ),
        }
    }
}

impl HopeFactoRail {
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

impl FacEntRailType {
    fn to_facto_name(&self) -> &str {
        match self {
            Self::Curved => "curved-rail",
            Self::Straight => "straight-rail",
        }
    }
}

fn neg_if_false(flag: bool, value: i32) -> i32 {
    if flag { value } else { -value }
}

#[cfg(test)]
mod test {
    use super::{HopeFactoRail, HopeLink, RailHopeSingle};
    use crate::blueprint::bpfac::entity::FacBpEntity;
    use crate::blueprint::bpfac::position::FacBpPosition;
    use crate::blueprint::contents::BlueprintContents;
    use crate::common::vpoint::VPOINT_TEN;
    use crate::{
        blueprint::output::FacItemOutput, common::vpoint::VPOINT_ZERO,
        game_blocks::rail_hope::RailHopeAppender, game_entities::direction::FacDirectionQuarter,
    };
    use itertools::Itertools;
    use std::borrow::Borrow;

    #[test]
    fn test_straight_chain() {
        let hope_long_output = FacItemOutput::new_blueprint().into_rc();
        let hope_long_next_pos = {
            let mut hope_long = RailHopeSingle::new(
                VPOINT_ZERO,
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
                VPOINT_ZERO,
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

    #[test]
    fn test_turn_90_clw() {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut hope = RailHopeSingle::new(VPOINT_TEN, FacDirectionQuarter::East, output.clone());
        hope.add_turn90(true);
        let links = hope.links.clone();
        drop(hope);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        compare_output(bpcontents, links, [
            (FacBpPosition::new(14.0, 12.0), "curved-rail"),
            (FacBpPosition::new(17.0, 15.0), "straight-rail"),
            (FacBpPosition::new(20.0, 18.0), "curved-rail"),
        ])
    }

    #[test]
    fn test_turn_90_ccw() {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut hope = RailHopeSingle::new(VPOINT_TEN, FacDirectionQuarter::East, output.clone());
        hope.add_turn90(false);
        let links = hope.links.clone();
        drop(hope);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        compare_output(bpcontents, links, [
            (FacBpPosition::new(14.0, 10.0), "curved-rail"),
            (FacBpPosition::new(17.0, 7.0), "straight-rail"),
            (FacBpPosition::new(20.0, 4.0), "curved-rail"),
        ])
    }

    #[test]
    fn test_shift_45_ccw() {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut hope = RailHopeSingle::new(VPOINT_TEN, FacDirectionQuarter::East, output.clone());
        hope.add_shift45(false, 1);
        hope.add_straight(1);
        let links = hope.links.clone();
        drop(hope);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        compare_output(bpcontents, links, [
            (FacBpPosition::new(14.0, 10.0), "curved-rail"),
            (FacBpPosition::new(17.0, 7.0), "straight-rail"),
            (FacBpPosition::new(19.0, 7.0), "straight-rail"),
            (FacBpPosition::new(22.0, 4.0), "curved-rail"),
            (FacBpPosition::new(27.0, 3.0), "straight-rail"),
        ])
    }

    #[test]
    fn test_shift_45_clw() {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut hope = RailHopeSingle::new(VPOINT_TEN, FacDirectionQuarter::East, output.clone());
        hope.add_shift45(true, 1);
        hope.add_straight(1);
        let links = hope.links.clone();
        drop(hope);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        compare_output(bpcontents, links, [
            (FacBpPosition::new(14.0, 12.0), "curved-rail"),
            (FacBpPosition::new(17.0, 15.0), "straight-rail"),
            (FacBpPosition::new(19.0, 15.0), "straight-rail"),
            (FacBpPosition::new(22.0, 18.0), "curved-rail"),
            (FacBpPosition::new(27.0, 19.0), "straight-rail"),
        ])
    }

    // panic!(
    //     "bp {}",
    //     encode_blueprint_to_string_auto_index(bpcontents.into()).unwrap()
    // );

    fn compare_output(
        bpcontents: BlueprintContents,
        links: Vec<HopeLink>,
        expected: impl Borrow<[(FacBpPosition, &'static str)]>,
    ) {
        let expected = expected.borrow();
        let mut is_success = true;
        let entities = bpcontents.fac_entities();
        let links_rails = links.iter().flat_map(|v| &v.rails).collect_vec();

        let entities_len = entities.len();
        let links_rails_len: usize = links_rails.len();
        let expected_len = expected.len();
        assert_eq!(entities_len, expected_len, "bpentities vs expected");
        assert_eq!(entities_len, links_rails_len, "bpentities vs links_cat");

        for (i, FacBpEntity { position, name, .. }) in entities.into_iter().enumerate() {
            let (expected_pos, expected_name) = &expected[i];
            // let (link_pos, link_name) = &links[i];
            let HopeFactoRail {
                position: link_pos,
                rtype: link_type,
                direction: _,
            } = &links_rails[i];
            let link_type_name = link_type.to_facto_name();

            println!(
                "actual {} expected {} {}",
                name,
                expected_name,
                if name == expected_name {
                    ""
                } else {
                    is_success = false;
                    "!!!"
                }
            );
            println!(
                "actual {}     expected {}   {}",
                position.display(),
                expected_pos.display(),
                if position == expected_pos {
                    ""
                } else {
                    is_success = false;
                    "!!!"
                }
            );

            // todo: entities have vpoint these are facto points
            // println!(
            //     "actual {} link expected {}   {}",
            //     position.display(),
            //     link_pos.display(),
            //     if position == link_pos {
            //         ""
            //     } else {
            //         is_success = false;
            //         "!!!"
            //     }
            // );

            println!(
                "actual {} link expected {}   {}",
                name,
                link_type_name,
                if name == link_type_name {
                    ""
                } else {
                    is_success = false;
                    "!!!"
                }
            );
            println!()
        }

        assert!(is_success, "not success");
        assert_eq!(entities_len, expected.len(), "diff len");
    }
}
