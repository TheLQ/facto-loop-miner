use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        electric_pole_small::{ElectricPoleSmallType, FacElectricPoleSmall},
        inserter::{FacInserter, FacInserterType},
        lamp::FacLamp,
    },
};

pub enum RailStationSide {}

pub struct RailStation {
    cars: usize,
}

impl RailStation {
    pub fn new(cars: usize) -> Self {
        Self { cars }
    }

    pub fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res: Vec<BlueprintItem> = Vec::new();

        self.place_side_inserters(&mut res, origin);

        res
    }

    fn place_side_inserters(&self, res: &mut Vec<BlueprintItem>, start_rail_center: VPoint) {
        const INSERTERS_PER_CAR: usize = 6;

        // inserters
        for car in 0..self.cars {
            let car_x_offset = car * (INSERTERS_PER_CAR + /*connection*/1);

            for exit in 0..INSERTERS_PER_CAR {
                for offset in [-1, 1] {
                    let start = start_rail_center
                        .move_x((/*pre-pole*/1 + car_x_offset + exit) as i32)
                        .move_y(offset);
                    res.push(BlueprintItem::new(
                        FacInserter::new(FacInserterType::Basic).into_boxed(),
                        start,
                    ));
                }
            }
        }

        // lamps and poles on start and end
        for car in 0..(self.cars + 1) {
            let car_x_offset = car * (INSERTERS_PER_CAR + /*connection*/1);

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
}
