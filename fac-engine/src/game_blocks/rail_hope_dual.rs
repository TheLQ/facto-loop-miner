use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::common::vpoint::{VPOINT_ONE, VPoint};
use crate::game_blocks::rail_hope::{RailHopeAppender, RailHopeLink};
use crate::game_blocks::rail_hope_single::{HopeLink, HopeLinkType, RailHopeSingle};
use crate::game_entities::direction::FacDirectionQuarter;
use crate::game_entities::electric_large::{FacEntElectricLarge, FacEntElectricLargeType};
use crate::game_entities::lamp::FacEntLamp;
use crate::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// The dreamed Side-by-side rail generator
pub struct RailHopeDual {
    links: Vec<HopeDualLink>,
    init_link: HopeDualLink,
    output: Rc<FacItemOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HopeDualLink {
    singles: [BackingLink; 2],
    start: VPoint,
    end: VPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum BackingLink {
    Single(HopeLink),
    MultiTurn([HopeLink; 3]),
}

impl RailHopeDual {
    pub fn new(
        origin: VPoint,
        origin_direction: FacDirectionQuarter,
        output: Rc<FacItemOutput>,
    ) -> Self {
        // move on axis, not rotation, to give every direction the same starting point
        // and maintain intersection
        let top_origin = origin
            .move_direction_sideways_axis_int(origin_direction, RAIL_STRAIGHT_DIAMETER as i32);
        let bottom_origin = origin
            .move_direction_sideways_axis_int(origin_direction, -(RAIL_STRAIGHT_DIAMETER as i32));

        // Generate links with Rail Singles
        let mut hopes = [
            RailHopeSingle::new(top_origin, origin_direction, output.clone()),
            RailHopeSingle::new(bottom_origin, origin_direction, output.clone()),
        ];
        match origin_direction {
            FacDirectionQuarter::East | FacDirectionQuarter::North => {}
            FacDirectionQuarter::West | FacDirectionQuarter::South => {
                // maintain order expected by turn90
                hopes.swap(0, 1);
                // maintain
            }
        }
        let singles = hopes.map(|v| BackingLink::Single(v.appender_link().clone()));

        Self {
            output: output.clone(),
            links: Vec::new(),
            init_link: HopeDualLink {
                singles,
                start: origin,
                end: origin,
            },
        }
    }

    pub fn add_electric_next(&mut self) {
        // let last_link = self.hopes[0].appender_link();
        // self.add_electric_next_for_link(
        //     last_link.next_direction,
        //     last_link.next_straight_position(),
        // );
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

    pub fn into_links(self) -> Vec<HopeDualLink> {
        self.links
    }

    fn appender_link(&self) -> &HopeDualLink {
        self.links.last().unwrap_or(&self.init_link)
    }

    fn tracking_single_link(&self) -> &HopeLink {
        let dual = self.links.last().unwrap();
        dual.singles[0].to_appendable_link()
    }

    pub(super) fn current_direction(&self) -> &FacDirectionQuarter {
        &self.tracking_single_link().next_direction
    }

    fn push_link(&mut self, new_link: HopeDualLink) {
        new_link.write_output(&self.output);
        self.links.push(new_link)
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, format!("👐Dual-{}", 0));
        let new_link = self.appender_link().add_straight(length);
        self.push_link(new_link);
    }

    fn add_straight_section(&mut self) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "👐DualSection".into());
        let new_link = self.appender_link().add_straight_section();
        self.push_link(new_link);
        // self.add_electric_next();
    }

    fn add_turn90(&mut self, clockwise: bool) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "👐Dual-Turn".into());
        let new_link = self.appender_link().add_turn90(clockwise);
        self.push_link(new_link);
        // self.add_electric_next();
    }

    fn add_shift45(&mut self, clockwise: bool, length: usize) {
        let new_link = self.appender_link().add_shift45(clockwise, length);
        self.links.push(new_link)
    }

    fn pos_next(&self) -> VPoint {
        self.links.last().unwrap().pos_next()
    }
}

impl RailHopeLink for HopeDualLink {
    fn add_straight(&self, length: usize) -> HopeDualLink {
        let singles = self
            .dual_appendable_links()
            .map(|v| BackingLink::Single(v.add_straight(length)));

        let start = self.end;
        let end = singles[0]
            .to_appendable_link()
            .pos_next()
            .midpoint(singles[1].to_appendable_link().pos_next());
        HopeDualLink {
            singles,
            start,
            end,
        }
    }

    fn add_straight_section(&self) -> HopeDualLink {
        let singles = self
            .dual_appendable_links()
            .map(|v| BackingLink::Single(v.add_straight_section()));

        let start = self.end;
        let end = singles[0]
            .to_appendable_link()
            .pos_next()
            .midpoint(singles[1].to_appendable_link().pos_next());
        HopeDualLink {
            singles,
            start,
            end,
        }
    }

