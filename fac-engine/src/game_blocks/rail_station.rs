use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        chest::{FacChest, FacChestType},
        direction::FacDirectionEighth,
        electric_pole_small::{ElectricPoleSmallType, FacElectricPoleSmall},
        inserter::{FacInserter, FacInserterType},
        lamp::FacLamp,
    },
};

const INSERTERS_PER_CAR: usize = 6;

pub enum RailStationSide {}

pub struct RailStation {
    cars: usize,
    chests: Option<FacChestType>,
}

impl RailStation {
    pub fn new(cars: usize, chests: Option<FacChestType>) -> Self {
        Self { cars, chests }
    }

    pub fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res: Vec<BlueprintItem> = Vec::new();

        self.place_side_inserters(&mut res, origin);
        self.place_side_inserter_electrics(&mut res, origin);
        if let Some(chests) = &self.chests {
            self.place_side_chests(&mut res, origin, chests);
        }

        res
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
                        FacInserter::new(FacInserterType::Basic, FacDirectionEighth::East)
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
                FacLamp::new().into_boxed(),
                start.move_y(1),
            ));
            res.push(BlueprintItem::new(
                FacElectricPoleSmall::new(ElectricPoleSmallType::Steel).into_boxed(),
                start.move_y(-1),
            ));
        }
    }

    fn place_side_chests(
        &self,
        res: &mut Vec<BlueprintItem>,
        start_rail_center: VPoint,
        chest_type: &FacChestType,
    ) {
        for car in 0..self.cars {
            let car_x_offset = get_car_offset(car);

            for exit in 0..INSERTERS_PER_CAR {
                for offset in [-2, 2] {
                    let start = start_rail_center
                        .move_x((/*pre-pole*/1 + car_x_offset + exit) as i32)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacChest::new(chest_type.clone()).into_boxed(),
                        start,
                    ));
                }
            }
        }
    }
}

fn get_car_offset(car: usize) -> usize {
    return car * (INSERTERS_PER_CAR + /*connection*/1);
}
