use crate::{
    blueprint::bpitem::BlueprintItem,
    common::{entity::FacEntity, vpoint::VPoint},
    game_entities::{
        assembler::FacEntAssembler,
        chest::FacEntChest,
        direction::FacDirectionQuarter,
        electric_large::{FacEntElectricLarge, FacEntElectricLargeType},
        electric_mini::{FacEntElectricMini, FacEntElectricMiniType},
        inserter::{FacEntInserter, FacEntInserterType},
        lamp::FacEntLamp,
    },
};

use super::block::FacBlock;

pub struct FacBlkAssemblerCell {
    pub assembler: FacEntAssembler,
    pub side_bottom: [Option<FacBlkAssemblerCellEntry>; 3],
    pub side_right: [Option<FacBlkAssemblerCellEntry>; 3],
    pub is_big_power: bool,
}

pub struct FacBlkAssemblerCellEntry {
    pub inserter: FacEntInserterType,
    pub chest: FacEntChest,
    pub is_loader: bool,
}

impl FacBlock for FacBlkAssemblerCell {
    fn generate(&self, origin: VPoint) -> Vec<BlueprintItem> {
        let mut res = Vec::new();

        res.push(BlueprintItem::new(
            self.assembler.clone().into_boxed(),
            origin.move_xy(1, 1),
        ));

        res.push(BlueprintItem::new(FacEntLamp::new().into_boxed(), origin));

        let power_pos = origin.move_xy(4, 4);
        if self.is_big_power {
            res.push(BlueprintItem::new(
                FacEntElectricLarge::new(FacEntElectricLargeType::Substation).into_boxed(),
                power_pos,
            ));
        } else {
            res.push(BlueprintItem::new(
                FacEntElectricMini::new(FacEntElectricMiniType::Medium).into_boxed(),
                power_pos,
            ));
        }

        for (i, bottom_entry) in self.side_bottom.iter().enumerate() {
            if let Some(entry) = bottom_entry {
                let row_point = origin.move_xy_usize(1 + i, 4);

                let inserter_direction = if entry.is_loader {
                    FacDirectionQuarter::South
                } else {
                    FacDirectionQuarter::North
                };
                res.push(BlueprintItem::new(
                    FacEntInserter::new(entry.inserter.clone(), inserter_direction).into_boxed(),
                    row_point,
                ));

                res.push(BlueprintItem::new(
                    entry.chest.clone().into_boxed(),
                    row_point.move_y(1),
                ));
            }
        }

        for (i, right_entry) in self.side_right.iter().enumerate() {
            if let Some(entry) = right_entry {
                let row_point = origin.move_xy_usize(4, 3 + i);

                let inserter_direction = if entry.is_loader {
                    FacDirectionQuarter::South
                } else {
                    FacDirectionQuarter::North
                };
                res.push(BlueprintItem::new(
                    FacEntInserter::new(entry.inserter.clone(), inserter_direction).into_boxed(),
                    row_point,
                ));

                res.push(BlueprintItem::new(
                    entry.chest.clone().into_boxed(),
                    row_point.move_x(1),
                ));
            }
        }

        res
    }
}
