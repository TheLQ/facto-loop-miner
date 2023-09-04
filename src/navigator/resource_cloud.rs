use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::PixelKdTree;

#[derive(Default)]
pub struct ResourceCloud {
    pub kdtree: PixelKdTree,
    pixels: Vec<Pixel>,
}

impl ResourceCloud {
    pub fn new(surface: &Surface) -> Self {
        let mut positions: Vec<[f32; 2]> = Vec::new();
        let mut pixels = Vec::new();

        for (i, pixel) in surface.buffer.iter().enumerate() {
            match pixel {
                Pixel::IronOre
                // | Pixel::CopperOre
                // | Pixel::Stone
                // | Pixel::CrudeOil
                // | Pixel::Coal
                // | Pixel::UraniumOre

                => {
                    let point = surface.index_to_xy(i);
                    positions.push([point.x as f32, point.y as f32]);
                    pixels.push(pixel.clone());
                }
                _ => {}
            }
        }
        println!("built total {}", positions.len());
        // positions.sort();
        positions.dedup();

        ResourceCloud {
            kdtree: (&positions).into(),
            pixels,
        }
    }
}
