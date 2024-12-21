use crate::blueprint::bpitem::BlueprintItem;
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::belt_under::{FacEntBeltUnder, FacEntBeltUnderType};
use crate::game_entities::direction::FacDirectionQuarter;
use std::borrow::Borrow;

/// Belt linkage v1 "Gavis Bettel 🎩"
///
/// Describe belts as a sequence of links,
/// without pathfinding concerns or complicated loop math
pub struct FacBlkBettelBelt {
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
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
        Self {
            btype,
            origin,
            origin_direction,
            links: Vec::new(),
        }
    }

    fn add_straight_raw(
        &mut self,
        length: usize,
        is_underground: bool,
        direction: FacDirectionQuarter,
    ) {
        // assert_ne!(length, 0, "length cannot be 0");
        if length == 0 {
            // warn!("not adding empty straight");
            return;
        }

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

    pub fn add_straight(&mut self, length: usize) {
        self.add_straight_raw(length, false, self.current_direction().clone());
    }

    pub fn add_straight_underground(&mut self, length: usize) {
        self.add_straight_raw(length, true, self.current_direction().clone());
    }

    pub fn add_turn90(&mut self, clockwise: bool) {
        let new_direction = if clockwise {
            self.current_direction().rotate_once()
        } else {
            self.current_direction().rotate_opposite()
        };
        self.add_straight_raw(1, false, new_direction);
    }

    pub fn to_fac(&self) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        let mut cursor = self.origin;
        for link in &self.links {
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
        }
        res
    }

    fn current_direction(&self) -> &FacDirectionQuarter {
        self.links
            .last()
            .map(|v| &v.direction)
            .unwrap_or(&self.origin_direction)
    }

    pub fn u_turn_from_east(
        btype: impl Borrow<FacEntBeltType>,
        origin: VPoint,
        mid_span: usize,
        belt_total: usize,
    ) -> Vec<BlueprintItem> {
        let belt_total_0 = belt_total - 1;
        let mut res = Vec::new();

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                btype.borrow().clone(),
                origin.move_y_usize(belt_num),
                FacDirectionQuarter::East,
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(false);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(false);
            belt.add_straight(belt_total_0 - belt_num);

            res.extend(belt.to_fac());
        }

        res
    }

    pub fn u_turn_from_west(
        btype: impl Borrow<FacEntBeltType>,
        origin: VPoint,
        mid_span: usize,
        belt_total: usize,
    ) -> Vec<BlueprintItem> {
        let belt_total_0 = belt_total - 1;
        let mut res = Vec::new();

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                btype.borrow().clone(),
                origin.move_xy_usize(belt_total_0, belt_num),
                FacDirectionQuarter::West,
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(true);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(true);
            belt.add_straight(belt_total_0 - belt_num);

            res.extend(belt.to_fac());
        }

        res
    }
}

impl FacBlkBettelBeltLinkType {
    // fn length(&self) -> usize {
    //     match self {
    //         FacBlkBettelBeltLinkType::Transport { length } => *length,
    //         FacBlkBettelBeltLinkType::Underground { length } => *length,
    //         FacBlkBettelBeltLinkType::Splitter => 1,
    //     }
    // }
}
