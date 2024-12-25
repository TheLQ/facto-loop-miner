use tracing::warn;

use crate::{
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        chest::{FacEntChest, FacEntChestType},
        direction::FacDirectionQuarter,
        electric_large::{FacEntElectricLarge, FacEntElectricLargeType},
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
        rail::RAIL_STRAIGHT_DIAMETER,
        rail_signal::{FacEntRailSignal, FacEntRailSignalType},
        train_stop::FacEntTrainStop,
    },
};

use super::{block::FacBlock, rail_hope::RailHopeAppender, rail_hope_single::RailHopeSingle};

const INSERTERS_PER_CAR: usize = 6;

pub enum RailStationSide {}

/// Rail onload/offload station
pub struct FacBlkRailStation {
    pub name: String,
    pub wagons: usize,
    pub front_engines: usize,
    pub chests: Option<FacEntChestType>,
    pub inserter: FacEntInserterType,
    pub is_east: bool,
    pub is_up: bool,
    pub is_input: bool,
}

impl FacBlock for FacBlkRailStation {
    fn generate(&self, origin: VPoint, output: &mut FacItemOutput) {
        let output = &mut output.context_handle(format!("Station-{}", self.name));
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
        };

        Self::place_electric_initial(&origin, &base_direction, output);

        let mut hope = RailHopeSingle::new(origin, base_direction.clone());
        hope.add_shift45(rotation, 6);

        Self::place_electric_connect(&hope.current_next_pos(), &base_direction, output);

        const RAILS_PER_CART: f32 = 3.5;
        let base_straight: usize =
            (RAILS_PER_CART * (self.wagons + self.front_engines) as f32).ceil() as usize;

        if !origin_after_straight {
            hope.add_straight(base_straight);
        }
        let station_origin = hope.current_next_pos();
        warn!("origin {:?}", station_origin);
        if origin_after_straight {
            hope.add_straight(base_straight);
        }

        let stop_rail_pos = station_origin.move_direction(
            fill_x_direction.rotate_flip(),
            (base_straight - 1) * RAIL_STRAIGHT_DIAMETER,
        );

        let stop_block = FacBlkRailStop {
            wagons: self.wagons,
            front_engines: self.front_engines,
            stop_rail_pos,
            fill_x_direction,
            rotation,
        };
        stop_block.place_train_stop(output, self.name.clone());
        stop_block.place_side_inserter_electrics(output);
        stop_block.place_side_inserters(&self.inserter, self.is_input, output);
        stop_block.place_rail_signals(output);
        if let Some(chests) = &self.chests {
            stop_block.place_side_chests(output, chests);
        }

        hope.add_turn90(!rotation);
        hope.add_turn90(!rotation);
        hope.add_straight(base_straight + /*opposite of 45*/13);

        hope.to_fac(output);
    }
}

impl FacBlkRailStation {
    fn place_electric_initial(
        origin: &VPoint,
        base_direction: &FacDirectionQuarter,
        ouput: &mut FacItemOutput,
    ) {
        let electric_start_pos = origin.move_direction(base_direction.rotate_once(), 2);
        ouput.write(BlueprintItem::new(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big).into_boxed(),
            electric_start_pos,
        ));
    }

    fn place_electric_connect(
        origin: &VPoint,
        base_direction: &FacDirectionQuarter,
        ouput: &mut FacItemOutput,
    ) {
        let electric_start_pos = origin
            .move_direction(base_direction.rotate_once(), 4)
            .move_direction(base_direction.rotate_once().rotate_once(), 6);
        ouput.write(BlueprintItem::new(
            FacEntElectricLarge::new(FacEntElectricLargeType::Big).into_boxed(),
            electric_start_pos,
        ));
    }
}

struct FacBlkRailStop {
    wagons: usize,
    front_engines: usize,
    stop_rail_pos: VPoint,
    fill_x_direction: FacDirectionQuarter,
    rotation: bool,
}

impl FacBlkRailStop {
    fn place_side_inserters(
        &self,
        inserter: &FacEntInserterType,
        is_input: bool,
        output: &mut FacItemOutput,
    ) {
        let output = &mut output.subcontext_handle("Inserters".into());
        for car in 0..self.wagons {
            let output = &mut output.subcontext_handle(format!("Car{}", car));
            let car_x_offset = self.get_wagon_x_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for (negative, direction) in [
                    (true, FacDirectionQuarter::South),
                    (false, FacDirectionQuarter::North),
                ] {
                    let output = &mut output
                        .subcontext_handle(if negative { "Bottom" } else { "Top" }.into());
                    let direction = if is_input {
                        direction.rotate_flip()
                    } else {
                        direction
                    };
                    let start = self
                        .stop_rail_pos
                        .move_direction(
                            &self.fill_x_direction,
                            /*pre-pole*/ 1 + car_x_offset + exit,
                        )
                        .move_y(centered_y_offset(negative, 1));
                    output.write(BlueprintItem::new(
                        FacEntInserter::new(inserter.clone(), direction).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_side_inserter_electrics(&self, output: &mut FacItemOutput) {
        // lamps and poles on start and end
        for car in 0..(self.wagons + 1) {
            let car_x_offset = self.get_wagon_x_offset(car);

            let start = self
                .stop_rail_pos
                .move_direction(&self.fill_x_direction, car_x_offset);

            output.write(BlueprintItem::new(
                FacEntLamp::new().into_boxed(),
                start.move_y(centered_y_offset(!self.rotation, 2)),
            ));
            output.write(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                start.move_y(centered_y_offset(!self.rotation, 1)),
            ));
        }
    }

    fn place_side_chests(&self, output: &mut FacItemOutput, chest_type: &FacEntChestType) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for negative in [true, false] {
                    let start = self
                        .stop_rail_pos
                        .move_direction(
                            &self.fill_x_direction,
                            /*pre-pole*/ 1 + car_x_offset + exit,
                        )
                        .move_y(centered_y_offset(negative, 2));
                    output.write(BlueprintItem::new(
                        FacEntChest::new(chest_type.clone()).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_train_stop(&self, output: &mut FacItemOutput, station_name: String) {
        // wtf? Why does this not work? centered_y_offset(self.rotation, 2)
        let y_offset = if self.rotation { -2 } else { 2 };
        output.write(BlueprintItem::new(
            FacEntTrainStop::new(self.fill_x_direction.rotate_flip(), station_name).into_boxed(),
            self.stop_rail_pos.move_y(y_offset),
        ));
    }

    fn place_rail_signals(&self, output: &mut FacItemOutput) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            let start = self
                .stop_rail_pos
                .move_direction(
                    &self.fill_x_direction,
                    /*pre-pole*/ 1 + car_x_offset + INSERTERS_PER_CAR,
                )
                .move_y(centered_y_offset(self.rotation, 1));
            output.write(BlueprintItem::new(
                FacEntRailSignal::new(FacEntRailSignalType::Basic, self.fill_x_direction.clone())
                    .into_boxed(),
                start,
            ));
        }

        output.write(BlueprintItem::new(
            FacEntRailSignal::new(FacEntRailSignalType::Basic, self.fill_x_direction.clone())
                .into_boxed(),
            self.stop_rail_pos
                .move_direction(self.fill_x_direction.rotate_flip(), 2)
                .move_y(centered_y_offset(self.rotation, 1)),
        ));
    }

    fn get_wagon_x_offset(&self, wagon: usize) -> usize {
        let engine_first_offset = 6;
        let engine_rest_offset = (self.front_engines - 1) * 7;
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
