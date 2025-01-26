use crate::surface::patch::DiskPatch;
use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use crate::PixelKdTree;

#[derive(Default)]
pub struct ResourceCloud {
    pub kdtree: PixelKdTree,
    pixels: Vec<Pixel>,
}

impl ResourceCloud {
    pub fn from_patches(patches: &DiskPatch) -> Self {
        let mut positions: Vec<[f32; 2]> = Vec::new();
        let mut pixels = Vec::new();
        for (pixel, resource_patches) in &patches.patches {
            for resource_patch in resource_patches {
                positions.push(resource_patch.corner_point_slice_f32());
                pixels.push(*pixel);
            }
        }

        ResourceCloud {
            kdtree: (&positions).into(),
            pixels,
        }
    }

    pub fn from_surface(surface: &VSurface) -> Self {
        let positions: Vec<[f32; 2]> = Vec::new();
        let pixels = Vec::new();

        // for patch in surface.get_patches_slice() {
        //     for patch_pixel in &patch.pixel_indexes {
        //         positions.push(patch_pixel.to_slice_f32());
        //         pixels.push(patch.resource);
        //     }
        // }
        // tracing::debug!("built total {}", positions.len());
        // // positions.sort();
        // positions.dedup();

        ResourceCloud {
            kdtree: (&positions).into(),
            pixels,
        }
    }
}
