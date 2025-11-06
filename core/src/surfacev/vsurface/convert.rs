macro_rules! as_vs_convert {
    (pixel $target_mod:ident) => {
        as_vs_convert!(@plug_impl
            pixel => $target_mod,
            pixels_mut => pixels,
            pixels,
        );
    };

    (patch $target_mod:ident) => {
        as_vs_convert!(@plug_impl
            patch => $target_mod,
            patches_mut => patches,
            patches, pixels,
        );
    };

    (rails $target_mod:ident) => {
        as_vs_convert!(@plug_impl
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
        impl<'s> super::$trait_mod::AsVsMut for super::$target_mod::PlugMut<'s> {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl<'s> super::$trait_mod::AsVs for super::$target_mod::PlugMut<'s> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }

        impl<'s> super::$trait_mod::AsVs for super::$target_mod::Plug<'s> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_>{
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    };
}
as_vs_convert!(pixel patch);
as_vs_convert!(pixel rail);
as_vs_convert!(pixel nav);
as_vs_convert!(patch nav);
as_vs_convert!(rails nav);

macro_rules! as_vs_builder {
    (
        $trait_mod:ident,
        $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl<'s> super::$trait_mod::AsVs for super::$trait_mod::PlugMut<'s> {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    }
}
as_vs_builder!(pixel, pixels, pixels,);
as_vs_builder!(patch, patches, patches, pixels,);
as_vs_builder!(rail, rails, rails, pixels,);

macro_rules! as_vs_main {
    (
        $trait_mod:ident,
        $fn_mut:ident => $fn_ref:ident,
        $( $field:ident, )+
    ) => {
        impl<'s> super::$trait_mod::AsVsMut for super::core::VSurface {
            fn $fn_mut(&mut self) -> super::$trait_mod::PlugMut<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::PlugMut { $( $field, )+ }
            }
        }

        impl<'s> super::$trait_mod::AsVs for super::core::VSurface {
            fn $fn_ref(&self) -> super::$trait_mod::Plug<'_> {
                let Self { $( $field, )+ .. } = self;
                super::$trait_mod::Plug { $( $field, )+ }
            }
        }
    }
}
as_vs_main!(
    pixel,
    pixels_mut => pixels,
    pixels,
);
as_vs_main!(
    patch,
    patches_mut => patches,
    patches, pixels,
);
as_vs_main!(
    rail,
    rails_mut => rails,
    rails, pixels,
);
as_vs_main!(
    nav,
    nav_mut => nav,
    rails, patches, pixels,
);
