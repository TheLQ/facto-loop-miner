use crate::blueprint::bpitem::BlueprintItem;
use crate::blueprint::output::FacItemOutput;
use crate::common::entity::FacEntity;
use crate::common::vpoint::VPoint;
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_split::FacEntBeltSplit;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::belt_under::{FacEntBeltUnder, FacEntBeltUnderType};
use crate::game_entities::direction::FacDirectionQuarter;
use std::borrow::Borrow;
use std::rc::Rc;

/// Belt linkage v1 "Gavis Bettel 🎩"
///
/// Describe belts as a sequence of links,
/// without pathfinding concerns or complicated loop math
pub struct FacBlkBettelBelt {
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
    btype: FacEntBeltType,
    links: Vec<FacBlkBettelBeltLink>,
    output: Rc<FacItemOutput>,
    write_cursor: VPoint,
}

pub struct FacBlkBettelBeltLink {
    ltype: FacBlkBettelBeltLinkType,
    direction: FacDirectionQuarter,
}

pub enum FacBlkBettelBeltLinkType {
    Transport { length: usize },
    Underground { length: usize },
    Splitter { clockwise: bool },
}

impl FacBlkBettelBelt {
    pub fn new(
        btype: FacEntBeltType,
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
        output: Rc<FacItemOutput>,
    ) -> Self {
        Self {
            btype,
            origin,
            origin_direction,
            links: Vec::new(),
            output,
            write_cursor: origin,
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

        self.write_link(FacBlkBettelBeltLink {
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

    pub fn add_split(&mut self, clockwise: bool) {
        self.write_link(FacBlkBettelBeltLink {
            direction: *self.current_direction(),
            ltype: FacBlkBettelBeltLinkType::Splitter { clockwise },
        });
    }

    pub fn write_link(&mut self, link: FacBlkBettelBeltLink) {
        match &link.ltype {
            FacBlkBettelBeltLinkType::Transport { length } => {
                let mut new_cursor = self.origin;
                for i in 0..*length {
                    new_cursor = self.write_cursor.move_direction_usz(&link.direction, i);
                    self.output.write(BlueprintItem::new(
                        FacEntBeltTransport::new(self.btype.clone(), link.direction.clone())
                            .into_boxed(),
                        new_cursor,
                    ))
                }
                // move cursor past the last belt we placed
                self.write_cursor = new_cursor.move_direction_int(&link.direction, 1);
            }
            FacBlkBettelBeltLinkType::Underground { length } => {
                self.output.write(BlueprintItem::new(
                    FacEntBeltUnder::new(
                        self.btype.clone(),
                        link.direction.clone(),
                        FacEntBeltUnderType::Input,
                    )
                    .into_boxed(),
                    self.write_cursor,
                ));

                self.output.write(BlueprintItem::new(
                    FacEntBeltUnder::new(
                        self.btype.clone(),
                        link.direction.clone(),
                        FacEntBeltUnderType::Output,
                    )
                    .into_boxed(),
                    self.write_cursor
                        .move_direction_usz(&link.direction, *length),
                ));

                self.write_cursor = self
                    .write_cursor
                    .move_direction_usz(&link.direction, *length + 1)
            }
            FacBlkBettelBeltLinkType::Splitter { clockwise } => {
                let split_pos = self.write_cursor;
                let new_direction = link.direction.rotate_clockwise(*clockwise);
                let split_pos = split_pos.move_factorio_style_direction(new_direction, 0.5);
                self.output.write(BlueprintItem::new(
                    FacEntBeltSplit::new(self.btype, link.direction).into_boxed(),
                    split_pos,
                ));

                self.write_cursor = self.write_cursor.move_direction_int(&link.direction, 1)
            }
        };
        self.links.push(link);
    }

    pub fn belt_for_splitter(&self) -> FacBlkBettelBelt {
        let Self {
            btype,
            links,
            origin: _,
            origin_direction,
            output,
            write_cursor,
        } = self;
        if let FacBlkBettelBeltLink {
            direction,
            ltype: FacBlkBettelBeltLinkType::Splitter { clockwise },
        } = &links.last().unwrap()
        {
            let new_direction = direction.rotate_clockwise(*clockwise);
            let origin = write_cursor
                // .move_direction_int(direction, 1)
                .move_direction_int(new_direction, 1);
            FacBlkBettelBelt {
                btype: *btype,
                links: Vec::new(),
                origin,
                origin_direction: *origin_direction,
                output: output.clone(),
                write_cursor: origin,
            }
        } else {
            panic!("not after inserting splitter")
        }
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
        output: Rc<FacItemOutput>,
    ) {
        let belt_total_0 = belt_total - 1;

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                btype.borrow().clone(),
                origin.move_y_usize(belt_num),
                FacDirectionQuarter::East,
                output.clone(),
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(false);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(false);
            belt.add_straight(belt_total_0 - belt_num);
        }
    }

    pub fn u_turn_from_west(
        btype: impl Borrow<FacEntBeltType>,
        origin: VPoint,
        mid_span: usize,
        belt_total: usize,
        output: Rc<FacItemOutput>,
    ) {
        let belt_total_0 = belt_total - 1;

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                btype.borrow().clone(),
                origin.move_xy_usize(belt_total_0, belt_num),
                FacDirectionQuarter::West,
                output.clone(),
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(true);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(true);
            belt.add_straight(belt_total_0 - belt_num);
        }
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

#[cfg(test)]
mod test {
    use crate::{
        blueprint::{
            bpfac::position::FacBpPosition, contents::BlueprintContents,
            converter::encode_blueprint_to_string, output::FacItemOutput,
        },
        common::vpoint::VPOINT_TEN,
        game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
    };

    use super::FacBlkBettelBelt;

    // north

    #[test]
    fn split_north_clw() {
        test_split(FacDirectionQuarter::North, true, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(11.0, 9.5),
            FacBpPosition::new(10.5, 8.5),
        ]);
    }

    #[test]
    fn split_north_ccw() {
        test_split(FacDirectionQuarter::North, false, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(10.0, 9.5),
            FacBpPosition::new(10.5, 8.5),
        ]);
    }

    // south

    #[test]
    fn split_south_clw() {
        test_split(FacDirectionQuarter::South, true, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(10.0, 11.5),
            FacBpPosition::new(10.5, 12.5),
        ]);
    }

    #[test]
    fn split_south_ccw() {
        test_split(FacDirectionQuarter::South, false, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(11.0, 11.5),
            FacBpPosition::new(10.5, 12.5),
        ]);
    }

