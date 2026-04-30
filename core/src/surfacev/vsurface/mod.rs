mod convert;
mod core;
// mod core_plugs;
mod mine;
mod nav;
mod patch;
mod pixel;
mod rail;

pub use core::VSurface;
// pub use core_plugs::{
//     //
//     AsVs as VSurfaceCoreAsVs,
//     AsVsMut as VSurfaceCoreAsVsMut,
//     Plug as VSurfaceCore,
//     PlugMut as VSurfaceCoreMut,
// };
pub use mine::{
    //
    AsVs as VSurfaceMineAsVs,
    AsVsMut as VSurfaceMineAsVsMut,
    Plug as VSurfaceMine,
    PlugMut as VSurfaceMineMut,
};
pub use nav::{
    //
    AsVs as VSurfaceNavAsVs,
    AsVsMut as VSurfaceNavAsVsMut,
    Plug as VSurfaceNav,
    PlugMut as VSurfaceNavMut,
};
pub use patch::{
    //
    AsVs as VSurfacePatchAsVs,
    AsVsMut as VSurfacePatchAsVsMut,
    Plug as VSurfacePatch,
    PlugMut as VSurfacePatchMut,
};
pub use pixel::{
    //
    AsVs as VSurfacePixelAsVs,
    AsVsMut as VSurfacePixelAsVsMut,
    Plug as VSurfacePixel,
    PlugMut as VSurfacePixelMut,
};
pub use rail::{
    //
    AsVs as VSurfaceRailAsVs,
    AsVsMut as VSurfaceRailAsVsMut,
    Plug as VSurfaceRail,
    PlugMut as VSurfaceRailMut,
};

pub use mine::MineRef;
pub use patch::PatchRef;
