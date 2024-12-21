use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        chest::{FacEntChest, FacEntChestType},
        direction::FacDirectionQuarter,
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
        train_stop::FacEntTrainStop,
    },
};

use super::block::FacBlock;

const INSERTERS_PER_CAR: usize = 6;

pub enum RailStationSide {}

/// Rail onload/offload station
pub struct FacBlkRailStation {
    cars: usize,
    chests: Option<FacEntChestType>,
    front_engines: usize,
}

impl FacBlock for FacBlkRailStation {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        self.place_side_inserters(&mut res, origin);
        self.place_side_inserter_electrics(&mut res, origin);
        if let Some(chests) = &self.chests {
            self.place_side_chests(&mut res, origin, chests);
        }
        self.place_train_stop(&mut res, origin);
        res
    }
}

impl FacBlkRailStation {
    pub fn new(cars: usize, chests: Option<FacEntChestType>, front_engines: usize) -> Self {
        Self {
            cars,
            chests,
            front_engines,
        }
    }

    fn place_side_inserters(&self, res: &mut Vec<BlueprintItem>, start_rail_center: VPoint) {
        for car in 0..self.cars {
            let car_x_offset = get_car_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for offset in [-1, 1] {
                    let start = start_rail_center
                        .move_x((/*pre-pole*/1 + car_x_offset + exit) as i32)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacEntInserter::new(FacEntInserterType::Basic, FacDirectionQuarter::East)
                            .into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_side_inserter_electrics(
        &self,
        res: &mut Vec<BlueprintItem>,
        start_rail_center: VPoint,
    ) {
        // lamps and poles on start and end
        for car in 0..(self.cars + 1) {
            let car_x_offset = get_car_offset(car);

            let start = start_rail_center.move_x((car_x_offset) as i32);

            res.push(BlueprintItem::new(
                FacEntLamp::new().into_boxed(),
                start.move_y(1),
            ));
            res.push(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                start.move_y(-1),
            ));
        }
    }

    fn place_side_chests(
        &self,
        res: &mut Vec<BlueprintItem>,
        start_rail_center: VPoint,
        chest_type: &FacEntChestType,
    ) {
        for car in 0..self.cars {
            let car_x_offset = get_car_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for offset in [-2, 2] {
                    let start = start_rail_center
                        .move_x((/*pre-pole*/1 + car_x_offset + exit) as i32)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacEntChest::new(chest_type.clone()).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }

    fn place_train_stop(&self, res: &mut Vec<BlueprintItem>, start_rail_center: VPoint) {
        let car_x_offset = get_car_offset(self.cars + self.front_engines + /*after engines*/1);
        let start = start_rail_center.move_x(car_x_offset as i32).move_y(1);
        res.push(BlueprintItem::new(
            FacEntTrainStop::new(FacDirectionQuarter::East).into_boxed(),
            start,
        ))
    }
}

fn get_car_offset(car: usize) -> usize {
    car * (INSERTERS_PER_CAR + /*connection*/1)
}
