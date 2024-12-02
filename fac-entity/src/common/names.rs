use crate::game_entities::{assembler::FacAssemblerLevel, inserter::FacInserterType};

pub enum FacEntityName {
    SquarePole,
    Lamp,
    Rail,
    Assembler(FacAssemblerLevel),
    Inserter(FacInserterType),
    Chest,
}
