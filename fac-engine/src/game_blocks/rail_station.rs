use crate::{
    blueprint::{
        bpfac::schedule::FacBpSchedule,
        output::{ContextLevel, FacItemOutput},
    },
    common::vpoint::VPoint,
    game_blocks::belt_train_unload::FacBlkBeltTrainUnload,
    game_entities::{
        belt::FacEntBeltType,
        cargo_wagon::FacEntWagon,
        chest::{FacEntChest, FacEntChestType},
        direction::FacDirectionQuarter,
        electric_large::FacEntElectricLargeType,
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        locomotive::FacEntLocomotive,
        rail_signal::{FacEntRailSignal, FacEntRailSignalType},
        rail_straight::RAIL_STRAIGHT_DIAMETER,
        train_stop::FacEntTrainStop,
    },
};
use std::rc::Rc;

use super::{
    belt_bettel::FacBlkBettelBelt, block::FacBlock2, rail_hope::RailHopeAppender,
    rail_hope_single::RailHopeSingle,
};

const INSERTERS_PER_CAR: usize = 6;

pub enum RailStationSide {}

/// Rail onload/offload station
pub struct FacBlkRailStation {
    pub name: String,
    pub wagons: usize,
    pub front_engines: usize,
    pub delivery: FacExtDelivery,
    pub inserter: FacEntInserterType,
    pub fuel_inserter: Option<FacEntInserterType>,
    pub fuel_inserter_chest: Option<FacEntChestType>,
    pub schedule: Option<FacBpSchedule>,
    pub is_east: bool,
    pub is_up: bool,
    pub is_input: bool,
    pub is_create_train: bool,
    pub is_electric_initial: bool,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock2<Vec<FacBlkBettelBelt>> for FacBlkRailStation {
    fn generate(&self, origin: VPoint) -> Vec<FacBlkBettelBelt> {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, format!("Station-{}", self.name));
        let base_direction;
        let fill_x_direction;
        let origin_after_straight;
        let rotation;
        #[allow(unreachable_code)]
        match (self.is_east, self.is_up) {
            (true, true) => {
                base_direction = FacDirectionQuarter::East;
                fill_x_direction = FacDirectionQuarter::West;
                origin_after_straight = true;
                rotation = false;
            }
            (true, false) => {
                todo!("bad rail insert pos");
                base_direction = FacDirectionQuarter::East;
                fill_x_direction = FacDirectionQuarter::East;
                origin_after_straight = false;
                rotation = true;
            }
            (false, true) => {
                todo!();
                base_direction = FacDirectionQuarter::West;
                fill_x_direction = FacDirectionQuarter::West;
                origin_after_straight = false;
                rotation = true;
            }
            (false, false) => {
                todo!();
                base_direction = FacDirectionQuarter::West;
                fill_x_direction = FacDirectionQuarter::West;
                origin_after_straight = false;
                rotation = true;
            }
        }

        if self.is_electric_initial {
            Self::place_electric_initial(&origin, &base_direction, &self.output);
        }

        let mut hope = RailHopeSingle::new(origin, base_direction, self.output.clone());

        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "TurnUp".into());
            hope.add_shift45(rotation, 6);
        }

        Self::place_electric_connect(&hope.pos_next(), &base_direction, &self.output);

        const RAILS_PER_CART: f32 = 3.5;
        let base_straight: usize =
            (RAILS_PER_CART * (self.wagons + self.front_engines) as f32).ceil() as usize;

        let station_origin = {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "StopRail".into());
            if !origin_after_straight {
                hope.add_straight(base_straight);
            }
            let res = hope.pos_next();
            // warn!("origin {:?}", station_origin);
            if origin_after_straight {
                hope.add_straight(base_straight);
            }
            res
        };

        let stop_rail_pos = station_origin.move_direction_usz(
            fill_x_direction.rotate_flip(),
            (base_straight - 1) * RAIL_STRAIGHT_DIAMETER,
        );

        let stop_block = FacBlkRailStop {
            wagons: self.wagons,
            front_engines: self.front_engines,
            stop_rail_pos,
            fill_x_direction,
            rotation,
            output: self.output.clone(),
        };
        stop_block.place_train_stop(self.name.clone());
        stop_block.place_side_electrics();
        stop_block.place_side_inserters(self.inserter, self.is_input);
        stop_block.place_rail_signals();
        let mut belts = None;
        match &self.delivery {
            FacExtDelivery::Chest(chests) => stop_block.place_side_chests(chests),
            FacExtDelivery::Belt(belt_type) => {
                belts = Some(stop_block.place_belts_output_combined(belt_type));
            }
            FacExtDelivery::None => {}
        }
        stop_block.place_fuel(&self.fuel_inserter, &self.fuel_inserter_chest);

        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "TurnBack-First".into());
            hope.add_turn90(!rotation);
        }
        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "TurnBack-Last".into());
            hope.add_turn90(!rotation);
        }
        {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "StraightBack".into());
            hope.add_straight(base_straight + /*opposite of 45*/13);
        }

        if self.is_create_train {
            // rails beneath must be placed already
            // otherwise it supposedly creates but doesn't show visually at least
            stop_block.place_train(&self.schedule);
        }

        belts.unwrap_or_default()
    }
}

