use crate::common::vpoint::VPoint;
use crate::game_blocks::rail_hope::RailHopeLink;
use crate::game_blocks::rail_hope_single::{HopeFactoRail, HopeLink, HopeLinkType};
use crate::game_entities::direction::FacDirectionQuarter;
use std::borrow::Borrow;

/// Rail Dual v2 "Irys💎 Soda"
///
/// Define as a grid of "Soda" (aka block, but term is overloaded)
/// with limited information: centerpoint and direction.
/// Radically simpler movement API.
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct HopeSodaLink {
    stype: SodaType,
    source_direction: FacDirectionQuarter,
    center: VPoint,
}

#[derive(Eq, PartialEq, Hash, Clone)]
enum SodaType {
    Straight,
    Turn90 { clockwise: bool },
}

pub(super) const SODA_RAILS_NUM: usize = 13;
const SODA_CENTER_OFFSET_I32: i32 = 13;
pub(super) const SODA_SIZE: i32 = SODA_CENTER_OFFSET_I32 * 2;

impl HopeSodaLink {
    pub fn new_soda_straight(center: VPoint, source_direction: FacDirectionQuarter) -> Self {
        Self {
            stype: SodaType::Straight,
            center,
            source_direction,
        }
    }

    pub(crate) fn new_soda_turn(
        center: VPoint,
        source_direction: FacDirectionQuarter,
        clockwise: bool,
    ) -> Self {
        Self {
            stype: SodaType::Turn90 { clockwise },
            center,
            source_direction,
        }
    }

    fn links_source(&self) -> [HopeLink; 2] {
        // let direction = match (self.stype, self.source_direction) {
        //     (SodaType::Straight, _) => self.source_direction,
        //     (SodaType::Turn90 { clockwise }, FacDirectionQuarter::North | FacDirectionQuarter::West) => {
        //         // undo rotation
        //         self.source_direction.rotate_clockwise(!clockwise)
        //     }
        //     (SodaType::Turn90 { clockwise }, direction) => {
        //         // undo rotation
        //         direction.rotate_clockwise(!clockwise)
        //     }
        // };
        let direction = match self.stype {
            SodaType::Straight => self.source_direction,
            SodaType::Turn90 { clockwise } => {
                // undo rotation
                self.source_direction.rotate_clockwise(!clockwise)
            }
        };

        let border = self
            .center
            .move_direction_int(direction, -SODA_CENTER_OFFSET_I32 + 1);
        let source_a = border.move_direction_sideways_axis_int(direction, 2);
        source_a.assert_even_position();
        let source_b = border.move_direction_sideways_axis_int(direction, -2);
        source_b.assert_even_position();
        [
            HopeLink::new_single(source_a, direction),
            HopeLink::new_single(source_b, direction),
        ]
    }

    fn links_for_soda(&self) -> Vec<HopeLink> {
        let mut output;

        let mut sources = self.links_source();
        match self.stype {
            SodaType::Straight => {
                output = Vec::with_capacity(2);
                output.extend(sources.map(|v| v.add_straight(SODA_RAILS_NUM)));
                assert_eq!(output.len(), 2);
            }
            SodaType::Turn90 { clockwise } => {
                if let FacDirectionQuarter::East | FacDirectionQuarter::South =
                    self.source_direction
                {
                    sources.swap(0, 1);
                }

                // sources.swap(0, 1);
                output = Vec::with_capacity(4);
                output.extend(create_turn_link_from(&sources[0], clockwise));
                output.push(sources[1].add_turn90(clockwise));
                assert_eq!(output.len(), 4);
            }
        }

        output
    }

    pub fn corners(&self) -> [VPoint; 4] {
        [
            self.center
                .move_xy(-SODA_CENTER_OFFSET_I32, -SODA_CENTER_OFFSET_I32),
            self.center
                .move_xy(-SODA_CENTER_OFFSET_I32, SODA_CENTER_OFFSET_I32),
            self.center
                .move_xy(SODA_CENTER_OFFSET_I32, -SODA_CENTER_OFFSET_I32),
            self.center
                .move_xy(SODA_CENTER_OFFSET_I32, SODA_CENTER_OFFSET_I32),
        ]
    }
}

