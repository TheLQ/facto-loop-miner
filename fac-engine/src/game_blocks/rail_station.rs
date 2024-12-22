use tracing::warn;

use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        chest::{FacEntChest, FacEntChestType},
        direction::{self, FacDirectionQuarter},
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
        rail::RAIL_STRAIGHT_DIAMETER,
        train_stop::FacEntTrainStop,
    },
};

use super::{block::FacBlock, rail_hope::RailHopeAppender, rail_hope_single::RailHopeSingle};

const INSERTERS_PER_CAR: i32 = 6;

pub enum RailStationSide {}

/// Rail onload/offload station
pub struct FacBlkRailStation {
    wagons: usize,
    chests: Option<FacEntChestType>,
    front_engines: usize,
}

impl FacBlock for FacBlkRailStation {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();

        let rotation = false;
        let mut hope = RailHopeSingle::new(origin, FacDirectionQuarter::East);
        hope.add_shift45(rotation, 6);

        let mut station_origin = hope.current_next_pos().move_y(1);
        warn!("origin {:?}", station_origin);

        const RAILS_PER_CART: f32 = 3.5;
        let base_straight: usize =
            (RAILS_PER_CART * (self.wagons + self.front_engines) as f32).ceil() as usize;

        let stop_rail_pos =
            station_origin.move_x_usize((base_straight - 1) * RAIL_STRAIGHT_DIAMETER);
        self.place_train_stop(&mut res, stop_rail_pos);
        self.place_side_inserter_electrics(&mut res, stop_rail_pos);
        self.place_side_inserters(&mut res, stop_rail_pos);
        if let Some(chests) = &self.chests {
            self.place_side_chests(&mut res, stop_rail_pos, chests);
        }

        // self.place_train_stop(&mut res, hope.current_next_pos());
        hope.add_straight(base_straight);

        hope.add_turn90(!rotation);
        hope.add_turn90(!rotation);
        hope.add_straight(base_straight + /*opposite of 45*/13);

        res.extend(hope.to_fac());

        res
    }
}

impl FacBlkRailStation {
    pub fn new(wagons: usize, chests: Option<FacEntChestType>, front_engines: usize) -> Self {
        Self {
            wagons,
            chests,
            front_engines,
        }
    }

    fn place_side_inserters(&self, res: &mut Vec<BlueprintItem>, stop_rail_pos: VPoint) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for (offset, direction) in [
                    (-2, FacDirectionQuarter::South),
                    (1, FacDirectionQuarter::North),
                ] {
                    let start = stop_rail_pos
                        .move_x(/*pre-pole*/ -1 - car_x_offset - exit)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacEntInserter::new(FacEntInserterType::Basic, direction).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_side_inserter_electrics(&self, res: &mut Vec<BlueprintItem>, stop_rail_pos: VPoint) {
        // lamps and poles on start and end
        for car in 0..(self.wagons + 1) {
            let car_x_offset = self.get_wagon_x_offset(car);

            let start = stop_rail_pos.move_x(-car_x_offset);

            res.push(BlueprintItem::new(
                FacEntLamp::new().into_boxed(),
                start.move_y(1),
            ));
            res.push(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                start.move_y(-2),
            ));
        }
    }

    fn place_side_chests(
        &self,
        res: &mut Vec<BlueprintItem>,
        stop_rail_pos: VPoint,
        chest_type: &FacEntChestType,
    ) {
        for car in 0..self.wagons {
            let car_x_offset = self.get_wagon_x_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for offset in [-3, 2] {
                    let start = stop_rail_pos
                        .move_x(/*pre-pole*/ -1 - car_x_offset - exit)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacEntChest::new(chest_type.clone()).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_train_stop(&self, res: &mut Vec<BlueprintItem>, stop_rail_pos: VPoint) {
        res.push(BlueprintItem::new(
            FacEntTrainStop::new(FacDirectionQuarter::East).into_boxed(),
            stop_rail_pos.move_y(1),
        ));
    }

    fn get_wagon_x_offset(&self, wagon: usize) -> i32 {
        let engine_first_offset = 6;
        let engine_rest_offset = (self.front_engines - 1) * 7;
        let wagon_offset = wagon * 7;
        (engine_first_offset + engine_rest_offset + wagon_offset) as i32
    }
}
