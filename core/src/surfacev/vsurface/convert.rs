macro_rules! vs_narrow_type_to_for {
    // (pixel $target_mod:ident $( $life:ident, )+) => {
    (pixel $target_mod:ident) => {
        vs_narrow_type_to_for!(@plug_impl
            pixel => $target_mod,
            pixels_mut => pixels,
            // $( $life, )+ => pixels,
            pixels,
        );
    };

    (patch $target_mod:ident) => {
        vs_narrow_type_to_for!(@plug_impl
            patch => $target_mod,
            patches_mut => patches,
            patches, pixels, mines,
        );
    };


    (mine $target_mod:ident) => {
        vs_narrow_type_to_for!(@plug_impl
            mine => $target_mod,
            mines_mut => mines,
            mines,
        );
    };

    (rails $target_mod:ident) => {
        vs_narrow_type_to_for!(@plug_impl
            rail => $target_mod,
            rails_mut => rails,
            rails, pixels,
        );
    };

    (nav $target_mod:ident) => {
        vs_narrow_type_to_for!(@plug_impl
            patch => $target_mod,
            patches_mut => patches,
            pixels, patches, mines, rails,
        );
    };

    (@plug_impl
        $trait_mod:ident => $target_mod:ident,
        $fn_mut:ident => $fn_ref:ident,
        // $( $life:ident, )+ => $( $field:ident, )+
        $( $field:ident, )+
    ) => {
        // impl<'s> super::$target_mod::PlugMut<$( $life, )+> {
        //     pub fn $fn_mut(&'s mut self) -> super::$trait_mod::PlugMut<'s > {
        //         super::$trait_mod::PlugMut { $( $field: self.$field, )+ }
        //     }
        //
        //     pub fn $fn_ref(&self) -> super::$trait_mod::Plug<'s> {
        //         super::$trait_mod::Plug { $( $field: &*self.$field, )+ }
        //     }
        // }

        // impl<'s> super::$target_mod::Plug<'s> {
        //     pub fn $fn_ref(&self) -> super::$trait_mod::Plug<'s > {
        //         super::$trait_mod::Plug { $( $field: &*self.$field, )+ }
        //     }
        // }

        impl super::$trait_mod::AsVsMut for super::$target_mod::PlugMut<'_> {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_ > {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for super::$target_mod::PlugMut<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                // let Self { $( $field, )+ .. } = self;
                // super::$trait_mod::Plug { $( $field: &*$field, )+ }
                super::$trait_mod::Plug { $( $field: &*self.$field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for super::$target_mod::Plug<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field: $field.clone(), )+ }
            }
        }
    };
}
vs_narrow_type_to_for!(pixel patch);
vs_narrow_type_to_for!(pixel rail);
vs_narrow_type_to_for!(pixel nav);
//
vs_narrow_type_to_for!(patch nav);
//
vs_narrow_type_to_for!(mine nav);
vs_narrow_type_to_for!(mine patch);
//
vs_narrow_type_to_for!(rails nav);
// vs_narrow_type_to_for!(rails core_plugs);

macro_rules! vs_plug_mut_to_plug {
    (
        $trait_mod:ident,
        $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl super::$trait_mod::AsVs for super::$trait_mod::PlugMut<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                // let super::$trait_mod::PlugMut::<'s> { $( $field, )+ .. } = self;
                // super::$trait_mod::Plug { $( $field: &*$field, )+ }
                super::$trait_mod::Plug { $( $field: &*self.$field, )+ }
            }
        }
    }
}
vs_plug_mut_to_plug!(pixel, pixels, pixels,);
vs_plug_mut_to_plug!(patch, patches, patches, pixels, mines,);
vs_plug_mut_to_plug!(rail, rails, rails, pixels,);
//
// // impl<'s> super::pixel::AsVs<'s> for super::pixel::PlugMut<'s> {
// //     fn pixels(&self) -> super::pixel::Plug<'s> {
// //         // let super::pixel::PlugMut::<'s> { pixels } = self;
// //         // let pixels: &'s VEntityMap<VPixel> = self.pixels;
// //         // super::pixel::Plug { pixels }
// //         super::pixel::Plug {
// //             pixels: self.pixels,
// //         }
// //     }
// // }

macro_rules! vs_actual_structs {
    (
        $for_struct:path,
        $trait_mod:tt,
        $fn_mut:ident => $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl super::$trait_mod::AsVsMut for $for_struct  {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for $for_struct {
            fn $fn_ref(&self) -> super::$trait_mod::Plug {
                // let Self { $( $field, )+ .. } = self;
                // super::$trait_mod::Plug { $( $field: &*$field, )+ }
                super::$trait_mod::Plug { $( $field: &self.$field, )+ }
            }
        }
        // Above traits don't make sense
        // nope go back to traits
        // impl $for_struct {
        //     pub fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_> {
        //         super::$trait_mod::PlugMut { $( $field: &mut self.$field, )+ }
        //     }
        //
        //     pub fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
        //         super::$trait_mod::Plug { $( $field: &self.$field, )+ }
        //     }
        // }
    };
}
vs_actual_structs!(
    super::core::VSurface,
    pixel,
    pixels_mut => pixels,
    pixels,
);
vs_actual_structs!(
    super::core::VSurface,
    patch,
    patches_mut => patches,
    patches, pixels, mines,
);
vs_actual_structs!(
    super::core::VSurface,
    rail,
    rails_mut => rails,
    rails, pixels,
);
vs_actual_structs!(
    super::core::VSurface,
    nav,
    nav_mut => nav,
    rails, patches, pixels, mines,
);
// vs_actual_structs!(
//     super::core::VSurface,
//     core_plugs,
//     core_mut => core,
//     rails, patches, pixels, mines,
// );
vs_actual_structs!(
    super::pixel::PlugCopy,
    pixel,
    pixels_mut => pixels,
    pixels,
);

vs_actual_structs!(
    super::rail::PlugCopy,
    rail,
    rails_mut => rails,
    rails, pixels,
);
vs_actual_structs!(
    super::rail::PlugCopy,
    pixel,
    pixels_mut => pixels,
    pixels,
);
