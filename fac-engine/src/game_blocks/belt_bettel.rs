use crate::blueprint::output::FacItemOutput;
use crate::common::vpoint::VPoint;
use crate::game_entities::belt::FacEntBeltType;
use crate::game_entities::belt_split::{FacEntBeltSplit, FacEntBeltSplitPriority};
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::belt_under::{FacEntBeltUnder, FacEntBeltUnderType};
use crate::game_entities::direction::FacDirectionQuarter;
use std::borrow::Borrow;
use std::rc::Rc;

/// Belt linkage v1 "Gavis Bettel 🎩"
///
/// Describe belts as a sequence of links,
/// without pathfinding concerns or complicated loop math
#[derive(Clone)]
pub struct FacBlkBettelBelt {
    origin: VPoint,
    origin_direction: FacDirectionQuarter,
    btype: FacEntBeltType,
    links: Vec<FacBlkBettelBeltLink>,
    output: Rc<FacItemOutput>,
    write_cursor: VPoint,
    dummy_nav_mode: bool,
}

#[derive(Clone)]
struct FacBlkBettelBeltLink {
    ltype: FacBlkBettelBeltLinkType,
    direction: FacDirectionQuarter,
}

#[derive(Clone)]
pub enum FacBlkBettelBeltLinkType {
    Transport {
        length: usize,
    },
    Underground {
        length: usize,
    },
    Splitter {
        clockwise: bool,
        priority: FacEntBeltSplitPriority,
    },
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
            dummy_nav_mode: false,
        }
    }

    pub fn set_dummy_nav_mode(&mut self, dummy_nav_mode: bool) {
        self.dummy_nav_mode = dummy_nav_mode;
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
                FacBlkBettelBeltLinkType::Underground { length }
            } else {
                FacBlkBettelBeltLinkType::Transport { length }
            },
            direction,
        })
    }

    pub fn add_straight(&mut self, length: usize) {
        self.add_straight_raw(length, false, *self.current_direction());
    }

    pub fn add_straight_underground(&mut self, length: usize) {
        self.add_straight_raw(length, true, *self.current_direction());
    }

    pub fn add_turn90(&mut self, clockwise: bool) {
        let new_direction = if clockwise {
            self.current_direction().rotate_once()
        } else {
            self.current_direction().rotate_opposite()
        };
        self.add_straight_raw(1, false, new_direction);
    }

    pub fn add_turn90_stacked_row_ccw(&mut self, row_number: usize) {
        self.add_straight(row_number);
        self.add_turn90(false);
        self.add_straight(row_number);
    }

    pub fn add_turn90_stacked_row_clk(&mut self, row_number: usize, total: usize) {
        self.add_straight(total - row_number);
        self.add_turn90(true);
        self.add_straight(total - 1 - row_number);
    }

    pub fn add_split(&mut self, clockwise: bool) {
        self.write_link(FacBlkBettelBeltLink {
            direction: *self.current_direction(),
            ltype: FacBlkBettelBeltLinkType::Splitter {
                clockwise,
                priority: Default::default(),
            },
        });
    }

    pub fn add_split_priority(&mut self, clockwise: bool, priority: FacEntBeltSplitPriority) {
        self.write_link(FacBlkBettelBeltLink {
            direction: *self.current_direction(),
            ltype: FacBlkBettelBeltLinkType::Splitter {
                clockwise,
                priority,
            },
        });
    }

    pub fn next_insert_position(&self) -> VPoint {
        self.write_cursor
    }

    pub fn write_link(&mut self, link: FacBlkBettelBeltLink) {
        let output = if self.dummy_nav_mode {
            &FacItemOutput::new_null()
        } else {
            &self.output
        };
        match &link.ltype {
            FacBlkBettelBeltLinkType::Transport { length } => {
                let mut new_cursor = self.origin;
                for i in 0..*length {
                    new_cursor = self.write_cursor.move_direction_usz(link.direction, i);
                    output.writei(
                        FacEntBeltTransport::new(self.btype, link.direction),
                        new_cursor,
                    )
                }
                // move cursor past the last belt we placed
                self.write_cursor = new_cursor.move_direction_int(link.direction, 1);
            }
            FacBlkBettelBeltLinkType::Underground { length } => {
                output.writei(
                    FacEntBeltUnder::new(self.btype, link.direction, FacEntBeltUnderType::Input),
                    self.write_cursor,
                );

                output.writei(
                    FacEntBeltUnder::new(self.btype, link.direction, FacEntBeltUnderType::Output),
                    self.write_cursor
                        .move_direction_usz(link.direction, *length + 1),
                );

                self.write_cursor = self
                    .write_cursor
                    .move_direction_usz(link.direction, *length + 2)
            }
            FacBlkBettelBeltLinkType::Splitter {
                clockwise,
                priority,
            } => {
                let split_pos = self.write_cursor;
                let new_direction = link.direction.rotate_clockwise(*clockwise);
                let split_pos = split_pos.move_factorio_style_direction(new_direction, 0.5);
                output.writei(
                    FacEntBeltSplit::new_priority(self.btype, link.direction, *priority),
                    split_pos,
                );

                self.write_cursor = self.write_cursor.move_direction_int(link.direction, 1)
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
            dummy_nav_mode,
        } = self;
        if let FacBlkBettelBeltLink {
            direction,
            ltype:
                FacBlkBettelBeltLinkType::Splitter {
                    clockwise,
                    priority: _,
                },
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
                dummy_nav_mode: *dummy_nav_mode,
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
        let clockwise = true;
        let belt_total_0 = belt_total - 1;

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                *btype.borrow(),
                origin.move_y_usize(belt_num),
                FacDirectionQuarter::East,
                output.clone(),
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(clockwise);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(clockwise);
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
        let clockwise = false;
        let belt_total_0 = belt_total - 1;

        for belt_num in 0..belt_total {
            let mut belt: FacBlkBettelBelt = FacBlkBettelBelt::new(
                *btype.borrow(),
                origin.move_xy_usize(belt_total_0, belt_num),
                FacDirectionQuarter::West,
                output.clone(),
            );
            belt.add_straight(belt_total_0 - belt_num);
            belt.add_turn90(clockwise);
            // go down past the middle "cell"
            belt.add_straight((belt_total_0 - belt_num) * 2 + mid_span);
            belt.add_turn90(clockwise);
            belt.add_straight(belt_total_0 - belt_num);
        }
    }

    pub fn turn_and_under(&mut self, clockwise: bool, row_number: usize) {
        self.add_straight(row_number);
        self.add_turn90(clockwise);
        self.add_straight(row_number);
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
            converter::encode_blueprint_to_string_dangerous_index, output::FacItemOutput,
        },
        common::vpoint::VPOINT_TEN,
        game_entities::{belt::FacEntBeltType, direction::FacDirectionQuarter},
    };

    use super::FacBlkBettelBelt;

    // north

    #[test]
    fn split_north_clw() {
        test_split(
            FacDirectionQuarter::North,
            true,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(11.0, 9.5),
                FacBpPosition::new(10.5, 8.5),
            ],
        );
    }

    #[test]
    fn split_north_ccw() {
        test_split(
            FacDirectionQuarter::North,
            false,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(10.0, 9.5),
                FacBpPosition::new(10.5, 8.5),
            ],
        );
    }

    // south

    #[test]
    fn split_south_clw() {
        test_split(
            FacDirectionQuarter::South,
            true,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(10.0, 11.5),
                FacBpPosition::new(10.5, 12.5),
            ],
        );
    }

    #[test]
    fn split_south_ccw() {
        test_split(
            FacDirectionQuarter::South,
            false,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(11.0, 11.5),
                FacBpPosition::new(10.5, 12.5),
            ],
        );
    }

    // east

    #[test]
    fn split_east_clk() {
        test_split(
            FacDirectionQuarter::East,
            true,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(11.5, 11.0),
                FacBpPosition::new(12.5, 10.5),
            ],
        );
    }

    #[test]
    fn split_east_ccw() {
        test_split(
            FacDirectionQuarter::East,
            false,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(11.5, 10.0),
                FacBpPosition::new(12.5, 10.5),
            ],
        );
    }

    // east

    #[test]
    fn split_west_clk() {
        test_split(
            FacDirectionQuarter::West,
            true,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(9.5, 10.0),
                FacBpPosition::new(8.5, 10.5),
            ],
        );
    }

    #[test]
    fn split_west_ccw() {
        test_split(
            FacDirectionQuarter::West,
            false,
            &[
                FacBpPosition::new(10.5, 10.5),
                FacBpPosition::new(9.5, 11.0),
                FacBpPosition::new(8.5, 10.5),
            ],
        );
    }

    //

    #[test]
    fn split_test() {
        let output = FacItemOutput::new_blueprint().into_rc();
        let mut is_error = false;

        let mut belt = FacBlkBettelBelt::new(
            FacEntBeltType::Basic,
            VPOINT_TEN,
            FacDirectionQuarter::East,
            output.clone(),
        );
        belt.add_straight(1);
        expect_output(FacBpPosition::new(10.5, 10.5), &output, &mut is_error);
        belt.add_split(true);
        expect_output(FacBpPosition::new(11.5, 11.0), &output, &mut is_error);

        let mut side_belt = belt.belt_for_splitter();
        side_belt.add_straight(1);
        expect_output(FacBpPosition::new(12.5, 11.5), &output, &mut is_error);
        drop(belt);
        drop(side_belt);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        assert_eq!(bpcontents.fac_entities().len(), 3, "too many entities");
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
        expect_output_slice(&output, all_expected, &mut expected_i, &mut is_error);
        belt.add_split(clockwise);
        expect_output_slice(&output, all_expected, &mut expected_i, &mut is_error);
        belt.add_straight(1);
        expect_output_slice(&output, all_expected, &mut expected_i, &mut is_error);
        drop(belt);

        let bpcontents = output.consume_rc().into_blueprint_contents();
        if is_error {
            panic!(
                "blueprint {}",
                encode_blueprint_to_string_dangerous_index(&bpcontents.into()).unwrap()
            );
        }
        assert_eq!(
            bpcontents.fac_entities().len(),
            expected_i,
            "too many entities"
        );
        // compare_output_to_expected(bpcontents, all_expected);
    }

    fn expect_output_slice(
        output: &FacItemOutput,
        all_expected: &[FacBpPosition],
        expected_i: &mut usize,
        is_error: &mut bool,
    ) {
        let output_write = output.last_blueprint_write();
        let actual_pos = output_write.blueprint.position;
        let expected = &all_expected[*expected_i];
        *expected_i += 1;
        let err = if &actual_pos != expected {
            *is_error = true;
            "!!!!"
        } else {
            ""
        };
        println!("facpos gen {actual_pos} expect {expected} {err}",);
    }

    fn expect_output(expected: FacBpPosition, output: &FacItemOutput, is_error: &mut bool) {
        let output_write = output.last_blueprint_write();
        let actual_pos = output_write.blueprint.position;
        let err = if actual_pos != expected {
            *is_error = true;
            "!!!!"
        } else {
            ""
        };
        println!("facpos gen {actual_pos} expect {expected} {err}",);
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
                "facpos gen {facpos} expect {expected} {:?} {err}",
                bp_item.entity().rectangle_size()
            );
        }

        if is_error {
            panic!(
                "blueprint {}",
                encode_blueprint_to_string_dangerous_index(&bp.into()).unwrap()
            );
        }
        let item_len = bp.items().len();
        if item_len != expected.len() {
            println!("mispatched sizes {} expected {}", item_len, expected.len(),)
        }
    }
}
