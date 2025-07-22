use super::{belt_bettel::FacBlkBettelBelt, block::FacBlock2};
use crate::common::varea::VArea;
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
use std::rc::Rc;
use tracing::trace;

pub struct FacBlkMineOre {
    pub ore_points: Vec<VPoint>,
    pub build_direction: FacDirectionQuarter,
    pub belt: FacEntBeltType,
    pub drill_modules: [Option<FacModule>; 3],
    pub output: Rc<FacItemOutput>,
}

impl FacBlkMineOre {
    pub fn generate(&self) -> Vec<FacBlkBettelBelt> {
        let build_area = VArea::from_arbitrary_points(&self.ore_points).normalize_3x3();
        let origin = match self.build_direction {
            FacDirectionQuarter::North => build_area.point_top_left(),
            FacDirectionQuarter::East => build_area.calc_point_top_right(),
            FacDirectionQuarter::South => build_area.point_bottom_right(),
            FacDirectionQuarter::West => build_area.calc_point_bottom_left(),
        };

        for cell_column in 0.. {
            let column_head =
                origin.move_direction_sideways_usz(self.build_direction, cell_column * 7);
            if !build_area.contains_point(&column_head) {
                break;
            }
            trace!("distance from origin column head {}", origin - column_head);

            let mut last_row_head = None;
            for cell_row in 0.. {
                let row_head = column_head
                    .move_direction_usz(self.build_direction.rotate_flip(), cell_row * 3);
                if !build_area.contains_point(&row_head) {
                    break;
                }
                trace!("distance from origin row head {}", origin - column_head);

                for (corner_start_steps, drill_flip) in [(0, false), (4, true)] {
                    let corner_start = row_head
                        .move_direction_sideways_usz(self.build_direction, corner_start_steps);
                    let corner_end = corner_start
                        .move_direction_usz(self.build_direction.rotate_flip(), 3)
                        .move_direction_sideways_usz(self.build_direction, 3);

                    const DRILL_FOR_ORE_PIXELS_MINIMUM: usize = 4;
                    let drill_area = VArea::from_arbitrary_points_pair(corner_start, corner_end);
                    assert_eq!(drill_area.as_size(), VPoint::new(3, 3));
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
                        self.build_direction.rotate_opposite()
                    } else {
                        self.build_direction.rotate_once()
                    };
                    trace!(
                        "distance from origin {}",
                        origin - drill_area.point_top_left()
                    );
                    self.output.writei(
                        FacEntMiningDrillElectric::new_modules(drill_direction, self.drill_modules),
                        // the only actual top_left regardless of direction
                        drill_area.point_top_left(),
                    );
                    last_row_head = Some((row_head, cell_row));
                }
            }

            if let Some((last_row_head, cell_row)) = last_row_head {
                self.place_inner_belt(
                    last_row_head
                        .move_direction_sideways_usz(self.build_direction, 3)
                        // move belt start from top of drill to middle of drill
                        .move_direction_usz(self.build_direction.rotate_flip(), 1),
                    (cell_row * 3) - /*last drill missing end*/1,
                );
            }
        }

        self.output.flush();

        todo!()
    }

    fn place_inner_belt(&self, origin: VPoint, straight_distance: usize) -> FacBlkBettelBelt {
        // first pole outside of neatly spaced belt ones
        if (straight_distance - 3) % 9 > 7 {
            self.output.write(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                origin.move_direction_usz(self.build_direction.rotate_flip(), 1),
            ));
        }

        let mut belt =
            FacBlkBettelBelt::new(self.belt, origin, self.build_direction, self.output.clone());
        let mut cur_distance = 0;
        while cur_distance < straight_distance {
            if (straight_distance - cur_distance + 2) % 9 == 0 {
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
        }
        belt
    }
}
