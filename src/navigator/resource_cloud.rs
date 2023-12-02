use crate::surface::patch::DiskPatch;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
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
                pixels.push(pixel.clone());
            }
        }

        ResourceCloud {
            kdtree: (&positions).into(),
            pixels,
        }
    }

    pub fn from_surface(surface: &Surface) -> Self {
        let mut positions: Vec<[f32; 2]> = Vec::new();
        let mut pixels = Vec::new();

        for (i, pixel) in surface.buffer.iter().enumerate() {
            match pixel {
                Pixel::IronOre
                | Pixel::CopperOre
                | Pixel::Stone
                | Pixel::CrudeOil
                | Pixel::Coal
                | Pixel::UraniumOre => {
                    let point = surface.index_to_xy(i);
                    positions.push(point_to_slice_f32(point));
                    pixels.push(pixel.clone());
                }
                _ => {}
            }
        }
        tracing::debug!("built total {}", positions.len());
        // positions.sort();
        positions.dedup();

        ResourceCloud {
            kdtree: (&positions).into(),
            pixels,
        }
    }
}

pub fn point_to_slice_f32(point: PointU32) -> [f32; 2] {
    [point.x as f32, point.y as f32]
}
