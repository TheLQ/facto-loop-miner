use std::rc::Rc;

use crate::{
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        belt::FacEntBeltType,
        direction::FacDirectionQuarter,
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        mining_drill_electric::{ELECTRIC_DRILL_SIZE, FacEntMiningDrillElectric},
        module::FacModule,
    },
};

use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock};

pub struct FacBlkMineOre {
    pub width: usize,
    pub height: usize,
    pub build_direction: FacDirectionQuarter,
    pub belt: FacEntBeltType,
    pub drill_modules: [Option<FacModule>; 3],
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkMineOre {
    fn generate(&self, origin: VPoint) {
        for height in 0..self.height {
            let offset_y = height * 7;
            self.place_drill_single_row(
                origin.move_direction_sideways_usz(self.build_direction, offset_y),
                self.width,
                self.build_direction,
                self.build_direction.rotate_once(),
            );

            self.place_drill_single_row(
                origin.move_direction_sideways_usz(
                    self.build_direction,
                    offset_y + ELECTRIC_DRILL_SIZE + /*belt*/1,
                ),
                self.width,
                self.build_direction,
                self.build_direction.rotate_once().rotate_flip(),
            );

            self.place_inner_belts(
                origin.move_direction_sideways_usz(
                    self.build_direction,
                    offset_y + ELECTRIC_DRILL_SIZE,
                ),
                height,
            );
        }
    }
}

impl FacBlkMineOre {
    fn place_drill_single_row(
        &self,
        origin: VPoint,
        count: usize,
        direction_build: FacDirectionQuarter,
        direction_drill: FacDirectionQuarter,
    ) {
        for i in 0..count {
            self.output.write(BlueprintItem::new(
                FacEntMiningDrillElectric::new_modules(direction_drill, self.drill_modules)
                    .into_boxed(),
                origin.move_direction_usz(direction_build, i * ELECTRIC_DRILL_SIZE),
            ));
        }
    }

    fn place_inner_belts(&self, origin: VPoint, cur_height: usize) {
        let mut belt =
            FacBlkBettelBelt::new(self.belt, origin, self.build_direction, self.output.clone());

        let needed_poles = self.width.div_ceil(3);
        for _ in 0..needed_poles {
            belt.add_straight(2);

            let mut electric_placer_belt = belt.clone();
            electric_placer_belt.set_dummy_nav_mode(true);
            electric_placer_belt.add_straight(1);
            self.output.write(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                electric_placer_belt.next_insert_position(),
            ));

            belt.add_straight_underground(1);
            belt.add_straight(4);
        }

        if cur_height == 0 {
            belt.add_straight(self.height - cur_height);
        } else {
            belt.add_straight(cur_height - 1);
            belt.add_turn90(false);
            belt.add_straight((ELECTRIC_DRILL_SIZE * 2 * cur_height) - /*turn spacing*/1);
            belt.add_turn90(true);
            belt.add_straight(self.height - cur_height);
        }
    }
}