impl FacBlkRailStation {
    fn place_electric_initial(
        origin: &VPoint,
        base_direction: &FacDirectionQuarter,
        output: &Rc<FacItemOutput>,
    ) {
        let _ = &mut output.context_handle(ContextLevel::Micro, "🔚Grid-0".into());
        let electric_start_pos = origin.move_direction_usz(base_direction.rotate_once(), 2);
        output.writei(FacEntElectricLargeType::Big.entity(), electric_start_pos);
    }

    fn place_electric_connect(
        origin: &VPoint,
        base_direction: &FacDirectionQuarter,
        output: &Rc<FacItemOutput>,
    ) {
        let _ = &mut output.context_handle(ContextLevel::Micro, "🔚Grid-1".into());
        let electric_start_pos = origin
            .move_direction_usz(base_direction.rotate_once(), 4)
            .move_direction_usz(base_direction.rotate_once().rotate_once(), 6);
        output.writei(FacEntElectricLargeType::Big.entity(), electric_start_pos);
    }
}

struct FacBlkRailStop {
    wagons: usize,
    front_engines: usize,
    stop_rail_pos: VPoint,
    fill_x_direction: FacDirectionQuarter,
    rotation: bool,
    output: Rc<FacItemOutput>,
}

impl FacBlkRailStop {
    fn place_side_inserters(&self, inserter: FacEntInserterType, is_input: bool) {
        for car in 0..self.wagons {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("🔚Car-{car}-Inserter"));

            let car_x_offset = self.get_wagon_x_offset(car);

            for (negative, direction) in [
                (true, FacDirectionQuarter::South),
                (false, FacDirectionQuarter::North),
            ] {
                for exit in 0..INSERTERS_PER_CAR {
                    let _ = &mut self.output.context_handle(
                        ContextLevel::Micro,
                        if negative { "Bottom" } else { "Top" }.into(),
                    );
                    let direction = if is_input {
                        direction.rotate_flip()
                    } else {
                        direction
                    };
                    let start = self
                        .stop_rail_pos
                        .move_direction_usz(
                            self.fill_x_direction,
                            /*pre-pole*/ 1 + car_x_offset + exit,
                        )
                        .move_y(centered_y_offset(negative, 1));
                    self.output
                        .writei(FacEntInserter::new(inserter, direction), start);
                }
            }
        }
    }

    fn place_side_electrics(&self) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "🔚Electrics".into());
        // lamps and poles on start and end
        for roller in 0..(self.wagons + self.front_engines + 1) {
            let electric_pos = self.get_rolling_point_at_xy(false, roller, -1, 1);

            // output.writei(
            //     FacEntLamp::new().into_boxed(),
            //     start.move_y(centered_y_offset(!self.rotation, 2)),
            // ));
            self.output.writei(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium),
                electric_pos,
            );
        }
    }

    fn place_side_chests(&self, chest_type: &FacEntChestType) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("🔚Car-{car}-Chest"));

            for negative in [true, false] {
                for exit in 0..INSERTERS_PER_CAR {
                    let _ = &mut self.output.context_handle(
                        ContextLevel::Micro,
                        if negative { "Top" } else { "Bottom" }.into(),
                    );
                    let start = self
                        .stop_rail_pos
                        .move_direction_usz(
                            self.fill_x_direction,
                            /*pre-pole*/ 1 + car_x_offset + exit,
                        )
                        .move_y(centered_y_offset(negative, 2));
                    self.output
                        .writei(FacEntChest::new(chest_type.clone()), start);
                }
            }
        }
    }

    fn place_train_stop(&self, station_name: String) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "🔚Stop".into());
        // wtf? Why does this not work? centered_y_offset(self.rotation, 2)
        let y_offset = if self.rotation { -2 } else { 2 };
        self.output.writei(
            FacEntTrainStop::new(self.fill_x_direction.rotate_flip(), station_name),
            self.stop_rail_pos.move_y(y_offset),
        );
    }

    fn place_rail_signals(&self) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "🔚Signals".into());
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            let start = self
                .stop_rail_pos
                .move_direction_usz(
                    self.fill_x_direction,
                    /*pre-pole*/ 1 + car_x_offset + INSERTERS_PER_CAR,
                )
                .move_y(centered_y_offset(self.rotation, 1));
            self.output.writei(
                FacEntRailSignal::new(FacEntRailSignalType::Basic, self.fill_x_direction),
                start,
            );
        }

        self.output.writei(
            FacEntRailSignal::new(FacEntRailSignalType::Basic, self.fill_x_direction),
            self.stop_rail_pos
                .move_direction_usz(self.fill_x_direction.rotate_flip(), 2)
                .move_y(centered_y_offset(self.rotation, 1)),
        );
    }

    fn place_train(&self, schedule: &Option<FacBpSchedule>) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "🔚Stock".into());
        for roller in 0..(self.front_engines + self.wagons) {
            let roller_pos = self.get_rolling_point_at_xy(true, roller + 1, 2, 0);
            if roller < self.front_engines {
                self.output.writei(
                    FacEntLocomotive::new_with_schedule(schedule.clone()),
                    roller_pos,
                );
            } else {
                self.output.writei(FacEntWagon::new(), roller_pos);
            };
        }
    }

    fn place_fuel(
        &self,
        fuel_inserter: &Option<FacEntInserterType>,
        fuel_inserter_chest: &Option<FacEntChestType>,
    ) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "🔚Fuel".into());

        let (fuel_inserter, fuel_inserter_chest) = match (fuel_inserter, fuel_inserter_chest) {
            (Some(fuel_inserter), Some(fuel_inserter_chest)) => {
                (fuel_inserter, fuel_inserter_chest)
            }
            (None, None) => return,
            _ => panic!("imbalance"),
        };

        for roller in 0..self.front_engines {
            let inserter_direction = FacDirectionQuarter::South;
            let inserter_direction = if self.rotation {
                inserter_direction.rotate_flip()
            } else {
                inserter_direction
            };
            self.output.writei(
                FacEntInserter::new(*fuel_inserter, inserter_direction),
                self.get_rolling_point_at_xy(true, roller, 1, 2),
            );
            self.output.writei(
                FacEntChest::new(fuel_inserter_chest.clone()),
                self.get_rolling_point_at_xy(true, roller, 1, 3),
            );
        }
    }

    fn place_belts_output_combined(&self, belt_type: &FacEntBeltType) -> Vec<FacBlkBettelBelt> {
        let bottom = FacBlkBeltTrainUnload {
            belt_type: *belt_type,
            output: self.output.clone(),
            origin_direction: self.fill_x_direction.rotate_opposite(),
            padding_unmerged: 0,
            padding_above: 0,
            padding_after: 1,
            turn_clockwise: self.rotation,
            wagons: self.wagons,
        };
        let mut output_belts =
            bottom.generate(self.get_rolling_point_at_xy(true, self.front_engines, 0, 3));

        let belt_num = output_belts.len();
        for (i, belt) in output_belts.iter_mut().enumerate() {
            let wagon_offset = (i + 1).div_ceil(6);
            belt.add_straight(wagon_offset);
            belt.add_turn90_stacked_row_ccw(i);
            belt.add_straight_underground(4);
            belt.add_turn90_stacked_row_clk(i, belt_num);
            belt.add_straight(self.wagons - wagon_offset);
        }

        let top = FacBlkBeltTrainUnload {
            belt_type: *belt_type,
            output: self.output.clone(),
            origin_direction: self.fill_x_direction.rotate_once(),
            padding_unmerged: 0,
            padding_above: (self.wagons * 3) - 1,
            padding_after: (self.wagons * 4),
            turn_clockwise: !self.rotation,
            wagons: self.wagons,
        };
        let input_belts = top.generate(self.get_rolling_point_at_xy(
            false,
            self.front_engines + self.wagons,
            /*??*/ -2,
            2,
        ));

        [input_belts, output_belts].concat().into_iter().collect()
    }

    #[allow(unused)]
    fn place_belts_output_pointy(&self, belt_type: &FacEntBeltType) {
        let bottom = FacBlkBeltTrainUnload {
            belt_type: *belt_type,
            output: self.output.clone(),
            origin_direction: self.fill_x_direction.rotate_opposite(),
            padding_unmerged: 0,
            padding_above: 0,
            padding_after: 0,
            turn_clockwise: self.rotation,
            wagons: self.wagons,
        };
        bottom.generate(self.get_rolling_point_at_xy(true, self.front_engines, 0, 3));

        let top = FacBlkBeltTrainUnload {
            belt_type: *belt_type,
            output: self.output.clone(),
            origin_direction: self.fill_x_direction.rotate_once(),
            padding_unmerged: 0,
            padding_above: 0,
            padding_after: 0,
            turn_clockwise: !self.rotation,
            wagons: self.wagons,
        };
        top.generate(self.get_rolling_point_at_xy(
            false,
            self.front_engines + self.wagons,
            /*??*/ -2,
            2,
        ));
    }

    fn get_rolling_point_at_xy(
        &self,
        is_inside: bool,
        roller: usize,
        offset_x: i32,
        offset_y: i32,
    ) -> VPoint {
        let neg = if is_inside { -1 } else { 1 };
        self.stop_rail_pos
            .move_direction_int(self.fill_x_direction, (7 * roller as i32) + offset_x)
            .move_direction_int(self.fill_x_direction.rotate_once(), neg * offset_y)
    }

    // fn get_wagon_point_at_xy(&self, roller: usize, offset_x: i32, offset_y: i32) -> VPoint {
    //     // assert!(roller < self.front_engines, )
    //     self.get_rolling_point_at_xy(roller + self.front_engines, offset_x, offset_y)
    // }

    fn get_wagon_x_offset(&self, wagon: usize) -> usize {
        let engine_first_offset = 6;
        let engine_rest_offset = self.front_engines.saturating_sub(1) * 7;
        let wagon_offset = wagon * 7;
        engine_first_offset + engine_rest_offset + wagon_offset
    }
}

/// Factorio entity_pos = center not top-left comes in handy here
fn centered_y_offset(negative: bool, entity_size: usize) -> i32 {
    let neg = if negative { -1.0 } else { 1.0 };

    let center_offset = 0.5 + entity_size as f32;
    (center_offset * neg).floor() as i32 + /*rel_vpoint*/1
}

#[derive(Clone)]
pub enum FacExtDelivery {
    Chest(FacEntChestType),
    Belt(FacEntBeltType),
    None,
}
