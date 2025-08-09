use std::rc::Rc;

use crate::{
    blueprint::{
        bpfac::{
            infinity::{FacBpFilter, FacBpInfinitySettings},
            schedule::{
                FacBpCircuitCondition, FacBpLogic, FacBpSchedule, FacBpScheduleData,
                FacBpScheduleWait, FacBpWaitType,
            },
            signal_id::{FacBpSignalId, FacBpSignalIdType},
        },
        output::{ContextLevel, FacItemOutput},
    },
    common::vpoint::VPoint,
    game_entities::{
        chest::FacEntChestType, direction::FacDirectionQuarter, inserter::FacEntInserterType,
    },
};

use super::{
    block::FacBlock2,
    rail_hope::RailHopeAppender,
    rail_hope_dual::RailHopeDual,
    rail_station::{FacBlkRailStation, FacExtDelivery},
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
    delivery_input: FacExtDelivery,
    delivery_output: FacExtDelivery,
    inserter_type: FacEntInserterType,
    is_start_input: bool,
    output: Rc<FacItemOutput>,
}

impl FacBlkRailLoop {
    pub fn new(
        FacBlkRailLoopProps {
            name_prefix,
            wagons,
            front_engines,
            delivery_input,
            delivery_output,
            inserter_type,
            origin,
            origin_direction,
            is_start_input,
            output,
        }: FacBlkRailLoopProps,
    ) -> Self {
        let mut new = Self {
            hope: RailHopeDual::new(origin, origin_direction, output.clone()),
            name_prefix,
            wagons,
            front_engines,
            delivery_input,
            delivery_output,
            inserter_type,
            origin,
            is_start_input,
            output,
        };
        new.hope
            .add_electric_next_for_link(origin_direction, origin);
        new
    }

    pub fn add_straight(&mut self) {
        let _ = &mut self.output.context_handle(
            ContextLevel::Block,
            format!("Loop-{}-Section", self.name_prefix),
        );
        self.hope.add_straight_section();
    }

    pub fn add_turn90(&mut self, clockwise: bool) {
        let _ = &mut self.output.context_handle(
            ContextLevel::Block,
            format!("Loop-{}-Turn90", self.name_prefix),
        );
        self.hope.add_turn90(clockwise);
    }

    fn add_start(&mut self) {
        let is_input = self.is_start_input;

        let station = FacBlkRailStation {
            name: self.station_input_to_name(is_input),
            wagons: self.wagons,
            front_engines: self.front_engines,
            delivery: if is_input {
                self.delivery_input.clone()
            } else {
                self.delivery_output.clone()
            },
            inserter: self.inserter_type,
            fuel_inserter: Some(FacEntInserterType::Basic),
            fuel_inserter_chest: Some(FacEntChestType::Infinity(FacBpInfinitySettings {
                filters: [FacBpFilter::new_for_item("nuclear-fuel")].to_vec(),
                remove_unfiltered_items: true,
            })),
            is_east: true,
            is_up: true,
            is_input,
            is_electric_initial: false,
            place_train: Some(Some(self.new_schedule())),
            output: self.output.clone(),
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

        station.generate(origin);
    }

    fn add_end(&mut self) {
        // self.is_end_set = true;
        let is_input = !self.is_start_input;
        let station = FacBlkRailStation {
            name: self.station_input_to_name(is_input),
            wagons: self.wagons,
            front_engines: self.front_engines,
            delivery: if is_input {
                self.delivery_input.clone()
            } else {
                self.delivery_output.clone()
            },
            inserter: self.inserter_type,
            fuel_inserter: None,
            fuel_inserter_chest: None,
            place_train: None,
            is_east: true,
            is_up: true,
            is_input,
            is_electric_initial: false,
            output: self.output.clone(),
        };

        let mut origin = self.hope.pos_next();
        match self.hope.current_direction() {
            FacDirectionQuarter::East => {
                origin = origin.move_y(-4);
            }
            dir => panic!("unsupported dir {dir}"),
        }
        station.generate(origin);
    }

    pub fn add_base_start_and_end(&mut self) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, format!("Loop-{}", self.name_prefix));
        self.add_start();
        self.add_end();
    }

    fn new_schedule(&self) -> FacBpSchedule {
        FacBpSchedule {
            locomotives: Vec::new(),
            schdata: [
                FacBpScheduleData {
                    station: self.station_input_to_name(true),
                    wait_conditions: [
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::ItemCount,
                            condition: Some(FacBpCircuitCondition {
                                comparator: Some(">".into()),
                                first_signal: Some(FacBpSignalId {
                                    stype: FacBpSignalIdType::Item,
                                    // TODO
                                    name: "heavy-oil-barrel".into(),
                                }),
                                second_signal: None,
                                constant: Some(800),
                            }),
                        },
                        FacBpScheduleWait {
                            compare_type: FacBpLogic::Or,
                            ctype: FacBpWaitType::Full,
                            condition: None,
                        },
                    ]
                    .into(),
                },
                FacBpScheduleData {
                    station: self.station_input_to_name(false),
                    wait_conditions: [FacBpScheduleWait {
                        compare_type: FacBpLogic::Or,
                        ctype: FacBpWaitType::Empty,
                        condition: None,
                    }]
                    .into(),
                },
            ]
            .into(),
        }
    }

    fn station_input_to_name(&self, is_input: bool) -> String {
        if is_input {
            format!("{}-Source", self.name_prefix)
        } else {
            format!("{}-Drain", self.name_prefix)
        }
    }
}

pub struct FacBlkRailLoopProps {
    pub name_prefix: String,
    pub wagons: usize,
    pub front_engines: usize,
    pub origin: VPoint,
    pub origin_direction: FacDirectionQuarter,
    pub delivery_input: FacExtDelivery,
    pub delivery_output: FacExtDelivery,
    pub inserter_type: FacEntInserterType,
    pub is_start_input: bool,
    pub output: Rc<FacItemOutput>,
}
