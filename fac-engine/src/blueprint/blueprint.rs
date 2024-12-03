use crate::common::entity::FacEntity;

struct Blueprint {
    entities: Vec<Box<dyn FacEntity>>,
}
