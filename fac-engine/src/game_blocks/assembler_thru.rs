use std::rc::Rc;

use tracing::info;

use super::block::FacBlock;
use crate::blueprint::output::{ContextLevel, FacItemOutput};
use crate::game_blocks::belt_bettel::FacBlkBettelBelt;
use crate::game_entities::infinity_power::FacEntInfinityPower;
use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        assembler::FacEntAssembler,
        belt::FacEntBeltType,
        belt_transport::FacEntBeltTransport,
        belt_under::{FacEntBeltUnder, FacEntBeltUnderType},
        direction::FacDirectionQuarter,
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
    },
};

/// Assembler farm with belt through design
pub struct FacBlkAssemblerThru {
    pub width: usize,
    pub height: usize,
    pub output_padding_width: Option<usize>,
    pub assembler: FacEntAssembler,
    pub belt_type: FacEntBeltType,
    pub inserter_input: FacEntInserterType,
    pub inserter_output: FacEntInserterType,
    pub output: Rc<FacItemOutput>,
}

impl FacBlock for FacBlkAssemblerThru {
    fn generate(&self, origin: VPoint) {
        let _ = &mut self.output.context_handle(
            ContextLevel::Micro,
            format!("Assembler-{}", self.assembler.recipe().as_ref()),
        );
        for height in 0..self.height {
            let super_row_pos = origin.move_y_usize(height * 9);

            if height == 0 {
                self.generate_belt_lead(super_row_pos);
            }

            // built first so when executing everything has power
            self.generate_center_offload(super_row_pos, FacDirectionQuarter::East, height);

            self.generate_assembler_chain(
                super_row_pos.move_x_usize(START_BUFFER),
                FacDirectionQuarter::East,
                false,
            );
            self.generate_assembler_chain(
                super_row_pos.move_xy_usize(START_BUFFER, CELL_HEIGHT * 2),
                FacDirectionQuarter::West,
                true,
            );

            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, "Intra".into());
            self.generate_belt_turn_for_row(super_row_pos.move_x_usize(START_BUFFER));

            if self.height > 1 && height != self.height - 1 {
                self.generate_belt_turn_for_between(super_row_pos.move_y_usize(CELL_HEIGHT * 2));
            }
        }
    }
}

const CELL_HEIGHT: usize = 3;
const START_BUFFER: usize = 5;

impl FacBlkAssemblerThru {
    fn cell_width(&self) -> usize {
        const CELL_WIDTH: usize = /*lead up*/ 3 + /*assembler*/3 + /*exit*/1;
        CELL_WIDTH
    }

    fn generate_assembler_chain(
        &self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        is_second_row: bool,
    ) {
        for row_pos in 0..self.width {
            let _ = &mut self
                .output
                .context_handle(ContextLevel::Micro, format!("Chain-{row_pos}"));
            let mut cell_x_offset = row_pos * self.cell_width();

            let mut utype = FacEntBeltUnderType::Input;
            if is_second_row {
                utype = utype.flip();
            }

            for y_offset in 0..3 {
                // lead in empty belt
                self.output.write(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type.clone(), direction.clone())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset, y_offset),
                ));

                // going underground
                self.output.write(BlueprintItem::new(
                    FacEntBeltUnder::new(self.belt_type.clone(), direction.clone(), utype.clone())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 1, y_offset),
                ));

                // inserter
                let inserter_direction = if is_second_row {
                    // belt goes other way but inserters are in same place
                    direction.clone()
                } else {
                    direction.rotate_flip()
                };
                self.output.write(BlueprintItem::new(
                    FacEntInserter::new(self.inserter_input.clone(), inserter_direction)
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 2, y_offset),
                ));
            }
            cell_x_offset += 3;

            // the actual assembler
            self.output.write(BlueprintItem::new(
                self.assembler.clone().into_boxed(),
                origin.move_xy_usize(cell_x_offset, 0),
            ));
            cell_x_offset += 3;

            for y_offset in 0..3 {
                // coming up underground
                self.output.write(BlueprintItem::new(
                    FacEntBeltUnder::new(self.belt_type.clone(), direction.clone(), utype.flip())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset, y_offset),
                ));
            }
        }
    }

    fn generate_center_offload(
        &self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        cur_height: usize,
    ) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, "Center".into());
        let cell_y_offset = 3;

        struct Side {
            side_direction: FacDirectionQuarter,
            side_y_offset: usize,
        }
        for Side {
            side_direction,
            side_y_offset,
        } in [
            Side {
                side_direction: FacDirectionQuarter::North,
                side_y_offset: cell_y_offset,
            },
            Side {
                side_direction: FacDirectionQuarter::South,
                side_y_offset: cell_y_offset + 2,
            },
        ] {
            for row_pos in 0..self.width {
                let cell_x_offset = START_BUFFER + row_pos * self.cell_width();

                // cell power
                self.output.write(BlueprintItem::new(
                    FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 3, side_y_offset),
                ));

                // removing inserter
                self.output.write(BlueprintItem::new(
                    FacEntInserter::new(self.inserter_output.clone(), side_direction.clone())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 4, side_y_offset),
                ));

                // highlighter
                self.output.write(BlueprintItem::new(
                    FacEntLamp::new().into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 5, side_y_offset),
                ));
            }
        }

        if cur_height == 0 {
            self.output.write(BlueprintItem::new(
                FacEntInfinityPower::new().into_boxed(),
                origin.move_x_usize(
                    START_BUFFER + (self.width * self.cell_width()) + CELL_HEIGHT + 1,
                ),
            ));
            self.output.write(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                origin.move_xy_usize(
                    START_BUFFER + (self.width * self.cell_width()) + CELL_HEIGHT + 1,
                    3,
                ),
            ));
        }

        let mut belt = FacBlkBettelBelt::new(
            self.belt_type,
            origin.move_xy_usize(START_BUFFER, cell_y_offset + /*center*/1),
            direction,
            self.output.clone(),
        );
        belt.add_straight(self.width * self.cell_width() - 1);
        belt.add_straight_underground(4);

        if let Some(output_padding_width) = &self.output_padding_width {
            let remaining_width = output_padding_width - self.width;
            info!("adding {}", remaining_width);
            belt.add_straight(self.cell_width() * remaining_width);
        }
    }

    fn generate_belt_lead(&self, origin: VPoint) {
        for x in 0..START_BUFFER {
            for y in 0..CELL_HEIGHT {
                self.output.write(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type, FacDirectionQuarter::East)
                        .into_boxed(),
                    origin.move_xy_usize(x, y),
                ));
            }
        }
    }

    fn generate_belt_turn_for_row(&self, origin: VPoint) {
        let start = origin.move_x_usize(self.cell_width() * self.width);
        FacBlkBettelBelt::u_turn_from_east(
            &self.belt_type,
            start,
            CELL_HEIGHT,
            CELL_HEIGHT,
            self.output.clone(),
        )
    }

    fn generate_belt_turn_for_between(&self, origin: VPoint) {
        let start = origin.move_x_usize(START_BUFFER - CELL_HEIGHT);
        FacBlkBettelBelt::u_turn_from_west(
            &self.belt_type,
            start,
            0,
            CELL_HEIGHT,
            self.output.clone(),
        )
    }

    pub fn total_height(height: usize) -> usize {
        (CELL_HEIGHT * 3) * height
    }
}
