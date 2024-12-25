use crate::{
    blueprint::bpitem::BlueprintItem,
    common::vpoint::VPoint,
    game_entities::{
        chest::FacEntChestType, direction::FacDirectionQuarter, inserter::FacEntInserterType,
    },
};

use super::{
    block::FacBlock, rail_hope::RailHopeAppender, rail_hope_dual::RailHopeDual,
    rail_station::FacBlkRailStation,
};

/// Thousands of lines of Rust all to place this.
pub struct FacBlkRailLoop {
    wagons: usize,
    front_engines: usize,
    hope: RailHopeDual,
    bpitems: Vec<BlueprintItem>,
    origin: VPoint,
    // is_start_set: bool,
    // is_end_set: bool,
    chest_type: Option<FacEntChestType>,
    inserter_type: FacEntInserterType,
}

impl FacBlkRailLoop {
    pub fn new(props: FacBlkRailLoopProps) -> Self {
        Self {
            wagons: props.wagons,
            front_engines: props.front_engines,
            chest_type: props.chest_type,
            inserter_type: props.inserter_type,
            origin: props.origin.clone(),
            hope: RailHopeDual::new(props.origin, props.origin_direction),
            bpitems: Vec::new(),
            // is_end_set: false,
        }
    }

    pub fn add_straight(&mut self) {
        self.hope.add_straight_section();
    }

    pub fn add_turn90(&mut self, clockwise: bool) {
        self.hope.add_turn90(clockwise);
    }

    fn add_start(&mut self) {
        let station = FacBlkRailStation {
            wagons: self.wagons,
            front_engines: self.front_engines,
            chests: self.chest_type.clone(),
            inserter: self.inserter_type.clone(),
            is_east: true,
            is_up: true,
        };

        let mut origin = self.origin;
        // hmm...
        // match self.hope.current_direction().rotate_flip() {
        //     FacDirectionQuarter::East => {
        //         origin = origin.move_y(-4);
        //     }
        //     dir => panic!("unsupported dir {}", dir),
        // }

        // RailHope places rail here
        origin = origin.move_x(2);

        self.bpitems.extend(station.generate(origin));
    }

    fn add_end(&mut self) {
        // self.is_end_set = true;
        let station = FacBlkRailStation {
            wagons: self.wagons,
            front_engines: self.front_engines,
            chests: self.chest_type.clone(),
            inserter: self.inserter_type.clone(),
            is_east: true,
            is_up: true,
        };

        let mut origin = self.hope.next_buildable_point();
        match self.hope.current_direction() {
            FacDirectionQuarter::East => {
                origin = origin.move_y(-4);
            }
            dir => panic!("unsupported dir {}", dir),
        }
        self.bpitems.extend(station.generate(origin));
    }

    pub fn to_fac(mut self) -> Vec<BlueprintItem> {
        self.add_start();
        self.add_end();

        let mut res = self.bpitems;
        res.extend(self.hope.to_fac());
        res
    }
}

pub struct FacBlkRailLoopProps {
    pub wagons: usize,
    pub front_engines: usize,
    pub origin: VPoint,
    pub origin_direction: FacDirectionQuarter,
    pub chest_type: Option<FacEntChestType>,
    pub inserter_type: FacEntInserterType,
}
