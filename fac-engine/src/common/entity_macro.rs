#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! def_entity_size_square {
    ($size: literal) => {
        fn size(&self) -> &crate::common::entity::Size {
            const RES: crate::common::entity::Size = crate::common::entity::Size::square($size);
            &RES
        }
    };
}

#[macro_export]
macro_rules! def_entity_name {
    ($name: path) => {
        fn name(&self) -> &FacEntityName {
            const RES: FacEntityName = $name;
            &RES
        }
    };
}
