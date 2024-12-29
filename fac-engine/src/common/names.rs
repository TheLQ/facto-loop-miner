use exhaustive::Exhaustive;
use strum::AsRefStr;

use crate::game_entities::{
    belt::FacEntBeltType, chest::FacEntChestType, electric_large::FacEntElectricLargeType,
    electric_mini::FacEntElectricMiniType, inserter::FacEntInserterType,
    rail_signal::FacEntRailSignalType, tier::FacTier,
};

#[derive(Clone, Debug, PartialEq, AsRefStr, Exhaustive)]
pub enum FacEntityName {
    Lamp,
    RailStraight,
    RailCurved,
    RailSignal(FacEntRailSignalType),
    Assembler(FacTier),
    Inserter(FacEntInserterType),
    Chest(FacEntChestType),
    ElectricMini(FacEntElectricMiniType),
    ElectricLarge(FacEntElectricLargeType),
    TrainStop,
    Beacon,
    Radar,
    Roboport,
    BeltTransport(FacEntBeltType),
    BeltUnder(FacEntBeltType),
    BeltSplit(FacEntBeltType),
    InfinityPower,
    Locomotive,
    CargoWagon,
}

impl FacEntityName {
    pub fn to_fac_name(&self) -> String {
        match self {
            Self::Lamp => "small-lamp".into(),
            Self::RailStraight => "straight-rail".into(),
            Self::RailCurved => "curved-rail".into(),
            Self::RailSignal(stype) => match stype {
                FacEntRailSignalType::Basic => "rail-signal",
                FacEntRailSignalType::Chain => "rail-chain-signal",
            }
            .into(),
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
                FacEntChestType::Wood => "wooden-chest",
                FacEntChestType::Iron => "iron-chest",
                FacEntChestType::Steel => "steel-chest",
                FacEntChestType::Active => "logistic-chest-active-provider",
                FacEntChestType::Passive => "logistic-chest-passive-provider",
                FacEntChestType::Storage => "logistic-chest-storage",
                FacEntChestType::Buffer => "logistic-chest-buffer",
                FacEntChestType::Requestor => "logistic-chest-requestor",
                FacEntChestType::Infinity(_) => "infinity-chest",
            }
            .into(),
            Self::ElectricMini(etype) => match etype {
                FacEntElectricMiniType::Small => "small-electric-pole",
                FacEntElectricMiniType::Medium => "medium-electric-pole",
            }
            .into(),
            Self::ElectricLarge(etype) => match etype {
                FacEntElectricLargeType::Big => "big-electric-pole",
                FacEntElectricLargeType::Substation => "substation",
            }
            .into(),
            Self::TrainStop => "train-stop".into(),
            Self::Beacon => "beacon".into(),
            Self::Radar => "radar".into(),
            Self::Roboport => "roboport".into(),
            Self::BeltTransport(btype) => format!("{}transport-belt", btype.name_prefix()),
            Self::BeltUnder(btype) => format!("{}underground-belt", btype.name_prefix()),
            Self::BeltSplit(btype) => format!("{}splitter", btype.name_prefix()),
            Self::InfinityPower => "electric-energy-interface".into(),
            Self::Locomotive => "locomotive".into(),
            Self::CargoWagon => "cargo-wagon".into(),
        }
    }
}
