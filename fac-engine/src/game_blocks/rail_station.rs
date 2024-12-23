use tracing::warn;

use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{
        entity::FacEntity,
        vpoint::{VPOINT_ONE, VPoint},
    },
    game_entities::{
        chest::{FacEntChest, FacEntChestType},
        direction::FacDirectionQuarter,
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
        rail::RAIL_STRAIGHT_DIAMETER,
        train_stop::FacEntTrainStop,
    },
};

use super::{block::FacBlock, rail_hope::RailHopeAppender, rail_hope_single::RailHopeSingle};

const INSERTERS_PER_CAR: usize = 6;

pub enum RailStationSide {}

/// Rail onload/offload station
pub struct FacBlkRailStation {
    pub wagons: usize,
    pub front_engines: usize,
    pub chests: Option<FacEntChestType>,
    pub is_east: bool,
    pub is_up: bool,
}

impl FacBlock for FacBlkRailStation {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();

        let base_direction;
        let fill_x_direction;
        let origin_after_straight;
        let rotation;
        match (self.is_east, self.is_up) {
            (true, true) => {
                base_direction = FacDirectionQuarter::East;
                fill_x_direction = FacDirectionQuarter::West;
                origin_after_straight = true;
                rotation = false;
            }
            (true, false) => {
                base_direction = FacDirectionQuarter::East;
                fill_x_direction = FacDirectionQuarter::East;
                origin_after_straight = false;
                rotation = true;
            }
            // (false, true) => (FacDirectionQuarter::West, true),
            _ => panic!("todo"),
        };

        let mut hope = RailHopeSingle::new(origin, base_direction);
        hope.add_shift45(rotation, 6);

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
        stop_block.place_train_stop(&mut res);
        stop_block.place_side_inserter_electrics(&mut res);
        stop_block.place_side_inserters(&mut res);
        if let Some(chests) = &self.chests {
            stop_block.place_side_chests(&mut res, chests);
        }

        hope.add_turn90(!rotation);
        hope.add_turn90(!rotation);
        hope.add_straight(base_straight + /*opposite of 45*/13);

        res.extend(hope.to_fac());

        res
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
    fn place_side_inserters(&self, res: &mut Vec<BlueprintItem>) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for (negative, direction) in [
                    (true, FacDirectionQuarter::South),
                    (false, FacDirectionQuarter::North),
                ] {
                    let start = self
                        .stop_rail_pos
                        .move_direction(
                            &self.fill_x_direction,
                            /*pre-pole*/ 1 + car_x_offset + exit,
                        )
                        .move_y(centered_y_offset(negative, 1));
                    res.push(BlueprintItem::new(
                        FacEntInserter::new(FacEntInserterType::Basic, direction).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_side_inserter_electrics(&self, res: &mut Vec<BlueprintItem>) {
        // lamps and poles on start and end
        for car in 0..(self.wagons + 1) {
            let car_x_offset = self.get_wagon_x_offset(car);

            let start = self
                .stop_rail_pos
                .move_direction(&self.fill_x_direction, car_x_offset);

            res.push(BlueprintItem::new(
                FacEntLamp::new().into_boxed(),
                start.move_y(centered_y_offset(!self.rotation, 2)),
            ));
            res.push(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                start.move_y(centered_y_offset(!self.rotation, 1)),
            ));
        }
    }

    fn place_side_chests(&self, res: &mut Vec<BlueprintItem>, chest_type: &FacEntChestType) {
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
                    res.push(BlueprintItem::new(
                        FacEntChest::new(chest_type.clone()).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_train_stop(&self, res: &mut Vec<BlueprintItem>) {
        // wtf? Why does this not work? centered_y_offset(self.rotation, 2)
        let y_offset = if self.rotation { -2 } else { 2 };
        res.push(BlueprintItem::new(
            FacEntTrainStop::new(self.fill_x_direction.rotate_flip()).into_boxed(),
            self.stop_rail_pos.move_y(y_offset),
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
