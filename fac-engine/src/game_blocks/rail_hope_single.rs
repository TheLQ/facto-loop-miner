use crate::blueprint::bpitem::BlueprintItem;
use crate::common::entity::FacEntity;
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_entities::direction::{FacDirectionEighth, FacDirectionQuarter};
use crate::game_entities::rail::{FacEntRailStraight, RAIL_STRAIGHT_DIAMETER};
use crate::game_entities::rail_curved::FacEntRailCurved;

/// Rail Pathing v10.999?, "Irys Hope"
///
/// Describe Rail as a self-contained sequence of links,
/// without significant Pathfinding-specific code overhead
pub struct RailHopeSingle {
    links: Vec<RailHopeLink>,
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
}

pub struct RailHopeLink {
    start_pos: VPoint,
    rtype: RailHopeLinkType,
    link_direction: FacDirectionQuarter,
    rails: Vec<RailHopeLinkRail>,
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
    Turn90 { opposite: bool },
    Shift45 { opposite: bool, length: usize },
}

impl RailHopeSingle {
    pub fn new(origin: VPoint, origin_direction: FacDirectionQuarter) -> Self {
        origin.assert_even_position();
        Self {
            links: Vec::new(),
            origin,
            origin_direction,
        }
    }

    // pub fn compress_straight(&mut self) {}

    fn add_straight_line_raw(
        &mut self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        length: usize,
    ) {
        let mut rails = Vec::new();
        for i in 0..length {
            rails.push(RailHopeLinkRail {
                position: origin.move_direction(&direction, i * RAIL_STRAIGHT_DIAMETER),
                direction: direction.to_direction_eighth(),
                rtype: FacEntRailType::Straight,
            })
        }
        self.links.push(RailHopeLink {
            start_pos: origin,
            link_direction: direction,
            rails,
            rtype: RailHopeLinkType::Straight { length },
        })
    }

    // fn last_link(&self) -> &RailHopeLink {
    //     // we always should have a link
    //     self.links.last().unwrap()
    // }

    fn current_direction(&self) -> &FacDirectionQuarter {
        self.links
            .last()
            .map(|v| &v.link_direction)
            .unwrap_or(&self.origin_direction)
    }

    fn current_next_pos(&self) -> VPoint {
        self.links
            .last()
            .map(|v| v.next_straight_position())
            .unwrap_or(self.origin)
    }
}

impl RailHopeAppender for RailHopeSingle {
    fn add_straight(&mut self, length: usize) {
        self.add_straight_line_raw(
            self.current_next_pos(),
            self.current_direction().clone(),
            length,
        );
    }

