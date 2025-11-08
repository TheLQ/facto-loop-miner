macro_rules! vs_impl_for {
    (pixel $target_mod:ident) => {
        vs_impl_for!(@plug_impl
            pixel => $target_mod,
            pixels_mut => pixels,
            pixels,
        );
    };

    (patch $target_mod:ident) => {
        vs_impl_for!(@plug_impl
            patch => $target_mod,
            patches_mut => patches,
            patches, pixels,
        );
    };

    (rails $target_mod:ident) => {
        vs_impl_for!(@plug_impl
            rail => $target_mod,
            rails_mut => rails,
            rails, pixels,
        );
    };

    (@plug_impl
        $trait_mod:ident => $target_mod:ident,
        $fn_mut:ident => $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl super::$trait_mod::AsVsMut for super::$target_mod::PlugMut<'_> {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for super::$target_mod::PlugMut<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for super::$target_mod::Plug<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    };
}
vs_impl_for!(pixel patch);
vs_impl_for!(pixel rail);
vs_impl_for!(pixel nav);
vs_impl_for!(patch nav);
vs_impl_for!(rails nav);

// todo: need macro param that's both an ident or ident with lifetime
macro_rules! vs_builder {
    (
        $trait_mod:ident,
        $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl super::$trait_mod::AsVs for super::$trait_mod::PlugMut<'_> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    }
}
vs_builder!(pixel, pixels, pixels,);
vs_builder!(patch, patches, patches, pixels,);
vs_builder!(rail, rails, rails, pixels,);

macro_rules! vs_main {
    (
        $for_struct:path,
        $trait_mod:ident,
        $fn_mut:ident => $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl super::$trait_mod::AsVsMut for $for_struct {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl super::$trait_mod::AsVs for $for_struct {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    }
}
vs_main!(
    super::core::VSurface,
    pixel,
    pixels_mut => pixels,
    pixels,
);
vs_main!(
    super::core::VSurface,
    patch,
    patches_mut => patches,
    patches, pixels,
);
vs_main!(
    super::core::VSurface,
    rail,
    rails_mut => rails,
    rails, pixels,
);
vs_main!(
    super::core::VSurface,
    nav,
    nav_mut => nav,
    rails, patches, pixels,
);

vs_main!(
    super::rail::PlugCopy,
    rail,
    rails_mut => rails,
    rails, pixels,
);
vs_main!(
    super::rail::PlugCopy,
    pixel,
    pixels_mut => pixels,
    pixels,
);
