use strum_macros::AsRefStr;

use crate::game_entities::{
    chest::FacEntityChestType, electric_pole_small::ElectricPoleSmallType,
    inserter::FacEntInserterType, tier::FacTier,
};

#[derive(Clone, AsRefStr)]
pub enum FacEntityName {
    Lamp,
    Rail,
    Assembler(FacTier),
    Inserter(FacEntInserterType),
    Chest(FacEntityChestType),
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
                FacEntInserterType::Burner => "burner-inserter",
                FacEntInserterType::Basic => "inserter",
                FacEntInserterType::Long => "long-handed-inserter",
                FacEntInserterType::Fast => "fast-inserter",
                FacEntInserterType::Filter => "filter-inserter",
                FacEntInserterType::Stack => "stack-inserter",
                FacEntInserterType::StackFilter => "stack-filter-inserter",
            }
            .into(),
            Self::Chest(ctype) => match ctype {
                FacEntityChestType::Wood => "wooden-chest",
                FacEntityChestType::Iron => "iron-chest",
                FacEntityChestType::Steel => "steel-chest",
                FacEntityChestType::Active => "logistic-chest-active-provider",
                FacEntityChestType::Passive => "logistic-chest-passive-provider",
                FacEntityChestType::Storage => "logistic-chest-storage",
                FacEntityChestType::Buffer => "logistic-chest-buffer",
                FacEntityChestType::Requestor => "logistic-chest-requestor",
            }
            .into(),
            Self::ElectricPoleSmall(ptype) => match ptype {
                ElectricPoleSmallType::Wooden => "small-electric-pole",
                ElectricPoleSmallType::Steel => "medium-electric-pole",
            }
            .into(),
            Self::ElectricPoleBig => "substation".into(),
            Self::TrainStop => "train-stop".into(),
            Self::Beacon => "beacon".into(),
            Self::Radar => "radar".into(),
        }
    }
}
