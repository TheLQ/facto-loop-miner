use crate::state::tuneables::Tunables;
use crate::surfacev::mine::MinePath;
use crate::surfacev::vpatch::VPatch;
use facto_loop_miner_io::varray::VArray;

struct Tape {}

//

struct TapeSurface {
    pixels: TapeMap,
    patches: Vec<VPatch>,
    rail_paths: Vec<MinePath>,
    tunables: Tunables,
    dirty: bool,
}

// #[derive(bitcode::Encode)]
struct TapeMap {
    entities: Vec<TapePixel>,
    /// More efficient to store a (radius * 2)^2 length Array as a raw file instead of JSON
    xy_to_entity: VArray,
    /// A *square* centered on 0,0
    radius: u32,
}

pub enum TapePixel {
    RailPath,
    Copper,
    Coal,
}

//

pub struct TapeChanger<'s>(&'s mut TapeSurface);

impl TapeChanger<'_> {
    fn post(self) {
        self.0.dirty = true;
    }

    fn init_patches(self) {
        self.post()
    }

    fn add_entity(self) {
        self.post()
    }

    fn add_rail_path(self) {
        self.post()
    }
}

pub struct TapeGet<'s>(&'s TapeSurface);

impl<'s> TapeGet<'s> {
    pub fn rail_paths(self) {}

    pub fn tuneables(self) -> &'s Tunables {
        &self.0.tunables
    }
}
