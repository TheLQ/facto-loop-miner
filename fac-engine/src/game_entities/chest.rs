use std::fmt::Debug;

use debug_ignore::DebugIgnore;
use exhaustive::Exhaustive;
use itertools::Itertools;

use crate::{
    blueprint::bpfac::infinity::FacBpInfinitySettings,
    common::{
        entity::{FacEntity, SquareArea},
        names::FacEntityName,
        vpoint::C_BLOCK_LINE,
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

#[derive(Clone)]
pub struct FacEntChest {
    ctype: FacEntChestType,
    name: DebugIgnore<FacEntityName>,
}

impl FacEntity for FacEntChest {
    fn name(&self) -> &FacEntityName {
        &self.name
    }

    fn to_fac_infinity_settings(&self) -> Option<FacBpInfinitySettings> {
        match &self.ctype {
            FacEntChestType::Infinity(settings) => Some(settings.clone()),
            _ => None,
        }
    }
}

impl Debug for FacEntChest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.chest_type() {
            FacEntChestType::Infinity(FacBpInfinitySettings {
                filters,
                remove_unfiltered_items,
            }) => {
                #[derive(Debug)]
                struct FacEntChestInfinity {
                    #[allow(dead_code)]
                    remove: bool,
                    #[allow(dead_code)]
                    filters: String,
                }
                let res = FacEntChestInfinity {
                    remove: *remove_unfiltered_items,
                    filters: filters
                        .iter()
                        .map(|f| f.name.clone())
                        .join(&C_BLOCK_LINE.to_string()),
                };
                Debug::fmt(&res, f)
            }
            ctype => {
                #[derive(Debug)]
                struct FacEntChestX(#[allow(dead_code)] FacEntChestType);
                let res = FacEntChestX(ctype.clone());
                Debug::fmt(&res, f)
            }
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
            name: FacEntityName::Chest(ctype.clone()).into(),
            ctype,
        }
    }

    pub fn chest_type(&self) -> &FacEntChestType {
        &self.ctype
    }
}
