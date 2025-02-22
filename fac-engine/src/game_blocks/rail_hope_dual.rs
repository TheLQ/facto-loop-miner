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

/// A 4 way intersection is 13 rails wide square.  
// pub const DUAL_RAIL_STEP: usize = STRAIGHT_RAIL_STEP * 2;
// pub const DUAL_RAIL_STEP_I32: i32 = DUAL_RAIL_STEP as i32;
// const STRAIGHT_RAIL_STEP: usize = 13;

/// The dreamed Side-by-side rail generator
pub struct RailHopeDual {
    links: Vec<HopeDualLink>,
    init_link: HopeDualLink,
    output: Rc<FacItemOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HopeDualLink {
    links: [BackingLink; 2],
    // rtype: HopeLinkType,
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
        let next_origin = origin.move_direction_sideways_axis_int(
            origin_direction,
            -((RAIL_STRAIGHT_DIAMETER * 2) as i32),
        );
        let mut hopes = [
            RailHopeSingle::new(origin, origin_direction, output.clone()),
            RailHopeSingle::new(next_origin, origin_direction, output.clone()),
        ];

        match origin_direction {
            FacDirectionQuarter::East | FacDirectionQuarter::North => {}
            FacDirectionQuarter::West | FacDirectionQuarter::South => {
                // maintain order expected by turn90
                hopes.swap(0, 1);
                // maintain
            }
        }
        Self {
            output: output.clone(),
            links: Vec::new(),
            init_link: HopeDualLink {
                links: hopes.map(|v| BackingLink::Single(v.appender_link().clone())),
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

    pub(crate) fn appender_link(&self) -> &HopeDualLink {
        self.links.last().unwrap_or(&self.init_link)
    }

    fn tracking_single_link(&self) -> &HopeLink {
        let dual = self.links.last().unwrap();
        dual.links[0].to_appendable_link()
    }

    pub(crate) fn current_direction(&self) -> &FacDirectionQuarter {
        &self.tracking_single_link().next_direction
    }
}

impl RailHopeAppender for RailHopeDual {
    fn add_straight(&mut self, length: usize) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, format!("👐Dual-{}", 0));
        let new_link = self.appender_link().add_straight(length);
        self.links.push(new_link)
    }

    fn add_straight_section(&mut self) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "👐Ruby".into());
        let new_link = self.appender_link().add_straight_section();
        self.links.push(new_link)
        // self.add_electric_next();
    }

    fn add_turn90(&mut self, clockwise: bool) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "👐Dual-Turn".into());
        let new_link = self.appender_link().add_turn90(clockwise);
        self.links.push(new_link)
        // self.add_electric_next();
    }

    fn add_shift45(&mut self, clockwise: bool, length: usize) {
        let new_link = self.appender_link().add_shift45(clockwise, length);
        self.links.push(new_link)
    }

    fn pos_next(&self) -> VPoint {
        self.links[0].pos_next()
    }
}

impl RailHopeLink for HopeDualLink {
    fn add_straight(&self, length: usize) -> HopeDualLink {
        HopeDualLink {
            links: self
                .dual_appendable_links()
                .map(|v| BackingLink::Single(v.add_straight(length))),
        }
    }

    fn add_straight_section(&self) -> HopeDualLink {
        HopeDualLink {
            links: self
                .dual_appendable_links()
                .map(|v| BackingLink::Single(v.add_straight_section())),
        }
    }

    fn add_turn90(&self, clockwise: bool) -> HopeDualLink {
        let links = self.dual_appendable_links();
        if clockwise {
            HopeDualLink {
                links: [
                    BackingLink::Single(links[0].add_turn90(clockwise)),
                    create_turn_link_from(links[1], clockwise),
                ],
            }
        } else {
            HopeDualLink {
                links: [
                    create_turn_link_from(links[0], clockwise),
                    BackingLink::Single(links[1].add_turn90(clockwise)),
                ],
            }
        }
    }

    fn add_shift45(&self, _clockwise: bool, _length: usize) -> HopeDualLink {
        unimplemented!()
    }

    fn link_type(&self) -> &HopeLinkType {
        match &self.links {
            [BackingLink::MultiTurn([_, link, _]), _]
            | [_, BackingLink::MultiTurn([_, link, _])] => link.link_type(),
            [BackingLink::Single(link), _] => link.link_type(),
        }
    }

    fn pos_start(&self) -> VPoint {
        self.links[0].to_appendable_link().pos_start()
    }

    fn pos_next(&self) -> VPoint {
        self.links[0].to_appendable_link().pos_next()
    }

    fn area(&self) -> Vec<VPoint> {
        // self.links.iter().flat_map(|v| match v {
        //     BackingLink::Straight(link) => [link],
        //     BackingLink::Turn90(links) => (links),
        // })
        // self.links
        let mut res = Vec::new();
        for link in &self.links {
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
            self.links[0].to_appendable_link(),
            self.links[1].to_appendable_link(),
        ]
    }
}

pub fn duals_into_single_vec(links: impl IntoIterator<Item = HopeDualLink>) -> Vec<HopeLink> {
    let mut res = Vec::new();
    for dual in links {
        for single in dual.links {
            match single {
                BackingLink::Single(link) => res.push(link),
                BackingLink::MultiTurn(links) => res.extend(links),
            }
        }
    }
    res
}

fn create_turn_link_from(link: &HopeLink, clockwise: bool) -> BackingLink {
    let first = link.add_straight(2);
    let middle = first.add_turn90(clockwise);
    let last = middle.add_straight(2);
    BackingLink::MultiTurn([first, middle, last])
}

impl Display for HopeDualLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.links {
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
    use crate::game_blocks::rail_hope_dual::RailHopeDual;
    use crate::game_blocks::rail_hope_single::SECTION_POINTS_I32;
    use crate::game_entities::direction::FacDirectionQuarter;
    use crate::game_entities::rail_straight::RAIL_STRAIGHT_DIAMETER_I32;

    #[test]
    fn step_vpoint_straight() {
        let output = FacItemOutput::new_null().into_rc();

        let mut rail = RailHopeDual::new(VPOINT_ZERO, FacDirectionQuarter::East, output);
        assert_eq!(rail.appender_link().pos_start(), VPOINT_ZERO);
        rail.add_straight_section();

        let target_point = VPoint::new(SECTION_POINTS_I32, 0);
        assert_eq!(
            rail.tracking_single_link().pos_start(),
            VPOINT_ZERO,
            "{:?}",
            rail.appender_link()
        );
        assert_eq!(rail.pos_next(), target_point, "{:?}", rail.appender_link());
    }

    #[test]
    fn step_vpoint_turn() {
        let output = FacItemOutput::new_blueprint().into_rc();

        let mut rail = RailHopeDual::new(VPOINT_ZERO, FacDirectionQuarter::East, output.clone());
        rail.add_turn90(true);
        // rail.add_turn90(false);

        assert_eq!(rail.links.len(), 1);
        let link = rail.into_links().remove(0);
        assert_eq!(link.pos_start(), VPOINT_ZERO, "{link}");
        assert_eq!(
            link.pos_next(),
            VPoint::new(SECTION_POINTS_I32, SECTION_POINTS_I32),
            "\n{link}"
        );

        // "bp {}",
        // output.consume_rc().into_blueprint_string().unwrap()
    }

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
