use exhaustive::Exhaustive;

use crate::{
    blueprint::bpfac::infinity::FacBpInfinitySettings,
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
    },
};

#[derive(Debug, Clone)]
pub enum FacEntChestType {
    Wood,
    Iron,
    Steel,
    Active,
    Passive,
    Storage,
    Buffer,
    Requestor,
    Infinity(FacBpInfinitySettings),
}

impl Exhaustive for FacEntChestType {
    fn generate(u: &mut exhaustive::DataSourceTaker) -> exhaustive::Result<Self> {
        Ok(match u.choice(9)? {
            0 => Self::Wood,
            1 => Self::Iron,
            2 => Self::Steel,
            3 => Self::Active,
            4 => Self::Passive,
            5 => Self::Storage,
            6 => Self::Buffer,
            7 => Self::Requestor,
            8 => Self::Infinity(FacBpInfinitySettings::default()),
            _ => panic!("asdf"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FacEntChest {
    name: FacEntityName,
}

impl FacEntity for FacEntChest {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_infinity_settings(&self) -> Option<FacBpInfinitySettings> {
        match &self.name {
            FacEntityName::Chest(FacEntChestType::Infinity(settings)) => Some(settings.clone()),
            _ => None,
        }
    }
}

impl SquareArea for FacEntChest {
    fn area_diameter() -> usize {
        1
    }
}

impl FacEntChest {
    pub fn new(ctype: FacEntChestType) -> Self {
        Self {
            name: FacEntityName::Chest(ctype.clone()),
        }
    }

    pub fn chest_type(&self) -> &FacEntChestType {
        match &self.name {
            FacEntityName::Chest(t) => t,
            _ => panic!("wtf"),
        }
    }
}