impl RailHopeLink for HopeSodaLink {
    fn add_straight(&self, length: usize) -> Self {
        todo!()
    }

    fn add_straight_section(&self) -> Self {
        let center = self
            .center
            .move_direction_int(self.source_direction, SODA_SIZE);
        Self {
            stype: SodaType::Straight,
            center,
            source_direction: self.source_direction,
        }
    }

    fn add_turn90(&self, clockwise: bool) -> Self {
        let mut next = self.add_straight_section();
        next.stype = SodaType::Turn90 { clockwise };
        next.source_direction = self.source_direction.rotate_clockwise(clockwise);
        next
    }

    fn add_shift45(&self, clockwise: bool, length: usize) -> Self {
        todo!()
    }

    fn link_type(&self) -> HopeLinkType {
        match self.stype {
            SodaType::Straight => HopeLinkType::Straight {
                length: SODA_RAILS_NUM,
            },
            SodaType::Turn90 { clockwise } => HopeLinkType::Turn90 { clockwise },
        }
    }

    fn pos_start(&self) -> VPoint {
        self.center
    }

    fn pos_next(&self) -> VPoint {
        // todo: this normally means "current" but doesn't make sense at all here
        self.center
    }

    fn area(&self, output: &mut Vec<VPoint>) {
        for link in self.links_for_soda() {
            link.area(output);
        }
    }
}

fn create_turn_link_from(link: &HopeLink, clockwise: bool) -> [HopeLink; 3] {
    let first = link.add_straight(2);
    let middle = first.add_turn90(clockwise);
    let last = middle.add_straight(2);
    [first, middle, last]
}

pub fn sodas_to_links(
    input: impl IntoIterator<Item = impl Borrow<HopeSodaLink>>,
) -> impl Iterator<Item = HopeLink> {
    input.into_iter().flat_map(|v| v.borrow().links_for_soda())
}

pub fn sodas_to_rails(
    input: impl IntoIterator<Item = impl Borrow<HopeSodaLink>>,
) -> impl Iterator<Item = HopeFactoRail> {
    input
        .into_iter()
        .flat_map(|v| v.borrow().links_for_soda())
        .flat_map(|v| v.rails)
}

#[cfg(test)]
mod test {
    use crate::blueprint::output::FacItemOutput;
    use crate::common::vpoint::VPOINT_TEN;
    use crate::game_blocks::rail_hope::RailHopeLink;
    use crate::game_blocks::rail_hope_soda::{HopeSodaLink, sodas_to_rails};
    use crate::game_entities::direction::FacDirectionQuarter;
    use itertools::Itertools;

    #[test]
    fn straight_chain() {
        let source = HopeSodaLink::new_soda_straight(VPOINT_TEN, FacDirectionQuarter::East);
        let then = source.add_straight_section();
        let after = then.add_straight_section();
        let sodas = [source, then, after];

        let output = FacItemOutput::new_blueprint();

        let rails = sodas_to_rails(sodas).collect_vec();

        let mut points = rails.iter().map(|v| v.position).collect_vec();
        let points_num_before = points.len();
        points.sort();
        points.dedup();
        assert_eq!(points_num_before, points.len(), "dedupe detected");

        for rail in rails {
            rail.write_output(&output);
        }

        let bp = output.into_blueprint_string().unwrap();
        assert_eq!(bp, "asd");
    }

    #[test]
    fn turn_chain() {
        let source = HopeSodaLink::new_soda_straight(VPOINT_TEN, FacDirectionQuarter::East);
        let then = source.add_turn90(true);
        let after = then.add_straight_section();
        let sodas = [source, then, after];

        let output = FacItemOutput::new_blueprint();

        let rails = sodas_to_rails(sodas).collect_vec();

        let mut points = rails.iter().map(|v| v.position).collect_vec();
        let points_num_before = points.len();
        points.sort();
        points.dedup();
        // assert_eq!(points_num_before, points.len(), "dedupe detected");

        for rail in rails {
            rail.write_output(&output);
        }

        let bp = output.into_blueprint_string().unwrap();
        assert_eq!(bp, "asd");
    }
}
