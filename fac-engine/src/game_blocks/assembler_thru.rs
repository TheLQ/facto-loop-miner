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

use super::block::FacBlock;

pub struct FacBlkAssemblerThru {
    pub width: usize,
    pub height: usize,
    pub assembler: FacEntAssembler,
    pub belt_type: FacEntBeltType,
    pub inserter_type: FacEntInserterType,
}

impl FacBlock for FacBlkAssemblerThru {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();
        for height in 0..self.height {
            let super_row_pos = origin.move_y_usize(height * 9);

            // built first so when executing everything has power
            self.generate_center_offload(super_row_pos, FacDirectionQuarter::East, &mut res);

            self.generate_assembler_chain(
                super_row_pos,
                FacDirectionQuarter::East,
                false,
                &mut res,
            );
            self.generate_assembler_chain(
                super_row_pos.move_y(6),
                FacDirectionQuarter::West,
                true,
                &mut res,
            );
            self.generate_belt_turn_for_row(super_row_pos, FacDirectionQuarter::East, &mut res);
        }

        res
    }
}

const CELL_HEIGHT: usize = 3;

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
        res: &mut Vec<BlueprintItem>,
    ) {
        for row_pos in 0..self.width {
            let mut cell_x_offset = row_pos * self.cell_width();

            let mut utype = FacEntBeltUnderType::Input;
            if is_second_row {
                utype = utype.flip();
            }

            for y_offset in 0..3 {
                // lead in empty belt
                res.push(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type.clone(), direction.clone())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset, y_offset),
                ));

                // going underground
                res.push(BlueprintItem::new(
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
                res.push(BlueprintItem::new(
                    FacEntInserter::new(self.inserter_type.clone(), inserter_direction)
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 2, y_offset),
                ));
            }
            cell_x_offset += 3;

            // the actual assembler
            res.push(BlueprintItem::new(
                self.assembler.clone().into_boxed(),
                origin.move_xy_usize(cell_x_offset, 0),
            ));
            cell_x_offset += 3;

            for y_offset in 0..3 {
                // coming up underground
                res.push(BlueprintItem::new(
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
        res: &mut Vec<BlueprintItem>,
    ) {
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
                let cell_x_offset = row_pos * self.cell_width();

                // cell power
                res.push(BlueprintItem::new(
                    FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 3, side_y_offset),
                ));

                // removing inserter
                res.push(BlueprintItem::new(
                    FacEntInserter::new(self.inserter_type.clone(), side_direction.clone())
                        .into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 4, side_y_offset),
                ));

                // highlighter
                res.push(BlueprintItem::new(
                    FacEntLamp::new().into_boxed(),
                    origin.move_xy_usize(cell_x_offset + 5, side_y_offset),
                ));
            }
        }

        for cell_x_offset in 0..(self.width * self.cell_width()) {
            // belt row
            res.push(BlueprintItem::new(
                FacEntBeltTransport::new(self.belt_type.clone(), direction.clone()).into_boxed(),
                origin.move_xy_usize(cell_x_offset, cell_y_offset + 1),
            ));
        }
    }

    fn generate_belt_turn_for_row(
        &self,
        origin: VPoint,
        direction: FacDirectionQuarter,
        res: &mut Vec<BlueprintItem>,
    ) {
        let start = origin.move_x_usize(self.cell_width() * self.width);

        // going out
        for belt_num in 0..CELL_HEIGHT {
            let start = start.move_y_usize(/*reverse expansion*/ 2 - belt_num);
            for i in 0..belt_num {
                res.push(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type.clone(), direction.clone())
                        .into_boxed(),
                    start.move_x_usize(i),
                ));
            }
        }

        // going down
        for belt_num in 0..CELL_HEIGHT {
            let start = start.move_xy_usize(belt_num, /*-1 to start turn*/ 2 - belt_num);
            for i in
                0..((belt_num * /*span both rows*/2) + /*center*/CELL_HEIGHT + /*+1 from turn*/1)
            {
                res.push(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type.clone(), direction.rotate_once())
                        .into_boxed(),
                    start.move_y_usize(i),
                ));
            }
        }

        // coming back
        // 1 belt longer to do the turn
        let start = start.move_y_usize(CELL_HEIGHT * 2);
        for belt_num in 0..CELL_HEIGHT {
            let start = start.move_y_usize(belt_num);
            for i in 0..(belt_num + /*turn start*/1) {
                res.push(BlueprintItem::new(
                    FacEntBeltTransport::new(self.belt_type.clone(), direction.rotate_flip())
                        .into_boxed(),
                    start.move_x_usize(i),
                ));
            }
        }
    }
}