    fn add_turn90(&self, clockwise: bool) -> HopeDualLink {
        let links = self.dual_appendable_links();
        if clockwise {
            let turn_links = create_turn_link_from(links[1], clockwise, false);
            let last_link = &turn_links[2];
            let end = last_link
                .pos_next()
                .move_direction_sideways_axis_int(last_link.next_direction, 0);
            // match last_link.link_type() {
            //     HopeLinkType::Straight { .. } => {
            //         println!("is straight fish")
            //     }
            //     other => panic!("not straight {other}"),
            // }
            HopeDualLink {
                singles: [
                    BackingLink::MultiTurn(create_turn_link_from(links[0], clockwise, true)),
                    BackingLink::MultiTurn(turn_links),
                ],
                start: self.pos_next(),
                end,
            }
        } else {
            let turn_links = create_turn_link_from(links[0], clockwise, false);
            let last_link = &turn_links[2];
            // match last_link.link_type() {
            //     HopeLinkType::Straight { .. } => {
            //         println!("is straight chicken")
            //     }
            //     other => panic!("not straight {other}"),
            // }
            let end = last_link
                .pos_next()
                .move_direction_sideways_axis_int(last_link.next_direction, 0);
            HopeDualLink {
                singles: [
                    BackingLink::MultiTurn(turn_links),
                    BackingLink::MultiTurn(create_turn_link_from(links[1], clockwise, true)),
                ],
                start: self.pos_next(),
                end,
            }
        }
    }

    fn add_shift45(&self, _clockwise: bool, _length: usize) -> HopeDualLink {
        unimplemented!()
    }

    fn link_type(&self) -> &HopeLinkType {
        match &self.singles {
            [BackingLink::MultiTurn([_, link, _]), _]
            | [_, BackingLink::MultiTurn([_, link, _])] => link.link_type(),
            [BackingLink::Single(link), _] => link.link_type(),
        }
    }

    fn pos_start(&self) -> VPoint {
        self.start
    }

    fn pos_next(&self) -> VPoint {
        self.end
    }

    fn area(&self) -> Vec<VPoint> {
        // self.links.iter().flat_map(|v| match v {
        //     BackingLink::Straight(link) => [link],
        //     BackingLink::Turn90(links) => (links),
        // })
        // self.links
        let mut res = Vec::new();
        for link in &self.singles {
            match link {
                BackingLink::Single(link) => res.extend(link.area()),
                BackingLink::MultiTurn(links) => {
                    for sub in links {
                        res.extend(sub.area())
                    }
                }
            }
        }
        res
    }
}

impl BackingLink {
    fn to_appendable_link(&self) -> &HopeLink {
        match &self {
            BackingLink::Single(link) => link,
            BackingLink::MultiTurn([_, _, link]) => link,
        }
    }
}

impl HopeDualLink {
    fn dual_appendable_links(&self) -> [&HopeLink; 2] {
        [
            self.singles[0].to_appendable_link(),
            self.singles[1].to_appendable_link(),
        ]
    }

    fn write_output(&self, output: &FacItemOutput) {
        for link in &self.singles {
            match link {
                BackingLink::Single(link) => link.write_output(output),
                BackingLink::MultiTurn(links) => {
                    for sub in links {
                        sub.write_output(output)
                    }
                }
            }
        }
    }
}

pub fn duals_into_single_vec(links: impl IntoIterator<Item = HopeDualLink>) -> Vec<HopeLink> {
    let mut res = Vec::new();
    for dual in links {
        for single in dual.singles {
            match single {
                BackingLink::Single(link) => res.push(link),
                BackingLink::MultiTurn(links) => res.extend(links),
            }
        }
    }
    res
}

fn create_turn_link_from(link: &HopeLink, clockwise: bool, is_single: bool) -> [HopeLink; 3] {
    let padding = if is_single { 1 } else { 3 };
    let first = link.add_straight(padding);
    let middle = first.add_turn90(clockwise);
    let last = middle.add_straight(padding);
    [first, middle, last]
}

impl Display for HopeDualLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.singles {
            [
                BackingLink::MultiTurn([outer_start, outer_turn, outer_end]),
                BackingLink::Single(inner_turn),
            ]
            | [
                BackingLink::Single(inner_turn),
                BackingLink::MultiTurn([outer_start, outer_turn, outer_end]),
            ] => {
                write!(
                    f,
                    "Inner-   {inner_turn}\nOuterSta-{outer_start}\nOuterTur-{outer_turn}\nOuterEnd-{outer_end}"
                )
            }
            [BackingLink::Single(inner), BackingLink::Single(outer)] => {
                write!(f, "Inner-{inner}\nOuter-{outer}")
            }
            [BackingLink::MultiTurn(_), BackingLink::MultiTurn(_)] => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::blueprint::bpitem::BlueprintItem;
    use crate::blueprint::output::FacItemOutput;
    use crate::common::vpoint::{VPOINT_ZERO, VPoint};
    use crate::common::vpoint_direction::VPointDirectionQ;
    use crate::game_blocks::rail_hope::{RailHopeAppender, RailHopeLink};
    use crate::game_blocks::rail_hope_dual::{HopeDualLink, RailHopeDual};
    use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use crate::game_entities::direction::FacDirectionQuarter;
    use crate::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;
    use std::vec::IntoIter;

