use crate::blueprint::bpitem::BlueprintItem;
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::belt_under::{FacEntBeltUnder, FacEntBeltUnderType};
use crate::game_entities::direction::FacDirectionQuarter;
use tracing::debug;

/// Belt linkage v1 "Gavis Bettel"
/// Describe belts as a sequence of links
pub struct FacBlkBettelBelt {
    origin: VPoint,
    btype: FacEntBeltType,
    links: Vec<FacBlkBettelBeltLink>,
}

pub struct FacBlkBettelBeltLink {
    ltype: FacBlkBettelBeltLinkType,
    direction: FacDirectionQuarter,
}

pub enum FacBlkBettelBeltLinkType {
    Transport { length: usize },
    Underground { length: usize },
    Splitter,
}

impl FacBlkBettelBelt {
    pub fn new(
        btype: FacEntBeltType,
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
    ) -> Self {
        let mut res = Self {
            btype,
            origin,
            links: Vec::new(),
        };
        res.add_straight_raw(1, false, origin_direction);
        res
    }

    fn add_straight_raw(
        &mut self,
        length: usize,
        is_underground: bool,
        direction: FacDirectionQuarter,
    ) {
        assert_ne!(length, 0, "length cannot be 0");
        self.links.push(FacBlkBettelBeltLink {
            ltype: if is_underground {
                assert!(length > 2, "underground length {} too short", length);
                FacBlkBettelBeltLinkType::Underground { length }
            } else {
                FacBlkBettelBeltLinkType::Transport { length }
            },
            direction,
        })
    }

    pub fn add_straight(&mut self, length: usize, is_underground: bool) {
        let last_link = self.last_link();
        self.add_straight_raw(length, is_underground, last_link.direction.clone());
    }

    pub fn add_turn90(&mut self, opposite: bool) {
        let last_link = self.last_link();
        let new_direction = if opposite {
            last_link.direction.rotate_opposite()
        } else {
            last_link.direction.rotate_once()
        };
        self.add_straight_raw(1, false, new_direction);
    }

    pub fn to_fac(&self) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        // let mut previous_direction = &self.links[0].direction;
        let mut cursor = self.origin;
        for link in &self.links {
            debug!("moving {}", link.direction);
            match &link.ltype {
                FacBlkBettelBeltLinkType::Transport { length } => {
                    let mut last_cursor = cursor;
                    for i in 0..*length {
                        last_cursor = cursor.move_direction(&link.direction, i);
                        res.push(BlueprintItem::new(
                            FacEntBeltTransport::new(self.btype.clone(), link.direction.clone())
                                .into_boxed(),
                            last_cursor,
                        ))
                    }
                    // move cursor past the last belt we placed
                    cursor = last_cursor.move_direction(&link.direction, 1);
                }
                FacBlkBettelBeltLinkType::Underground { length } => {
                    res.push(BlueprintItem::new(
                        FacEntBeltUnder::new(
                            self.btype.clone(),
                            link.direction.clone(),
                            FacEntBeltUnderType::Input,
                        )
                        .into_boxed(),
                        cursor,
                    ));

                    res.push(BlueprintItem::new(
                        FacEntBeltUnder::new(
                            self.btype.clone(),
                            link.direction.clone(),
                            FacEntBeltUnderType::Output,
                        )
                        .into_boxed(),
                        cursor.move_direction(&link.direction, *length),
                    ));

                    cursor = cursor.move_direction(&link.direction, *length + 1)
                }
                FacBlkBettelBeltLinkType::Splitter => unimplemented!(),
            };
            // previous_direction = &link.direction;
        }
        res
    }

    fn last_link(&self) -> &FacBlkBettelBeltLink {
        // should always have something
        self.links.last().unwrap()
    }
}

impl FacBlkBettelBeltLinkType {
    fn length(&self) -> usize {
        match self {
            FacBlkBettelBeltLinkType::Transport { length } => *length,
            FacBlkBettelBeltLinkType::Underground { length } => *length,
            FacBlkBettelBeltLinkType::Splitter => 1,
        }
    }
}
