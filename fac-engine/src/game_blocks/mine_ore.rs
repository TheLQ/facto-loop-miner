use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock2};
use crate::common::entity::{FacArea, SquareArea, SquareAreaConst};
use crate::common::varea::VArea;
use crate::common::vpoint::VPOINT_THREE;
use crate::game_blocks::block::FacBlockFancy;
use crate::game_entities::belt_transport::FacEntBeltTransport;
use crate::game_entities::chest::{FacEntChest, FacEntChestType};
use crate::game_entities::electric_mini::FacEntElectricMini;
use crate::{
    blueprint::{bpitem::BlueprintItem, output::FacItemOutput},
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        belt::FacEntBeltType,
        direction::FacDirectionQuarter,
        electric_mini::FacEntElectricMiniType,
        mining_drill_electric::{ELECTRIC_DRILL_SIZE, FacEntMiningDrillElectric},
        module::FacModule,
    },
};
use facto_loop_miner_common::util::always_true_test;
use std::rc::Rc;
use tracing::trace;

const ROW_DEPTH: usize = (FacEntMiningDrillElectric::DIAMETER * 2) + FacEntBeltTransport::DIAMETER;

pub struct FacBlkMineOre {
    pub ore_points: Vec<VPoint>,
    pub exit_clockwise: bool,
    pub exit_direction: FacDirectionQuarter,
    pub belt: FacEntBeltType,
    pub drill_modules: [Option<FacModule>; 3],
    pub output: Rc<FacItemOutput>,
}

impl FacBlockFancy<Vec<FacBlkBettelBelt>> for FacBlkMineOre {
    fn generate(&self) -> Vec<FacBlkBettelBelt> {
        let build_area = VArea::from_arbitrary_points(&self.ore_points).normalize_3x3();
        let origin = match self.exit_direction {
            FacDirectionQuarter::North => build_area.point_top_left(),
            FacDirectionQuarter::East => build_area.calc_point_top_right(),
            FacDirectionQuarter::South => build_area.point_bottom_right(),
            FacDirectionQuarter::West => build_area.calc_point_bottom_left(),
        };

        let mut belts: Vec<FacBlkBettelBelt> = Vec::new();
        'outer: for cell_row in 0.. {
            let row_head =
                origin.move_direction_sideways_usz(self.exit_direction, cell_row * ROW_DEPTH);
            if !build_area.contains_point(&row_head) {
                break;
            }
            trace!("distance from origin column head {}", origin - row_head);

            let mut last_column_head = None;
            'row: for cell_column in 0.. {
                let column_head = row_head.move_direction_usz(
                    self.exit_direction.rotate_flip(),
                    cell_column * FacEntMiningDrillElectric::DIAMETER,
                );
                if !build_area.contains_point(&column_head) {
                    break;
                }
                trace!("distance from origin row head {}", origin - column_head);

                for (corner_start_steps, drill_flip) in [
                    (0, false),
                    (
                        FacEntMiningDrillElectric::DIAMETER + FacEntBeltTransport::DIAMETER,
                        true,
                    ),
                ] {
                    let corner_start = column_head
                        .move_direction_sideways_usz(self.exit_direction, corner_start_steps);
                    let corner_end = corner_start
                        .move_direction_usz(
                            self.exit_direction.rotate_flip(),
                            FacEntMiningDrillElectric::DIAMETER,
                        )
                        .move_direction_sideways_usz(
                            self.exit_direction,
                            FacEntMiningDrillElectric::DIAMETER,
                        );

                    const DRILL_FOR_ORE_PIXELS_MINIMUM: usize = 4;
                    let drill_area = VArea::from_arbitrary_points_pair(corner_start, corner_end);
                    assert_eq!(drill_area.as_size(), VPOINT_THREE);
                    if self
                        .ore_points
                        .iter()
                        .filter(|p| drill_area.contains_point(p))
                        .count()
                        < DRILL_FOR_ORE_PIXELS_MINIMUM
                    {
                        continue;
                    }

                    let drill_direction = if drill_flip {
                        self.exit_direction.rotate_opposite()
                    } else {
                        self.exit_direction.rotate_once()
                    };
                    trace!(
                        "distance from origin {}",
                        origin - drill_area.point_top_left()
                    );
                    self.output.writei(
                        FacEntMiningDrillElectric::new_modules(drill_direction, self.drill_modules),
                        // the only actual top_left regardless of direction
                        drill_area.point_top_left(),
                        // corner_start,
                    );
                    last_column_head = Some((column_head, cell_column));

                    // if always_true_test() {
                    //     break 'row;
                    // }
                }
            }

            if let Some((last_column_head, cell_column)) = last_column_head {
                let belt = self.place_inner_belt(
                    last_column_head
                        .move_alloc::<FacEntBeltTransport>(self.exit_direction)
                        .move_direction_sideways_usz(self.exit_direction, 3)
                        // move belt start from top of drill to middle of drill
                        .move_direction_usz(self.exit_direction.rotate_flip(), 2),
                    (cell_column + /*total*/1) * FacEntMiningDrillElectric::DIAMETER,
                );
                belts.push(belt);
                // self.output.writei(
                //     FacEntChest::new(FacEntChestType::Active),
                //     last_column_head
                //         .move_alloc_usz(self.exit_direction, 1)
                //         .move_direction_sideways_usz(self.exit_direction, 3)
                //         .move_direction_usz(self.exit_direction.rotate_flip(), 2),
                // );
                // break 'outer;
            }

            // if always_true_test() {
            //     break;
            // }
        }

        if self.exit_clockwise {
            belts.reverse();
        }
        for (i, belt) in belts.iter_mut().enumerate() {
            belt.add_straight(i);
            belt.add_turn90(self.exit_clockwise);
            belt.add_straight(i * ROW_DEPTH);
        }

        self.output.flush();

        belts
    }
}

impl FacBlkMineOre {
    fn place_inner_belt(&self, origin: VPoint, straight_distance: usize) -> FacBlkBettelBelt {
        assert_ne!(straight_distance, 0, "belt going 0 distance");

        // first pole outside of neatly spaced belt ones
        // todo: this is horrible magic
        if (straight_distance) % 9 > /*magic tuning*/2 {
            self.output.write(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                origin.move_direction_usz(self.exit_direction.rotate_flip(), 1),
            ));
        }

        let mut belt =
            FacBlkBettelBelt::new(self.belt, origin, self.exit_direction, self.output.clone());
        let mut cur_distance = 0;
        while cur_distance < (straight_distance - /*first drill missing 1 belt*/1) {
            if (straight_distance - cur_distance + /*magic tuning*/1) % 9 == 0 {
                let mut electro_pos_belt = belt.clone();
                electro_pos_belt.set_dummy_nav_mode(true);
                electro_pos_belt.add_straight(1);
                self.output.writei(
                    FacEntElectricMini::new(FacEntElectricMiniType::Medium),
                    electro_pos_belt.next_insert_position(),
                );

                belt.add_straight_underground(1);
                cur_distance += 3;
            } else {
                belt.add_straight(1);
                cur_distance += 1;
            }

            if cur_distance > 1000 {
                panic!("uhh {cur_distance} and {straight_distance}");
            }
        }
        belt
    }
}