    fn do_simple_test<F>(
        source_direction: FacDirectionQuarter,
        mut actions: F,
    ) -> (IntoIter<HopeDualLink>, String)
    where
        F: FnMut(&mut RailHopeDual),
    {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut rail = RailHopeDual::new(VPOINT_ZERO, source_direction, output.clone());
        actions(&mut rail);
        output.flush();

        let links = rail.links.clone().into_iter();
        drop(rail);
        let output_bp = output.consume_rc().into_blueprint_string().unwrap();

        (links, output_bp)
    }

    //////////////////////////

    #[test]
    fn step_east_straight() {
        let target_point = VPoint::new(SECTION_POINTS_I32, 0);

        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            assert_eq!(rail.appender_link().pos_start(), VPOINT_ZERO);
            rail.add_straight_section();
            assert_eq!(rail.pos_next(), target_point, "{}", rail.appender_link());
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "bp {output_bp}\n{link}",);
        assert_eq!(link.pos_next(), target_point, "bp {output_bp}\n{link}",);

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_east_turn_clw() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            rail.add_turn90(true); //
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "bp {output_bp}\n{link}",);
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_east_turn_cww() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            rail.add_turn90(false); //
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, -SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    //////////////////////////

    #[test]
    fn step_south_straight() {
        let target_point = VPoint::new(0, SECTION_POINTS_I32);

        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::South, |rail| {
            assert_eq!(rail.appender_link().pos_start(), VPOINT_ZERO);
            rail.add_straight_section();
            assert_eq!(rail.pos_next(), target_point, "{}", rail.appender_link());
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "bp {output_bp}\n{link}",);
        assert_eq!(link.pos_next(), target_point, "bp {output_bp}\n{link}",);

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_south_turn_clw() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::South, |rail| {
            rail.add_turn90(true); //
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "bp {output_bp}\n{link}",);
        assert_eq!(
            link.pos_next(),
            VPoint::new(-SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_south_turn_cww() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::South, |rail| {
            rail.add_turn90(false); //
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    //////////////////////////

    #[test]
    fn step_east_straight_then_turn_clw_then_straight() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            rail.add_straight_section();
            rail.add_turn90(true);
            rail.add_straight_section();
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, 0),
            "bp {output_bp}\n{link}",
        );

        let link = links.next().unwrap();
        assert_eq!(
            link.pos_start(),
            VPoint::new(SECTION_POINTS_I32, 0),
            "bp {output_bp}\n{link}",
        );
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32 * 2, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        let link = links.next().unwrap();
        assert_eq!(
            link.pos_start(),
            VPoint::new(SECTION_POINTS_I32 * 2, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32 * 2, SECTION_POINTS_I32 * 2),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_south_straight_then_turn_clw() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::South, |rail| {
            rail.add_straight_section();
            rail.add_turn90(true);
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(0, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        let link = links.next().unwrap();
        assert_eq!(
            link.pos_start(),
            VPoint::new(0, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );
        assert_eq!(
            link.pos_next(),
            VPoint::new(-SECTION_POINTS_I32, SECTION_POINTS_I32 * 2),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_turn_clw_then_clw_again() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            rail.add_turn90(true);
            rail.add_turn90(true);
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        let link = links.next().unwrap();
        assert_eq!(
            link.pos_start(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );
        assert_eq!(
            link.pos_next(),
            VPoint::new(0, SECTION_POINTS_I32 * 2),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    #[test]
    fn step_turn_clw_then_ccw() {
        let (mut links, output_bp) = do_simple_test(FacDirectionQuarter::East, |rail| {
            rail.add_turn90(true);
            rail.add_turn90(false);
        });

        let link = links.next().unwrap();
        assert_eq!(link.pos_start(), VPOINT_ZERO, "\n{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );

        let link = links.next().unwrap();
        assert_eq!(
            link.pos_start(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "bp {output_bp}\n{link}",
        );
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32 * 2, SECTION_POINTS_I32 * 2),
            "bp {output_bp}\n{link}",
        );

        assert_eq!(links.next(), None);
    }

    //////////////////////////

    #[test]
    fn congruent_line() {
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
            rail.add_straight_section();
            rail.add_turn90(true);
            rail.add_straight_section();
        });
        a.sort();

        let mut b = dual_gen(
            (
                VPOINT_ZERO.move_y(SECTION_POINTS_I32),
                FacDirectionQuarter::East,
            ),
            |rail| {
                rail.add_straight_section();
                rail.add_turn90(false);
                rail.add_straight_section();
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