    // east

    #[test]
    fn split_east_clk() {
        test_split(FacDirectionQuarter::East, true, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(11.5, 11.0),
            FacBpPosition::new(12.5, 10.5),
        ]);
    }

    #[test]
    fn split_east_ccw() {
        test_split(FacDirectionQuarter::East, false, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(11.5, 10.0),
            FacBpPosition::new(12.5, 10.5),
        ]);
    }

    // east

    #[test]
    fn split_west_clk() {
        test_split(FacDirectionQuarter::West, true, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(9.5, 10.0),
            FacBpPosition::new(8.5, 10.5),
        ]);
    }

    #[test]
    fn split_west_ccw() {
        test_split(FacDirectionQuarter::West, false, &[
            FacBpPosition::new(10.5, 10.5),
            FacBpPosition::new(9.5, 11.0),
            FacBpPosition::new(8.5, 10.5),
        ]);
    }

    //

    fn test_split(
        origin_direction: FacDirectionQuarter,
        clockwise: bool,
        all_expected: &[FacBpPosition],
    ) {
        let output = FacItemOutput::new_blueprint().into_rc();
        let mut expected_i = 0;
        let mut is_error = false;

        let mut belt = FacBlkBettelBelt::new(
            FacEntBeltType::Basic,
            VPOINT_TEN,
            origin_direction,
            output.clone(),
        );
        belt.add_straight(1);
        expect_output(&output, all_expected, &mut expected_i, &mut is_error);
        belt.add_split(clockwise);
        expect_output(&output, all_expected, &mut expected_i, &mut is_error);
        belt.add_straight(1);
        expect_output(&output, all_expected, &mut expected_i, &mut is_error);
        drop(belt);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        if is_error {
            panic!(
                "blueprint {}",
                encode_blueprint_to_string(&bpcontents.into()).unwrap()
            );
        }

        assert_eq!(
            bpcontents.fac_entities().len(),
            expected_i,
            "too many entities"
        );
        // compare_output_to_expected(bpcontents, all_expected);
    }

    fn expect_output(
        output: &FacItemOutput,
        all_expected: &[FacBpPosition],
        expected_i: &mut usize,
        is_error: &mut bool,
    ) {
        let output_write = output.last_blueprint_write_last();
        let actual_pos = output_write.blueprint.position;
        let expected = &all_expected[*expected_i];
        *expected_i += 1;
        let err = if &actual_pos != expected {
            *is_error = true;
            "!!!!"
        } else {
            ""
        };
        println!(
            "facpos gen {:10} expect {:10} {err}",
            actual_pos.display(),
            expected.display(),
        );
    }

    #[allow(dead_code)]
    fn compare_output_to_expected(bp: BlueprintContents, expected: &[FacBpPosition]) {
        // output: Rc<FacItemOutput>
        // let bp = output.consume_rc().into_blueprint_contents();
        let mut is_error = false;
        for (i, bp_item) in bp.items().iter().enumerate() {
            let facpos = bp_item.to_blueprint().position;
            let expected = &expected[i];
            let err = if &facpos != expected {
                is_error = true;
                "!!!!"
            } else {
                ""
            };
            println!(
                "facpos gen {:10} expect {:10} {:?} {err}",
                facpos.display(),
                expected.display(),
                bp_item.entity().rectangle_size()
            );
        }

        if is_error {
            panic!(
                "blueprint {}",
                encode_blueprint_to_string(&bp.into()).unwrap()
            );
        }
        let item_len = bp.items().len();
        if item_len != expected.len() {
            println!("mispatched sizes {} expected {}", item_len, expected.len(),)
        }
    }
}
