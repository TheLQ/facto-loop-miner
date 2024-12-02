use crate::game_entities::{
    assembler::FacAssemblerLevel, chest::FacChestType, inserter::FacInserterType,
};

pub enum FacEntityName {
    SquarePole,
    Lamp,
    Rail,
    Assembler(FacAssemblerLevel),
    Inserter(FacInserterType),
    Chest(FacChestType),
}
