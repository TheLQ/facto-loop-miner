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
    ElectricMiningDrill,
    SolarPanel,
    // resources
    IronOre,
    IronPlate,
    IronStick,
    IronGear,
    Steel,
    CopperOre,
    CopperPlate,
    CopperCable,
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
            Self::ElectricMiningDrill => "electric-mining-drill".into(),
            Self::SolarPanel => "solar-panel".into(),
            // resources
            Self::IronOre => "iron-ore".into(),
            Self::IronPlate => "iron-plate".into(),
            Self::IronStick => "iron-stick".into(),
            Self::IronGear => "iron-gear-wheel".into(),
            Self::Steel => "steel".into(),
            Self::CopperOre => "copper-ore".into(),
            Self::CopperPlate => "copper-plate".into(),
            Self::CopperCable => "copper-cable".into(),
        }
    }
}

pub struct FacEntityNameBuilder {
    names: Vec<FacEntityName>,
}

impl FacEntityNameBuilder {
    pub fn new() -> Self {
        Self { names: Vec::new() }
    }

    pub fn into_vec(self) -> Vec<FacEntityName> {
        self.names
    }

    //

    pub fn new_all() -> Self {
        Self::new()
            .with_electric()
            .with_belts()
            .with_rail()
            .with_chests()
            .with_drill()
            .with_inserters()
    }

    //

    const fn group_electric() -> [FacEntityName; 5] {
        [
            FacEntityName::ElectricMini(FacEntElectricMiniType::Small),
            FacEntityName::ElectricMini(FacEntElectricMiniType::Medium),
            FacEntityName::ElectricLarge(FacEntElectricLargeType::Big),
            FacEntityName::ElectricLarge(FacEntElectricLargeType::Substation),
            FacEntityName::InfinityPower,
        ]
    }

    pub fn with_electric(mut self) -> Self {
        self.names.extend(Self::group_electric());
        self
    }

    //

    const fn group_belts() -> [FacEntityName; 9] {
        [
            FacEntityName::BeltTransport(FacEntBeltType::Basic),
            FacEntityName::BeltTransport(FacEntBeltType::Fast),
            FacEntityName::BeltTransport(FacEntBeltType::Express),
            FacEntityName::BeltUnder(FacEntBeltType::Basic),
            FacEntityName::BeltUnder(FacEntBeltType::Fast),
            FacEntityName::BeltUnder(FacEntBeltType::Express),
            FacEntityName::BeltSplit(FacEntBeltType::Basic),
            FacEntityName::BeltSplit(FacEntBeltType::Fast),
            FacEntityName::BeltSplit(FacEntBeltType::Express),
        ]
    }

    pub fn with_belts(mut self) -> Self {
        self.names.extend(Self::group_belts());
        self
    }

    //

    const fn group_rail() -> [FacEntityName; 5] {
        [
            FacEntityName::RailStraight,
            FacEntityName::RailCurved,
            FacEntityName::RailSignal(FacEntRailSignalType::Basic),
            FacEntityName::RailSignal(FacEntRailSignalType::Chain),
            FacEntityName::TrainStop,
        ]
    }

    pub fn with_rail(mut self) -> Self {
        self.names.extend(Self::group_rail());
        self
    }

    // -- exhaustive generators

    fn group_chests() -> impl Iterator<Item = FacEntityName> {
        FacEntChestType::iter_exhaustive(None).map(FacEntityName::Chest)
    }

    pub fn with_chests(mut self) -> Self {
        self.names.extend(Self::group_chests());
        self
    }

    //

    fn group_inserters() -> impl Iterator<Item = FacEntityName> {
        FacEntInserterType::iter_exhaustive(None).map(FacEntityName::Inserter)
    }

    pub fn with_inserters(mut self) -> Self {
        self.names.extend(Self::group_inserters());
        self
    }

    // -- misc singles

    pub fn with_drill(mut self) -> Self {
        self.names.push(FacEntityName::ElectricMiningDrill);
        self
    }
}

// macro_rules! all_factory {
//     ($($enum_name: expr_2021, $to_string: literal),+) => {
//         /*const*/ fn to_facto_name_inner(&self) -> &'static str {
//             match self {
//                 $($enum_name => $to_string,)+
//                 cur_input => panic!("unknown name")
//             }
//         }

