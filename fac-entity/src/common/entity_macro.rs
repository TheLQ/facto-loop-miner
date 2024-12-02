#[macro_export]
macro_rules! impl_facentity {
    ($to: ident, $size: literal, $name: path) => {
        impl FacEntity for $to {
            fn name(&self) -> &FacEntityName {
                const RES: FacEntityName = $name;
                &RES
            }

            fn size(&self) -> &Size {
                const RES: Size = Size::square($size);
                &RES
            }
        }
    };
}
