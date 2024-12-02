use super::entity::FacEntity;

trait FacGroup {
    fn name(&self) -> &str;

    fn into_children(self) -> Vec<Box<dyn FacEntity>>;
}