    fn add_turn90(&mut self, opposite: bool) {
        /*
        Factorio 1 Rails are really complicated
        This is version 3544579 💎, written obviously generic with hindsight

        Order is Curve > Straight 45 > Curve

        In X, steps are 3 > 3 > 3
        In Y, steps are 1 > 3 > 3
        (signs and axis depend on direction)

        Compass "normal" is counter-clockwise, opposite is clockwise (my decision)
        Normal   rotations are 0 > 2 > 2
        Opposite rotations are 1 > 2 > 1
        (yes, curved rail from North to NorthWest in Factorio is... curved-rail North?)
        */

        let cur_direction = self.current_direction();
        // 1,1 to cancel RailStraight's to_fac offset
        let origin_fac = self.current_next_pos() + VPOINT_ONE;

        let mut rails = Vec::new();

        // curve 1
        let first_curve_pos = origin_fac
            .move_direction(cur_direction, 3)
            .move_direction_sideways(cur_direction, neg_opposite(opposite, -1));
        let first_curve_direction = cur_direction.to_direction_eighth();
        let first_curve_direction = if opposite {
            first_curve_direction.rotate_once()
        } else {
            first_curve_direction
        };
        rails.push(RailHopeLinkRail {
            position: first_curve_pos,
            direction: first_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        // middle
        let middle_straight_pos = first_curve_pos
            .move_direction(cur_direction, 3)
            .move_direction_sideways(cur_direction, neg_opposite(opposite, -3));
        let middle_straight_direction = if opposite {
            first_curve_direction.rotate_opposite().rotate_opposite()
        } else {
            first_curve_direction.rotate_once()
        };
        rails.push(RailHopeLinkRail {
            // -1,-1 to cancel RailStraight's to_fac offset
            position: middle_straight_pos - VPOINT_ONE,
            direction: middle_straight_direction.clone(),
            rtype: FacEntRailType::Straight,
        });

        // curve 2
        let last_curve_pos = middle_straight_pos
            .move_direction(cur_direction, 3)
            .move_direction_sideways(cur_direction, neg_opposite(opposite, -3));
        let last_curve_direction = if opposite {
            middle_straight_direction.rotate_opposite()
        } else {
            middle_straight_direction.rotate_once().rotate_once()
        };
        rails.push(RailHopeLinkRail {
            position: last_curve_pos,
            direction: last_curve_direction.clone(),
            rtype: FacEntRailType::Curved,
        });

        // where to go next
        let end_direction = if opposite {
            cur_direction.rotate_opposite()
        } else {
            cur_direction.rotate_once()
        };
        self.links.push(RailHopeLink {
            start_pos: self.current_next_pos(),
            link_direction: end_direction,
            rails,
            rtype: RailHopeLinkType::Turn90 { opposite },
        })
    }

    fn to_fac(&self) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        for link in &self.links {
            link.to_fac(&mut res);
        }
        res
    }
}

impl RailHopeLink {
    fn next_straight_position(&self) -> VPoint {
        match &self.rtype {
            RailHopeLinkType::Straight { length } => self
                .start_pos
                .move_direction(&self.link_direction, length * RAIL_STRAIGHT_DIAMETER),
            RailHopeLinkType::Turn90 { opposite } => {
                let unrotated = if *opposite {
                    self.link_direction.rotate_once()
                } else {
                    self.link_direction.rotate_opposite()
                };
                self.start_pos
                    .move_direction(&unrotated, 10)
                    .move_direction_sideways(&unrotated, neg_opposite(*opposite, -14))
            }
            _ => todo!("wip"),
        }
    }

    fn to_fac(&self, res: &mut Vec<BlueprintItem>) {
        for rail in &self.rails {
            match rail.rtype {
                FacEntRailType::Straight => res.push(BlueprintItem::new(
                    FacEntRailStraight::new(rail.direction.clone()).into_boxed(),
                    rail.position,
                )),
                FacEntRailType::Curved => res.push(BlueprintItem::new(
                    FacEntRailCurved::new(rail.direction.clone()).into_boxed(),
                    rail.position,
                )),
            }
        }
    }
}

fn neg_opposite(opposite: bool, value: i32) -> i32 {
    if opposite { -value } else { value }
}

#[cfg(test)]
mod test {
    use crate::{
        blueprint::bpfac::entity::FacBpEntity, common::vpoint::VPoint,
        game_blocks::rail_hope::RailHopeAppender, game_entities::direction::FacDirectionQuarter,
    };

    use super::RailHopeSingle;

    #[test]
    fn test_straight_chain() {
        let mut hope_long = RailHopeSingle::new(VPoint::zero(), FacDirectionQuarter::North);
        hope_long.add_straight(2);
        hope_long.add_straight(3);
        hope_long.add_straight(6);

        let mut hope_short = RailHopeSingle::new(VPoint::zero(), FacDirectionQuarter::North);
        hope_short.add_straight(11);

        assert_eq!(
            hope_long
                .to_fac()
                .into_iter()
                .map(|v| v.to_blueprint())
                .collect::<Vec<FacBpEntity>>(),
            hope_short
                .to_fac()
                .into_iter()
                .map(|v| v.to_blueprint())
                .collect::<Vec<FacBpEntity>>(),
        );

        assert_eq!(hope_long.current_next_pos(), hope_short.current_next_pos());
    }
}
