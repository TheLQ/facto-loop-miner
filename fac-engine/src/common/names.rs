use strum_macros::AsRefStr;

use crate::game_entities::{
    assembler::FacAssemblerLevel, chest::FacChestType, electric_pole_small::ElectricPoleSmallType,
    inserter::FacInserterType,
};

#[derive(AsRefStr)]
pub enum FacEntityName {
    SquarePole,
    Lamp,
    Rail,
    Assembler(FacAssemblerLevel),
    Inserter(FacInserterType),
    Chest(FacChestType),
    ElectricPoleSmall(ElectricPoleSmallType),
    TrainStop,
}
