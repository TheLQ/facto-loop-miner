use strum_macros::AsRefStr;

use crate::game_entities::{
    chest::FacChestType, electric_pole_small::ElectricPoleSmallType, inserter::FacInserterType,
    tier::FacTier,
};

#[derive(AsRefStr)]
pub enum FacEntityName {
    Lamp,
    Rail,
    Assembler(FacTier),
    Inserter(FacInserterType),
    Chest(FacChestType),
    ElectricPoleSmall(ElectricPoleSmallType),
    ElectricPoleBig,
    TrainStop,
    Beacon,
    Radar,
}

impl FacEntityName {
    pub fn to_fac_name(&self) -> String {
        match self {
            Self::Lamp => "small-lamp".into(),
            Self::Rail => todo!(),
            Self::Assembler(tier) => format!("assembling-machine-{}", tier.to_number()),
            Self::Inserter(itype) => match itype {
                FacInserterType::Burner => "burner-inserter",
                FacInserterType::Basic => "inserter",
                FacInserterType::Long => "long-handed-inserter",
                FacInserterType::Fast => "fast-inserter",
                FacInserterType::Filter => "filter-inserter",
                FacInserterType::Stack => "stack-inserter",
                FacInserterType::StackFilter => "stack-filter-inserter",
            }
            .into(),
            Self::Chest(ctype) => match ctype {
                FacChestType::Wood => "wooden-chest",
                FacChestType::Iron => "iron-chest",
                FacChestType::Steel => "steel-chest",
                FacChestType::Active => "logistic-chest-active-provider",
                FacChestType::Passive => "logistic-chest-passive-provider",
                FacChestType::Storage => "logistic-chest-storage",
                FacChestType::Buffer => "logistic-chest-buffer",
                FacChestType::Requestor => "logistic-chest-requestor",
            }
            .into(),
            Self::ElectricPoleSmall(ptype) => match ptype {
                ElectricPoleSmallType::Wooden => "small-electric-pole",
                ElectricPoleSmallType::Steel => "medium-electric-pole",
            }
            .into(),
            Self::ElectricPoleBig => "big-electric-pole".into(),
            Self::TrainStop => "train-stop".into(),
            Self::Beacon => "beacon".into(),
            Self::Radar => "radar".into(),
        }
    }
}