//         fn from_facto_name(&self, input: &str) -> Self {
//             match input {
//                 $($to_string => $enum_name,)+
//                 cur_input => panic!("unknown name {}", cur_input)
//             }
//         }
//     };
// }

// Self::Assembler(tier) , format!("assembling-machine-{}", tier.to_number()),

// Self::BeltTransport(btype) , format!("{}transport-belt", btype.name_prefix()),
// Self::BeltUnder(btype) , format!("{}underground-belt", btype.name_prefix()),
// Self::BeltSplit(btype) , format!("{}splitter", btype.name_prefix()),
// impl FacEntityName {
//     all_factory! {
//         Self::Lamp , "small-lamp",
//             Self::RailStraight , "straight-rail",
//             Self::RailCurved , "curved-rail",
//             Self::RailSignal(FacEntRailSignalType::Basic) , "rail-signal",
//             Self::RailSignal(FacEntRailSignalType::Chain) , "rail-chain-signal",
//             Self::Inserter(FacEntInserterType::Burner), "burner-inserter",
//             Self::Inserter(FacEntInserterType::Basic), "inserter",
//             Self::Inserter(FacEntInserterType::Long), "long-handed-inserter",
//             Self::Inserter(FacEntInserterType::Fast), "fast-inserter",
//             Self::Inserter(FacEntInserterType::Filter), "filter-inserter",
//             Self::Inserter(FacEntInserterType::Stack), "stack-inserter",
//             Self::Inserter(FacEntInserterType::StackFilter), "stack-filter-inserter",
//             Self::Chest(FacEntChestType::Wood), "wooden-chest",
//             Self::Chest(FacEntChestType::Iron), "iron-chest",
//             Self::Chest(FacEntChestType::Steel), "steel-chest",
//             Self::Chest(FacEntChestType::Active), "logistic-chest-active-provider",
//             Self::Chest(FacEntChestType::Passive), "logistic-chest-passive-provider",
//             Self::Chest(FacEntChestType::Storage), "logistic-chest-storage",
//             Self::Chest(FacEntChestType::Buffer), "logistic-chest-buffer",
//             Self::Chest(FacEntChestType::Requestor), "logistic-chest-requestor",
//             Self::Chest(FacEntChestType::Infinity(_)), "infinity-chest",
//             Self::ElectricMini(FacEntElectricMiniType::Small), "small-electric-pole",
//             Self::ElectricMini(FacEntElectricMiniType::Medium), "medium-electric-pole",
//             Self::ElectricLarge(FacEntElectricLargeType::Big), "big-electric-pole",
//             Self::ElectricLarge(FacEntElectricLargeType::Substation), "substation",
//             Self::TrainStop , "train-stop",
//             Self::Beacon , "beacon",
//             Self::Radar , "radar",
//             Self::Roboport , "roboport",
//             Self::InfinityPower , "electric-energy-interface",
//             Self::Locomotive , "locomotive",
//             Self::CargoWagon , "cargo-wagon",
//             // resources
//             Self::IronOre , "iron-ore",
//             Self::IronPlate , "iron-plate",
//             Self::IronStick , "iron-stick",
//             Self::IronGear , "iron-gear-wheel",
//             Self::Steel , "steel",
//             Self::CopperOre , "copper-ore",
//             Self::CopperPlate , "copper-plate",
//             Self::CopperCable , "copper-cable"
//     }
// }

// macro_rules! name_factory {
//     ($($enum_name: ident, $to_string: literal),+) => {
//         const fn to_facto_name(&self) -> &'static str {
//             match self {
//                 $(Self::$enum_name => $to_string,)+
//             }
//         }

//         fn from_facto_name(&self, input: &str) -> Self {
//             match input {
//                 $($to_string => Self::$enum_name,)+
//                 cur_input => panic!("unknown name {}".into(), cur_input)
//             }
//         }
//     };
// }

// impl FacEntConcreteType {
//     name_factory! {
//     Basic,
//      "concrete",
//     Hazard,
//     "hazard-concrete",
//     Refined,
//     "refined-concrete",
//     RefinedHazard,
//      "refined-hazard-concrete"
//     }
// }
