use crate::blueprint::bpitem::BlueprintItem;
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope::RailHopeAppender;
use crate::game_entities::direction::{FacDirectionEighth, FacDirectionQuarter};
use crate::game_entities::rail::{FacEntRail, FacEntRailType, RAIL_STRAIGHT_DIAMETER};

/// Rail Pathing v10?, "Irys Hope"
///
/// Describe Rail as a self-contained sequence of links,
/// without intermingled "Mori"'s Pathfinding concerns
pub struct RailHopeSingle {
    links: Vec<RailHopeLink>,
}

pub struct RailHopeLink {
    start_pos: VPoint,
    rtype: RailHopeLinkType,
    link_direction: FacDirectionQuarter,
    rails: Vec<RailHopeLinkRail>,
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
        let mut res = Self { links: Vec::new() };
        res.add_straight_line_raw(origin, origin_direction, 8);
        res
    }

    pub fn compress_straight(&mut self) {}

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

    fn last_link(&self) -> &RailHopeLink {
        // we always should have a link
        &self.links.last().unwrap()
    }
}

impl RailHopeAppender for RailHopeSingle {
    fn add_straight(&mut self, length: usize) {
        let last_link = self.last_link();
        self.add_straight_line_raw(
            last_link.next_straight_position(),
            last_link.link_direction.clone(),
            length,
        );
    }

    fn add_turn90(&mut self, opposite: bool) {}

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
            _ => todo!("wip"),
        }
    }

    fn to_fac(&self, res: &mut Vec<BlueprintItem>) {
        for rail in &self.rails {
            res.push(BlueprintItem::new(
                FacEntRail::new(rail.rtype.clone(), rail.direction.clone()).into_boxed(),
                rail.position,
            ))
        }
    }
}
