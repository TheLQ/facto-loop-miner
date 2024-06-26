use crate::surface::patch::{DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
// use opencv::core::{Range, Vector};
// use opencv::imgcodecs::imwrite;
// use opencv::prelude::*;
use std::path::Path;

pub fn get_patch_main() {
    let mut surface = Surface::load(Path::new("work/out0/step03-crop"));
    let patches = DiskPatch::load_from_dir(Path::new("work/out0/step04-contours"));

    let first_patch: &Patch = patches.patches[&Pixel::IronOre]
        .iter()
        .find(|v| v.height + v.width > 50)
        .unwrap();
    tracing::debug!("dumping {:?}", first_patch);

    let img = surface.get_buffer_to_cv();
    // TODO: doesn't compile anymore???
    // img = img
    //     .apply(
    //         Range::new(first_patch.y, first_patch.y + first_patch.height).unwrap(),
    //         Range::new(first_patch.x, first_patch.x + first_patch.width).unwrap(),
    //     )
    //     .unwrap()
    //     // clone to new contiguous memory location
    //     .clone();
    // imwrite("work/test3/inner.png", &img, &Vector::new()).unwrap();

    surface.set_buffer_from_cv(img);

    let out = Path::new("work/test3");
    surface.save(out);
}
