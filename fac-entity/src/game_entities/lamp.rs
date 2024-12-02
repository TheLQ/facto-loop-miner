use crate::{
    common::{
        entity::{FacEntity, Size},
        name::FacEntityName,
    },
    impl_facentity,
};

pub struct FacLamp {}

impl_facentity!(FacLamp, 2, FacEntityName::Lamp);
