use crate::{
    blueprint::output::FacItemOutput,
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
    name_prefix: String,
    wagons: usize,
    front_engines: usize,
    hope: RailHopeDual,
    origin: VPoint,
    // is_start_set: bool,
    // is_end_set: bool,
    chest_type: Option<FacEntChestType>,
    inserter_type: FacEntInserterType,
    is_start_input: bool,
}

impl FacBlkRailLoop {
    pub fn new(props: FacBlkRailLoopProps) -> Self {
        Self {
            name_prefix: props.name_prefix,
            wagons: props.wagons,
            front_engines: props.front_engines,
            chest_type: props.chest_type,
            inserter_type: props.inserter_type,
            origin: props.origin,
            hope: RailHopeDual::new(props.origin, props.origin_direction),
            is_start_input: props.is_start_input,
        }
    }

    pub fn add_straight(&mut self) {
        self.hope.add_straight_section();
    }

    pub fn add_turn90(&mut self, clockwise: bool) {
        self.hope.add_turn90(clockwise);
    }

    fn add_start(&mut self, output: &mut FacItemOutput) {
        let is_input = self.is_start_input;
        let station = FacBlkRailStation {
            name: station_input_to_name(is_input, &self.name_prefix),
            wagons: self.wagons,
            front_engines: self.front_engines,
            chests: self.chest_type.clone(),
            inserter: self.inserter_type.clone(),
            is_east: true,
            is_up: true,
            is_input,
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

        station.generate(origin, output)
    }

    fn add_end(&mut self, output: &mut FacItemOutput) {
        // self.is_end_set = true;
        let is_input = !self.is_start_input;
        let station = FacBlkRailStation {
            name: station_input_to_name(is_input, &self.name_prefix),
            wagons: self.wagons,
            front_engines: self.front_engines,
            chests: self.chest_type.clone(),
            inserter: self.inserter_type.clone(),
            is_east: true,
            is_up: true,
            is_input,
        };

        let mut origin = self.hope.next_buildable_point();
        match self.hope.current_direction() {
            FacDirectionQuarter::East => {
                origin = origin.move_y(-4);
            }
            dir => panic!("unsupported dir {}", dir),
        }
        station.generate(origin, output)
    }

    pub fn to_fac(mut self, output: &mut FacItemOutput) {
        self.add_start(output);
        self.add_end(output);

        self.hope.to_fac(output)
    }
}

fn station_input_to_name(is_input: bool, prefix: &str) -> String {
    if is_input {
        format!("{}-Source", prefix)
    } else {
        format!("{}-Drain", prefix)
    }
}

pub struct FacBlkRailLoopProps {
    pub name_prefix: String,
    pub wagons: usize,
    pub front_engines: usize,
    pub origin: VPoint,
    pub origin_direction: FacDirectionQuarter,
    pub chest_type: Option<FacEntChestType>,
    pub inserter_type: FacEntInserterType,
    pub is_start_input: bool,
}
