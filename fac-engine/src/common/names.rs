use strum_macros::AsRefStr;

use crate::game_entities::{
    chest::FacChestType, electric_pole_small::ElectricPoleSmallType, inserter::FacInserterType,
    tier::FacTier,
};

#[derive(AsRefStr)]
pub enum FacEntityName {
    SquarePole,
    Lamp,
    Rail,
    Assembler(FacTier),
    Inserter(FacInserterType),
    Chest(FacChestType),
    ElectricPoleSmall(ElectricPoleSmallType),
    TrainStop,
    Beacon,
}
