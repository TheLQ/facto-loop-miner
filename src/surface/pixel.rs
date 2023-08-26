use color_space::Rgb;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use opencv::core::{Point3d, Vec3b, VecN};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Pixel {
    Iron,
    Copper,
    Stone,
    Coal,
    Uranium,
    Water,
    CrudeOil,
    //
    Empty,
    EdgeWall,
}

impl Pixel {
    pub fn parse(name: &str) -> Pixel {
        match name {
            "iron-ore" => Pixel::Iron,
            "copper-ore" => Pixel::Copper,
            "stone" => Pixel::Stone,
            "coal" => Pixel::Coal,
            "uranium-ore" => Pixel::Uranium,
            "water" => Pixel::Water,
            "crude-oil" => Pixel::CrudeOil,
            //
            "loop-empty" => Pixel::Empty,
            "loop-edge" => Pixel::EdgeWall,
            _ => panic!("unknown name {}", name),
        }
    }

    // pub fn lua_resource_name(&self) -> &str {
    //     match self {
    //         Pixel::Iron => "iron-ore",
    //         Pixel::Copper => "copper-ore",
    //         Pixel::Stone => "stone",
    //         Pixel::Coal => "coal",
    //         Pixel::Uranium => "uranium-ore",
    //         Pixel::Water => "water",
    //         Pixel::CrudeOil => "crude-oil",
    //         //
    //         Pixel::Empty => "loop-empty",
    //         Pixel::EdgeWall => "loop-edge",
    //     }
    // }

    pub fn color(&self) -> [u8; 3] {
        match self {
            Pixel::Iron => [0x68, 0x82, 0x90],
            Pixel::Copper => [0xc8, 0x62, 0x30],
            Pixel::Stone => [0xb0, 0x98, 0x68],
            Pixel::Coal => [0x5e, 0x62, 0x66],
            Pixel::Uranium => [0x0b, 0x20, 0x00],
            Pixel::Water => [0xFF, 0xFF, 0xFF],
            Pixel::CrudeOil => [0x20, 0x66, 0xFF],
            //
            Pixel::Empty => [0x00, 0x00, 0x00],
            Pixel::EdgeWall => [0xBD, 0x5F, 0x5F],
        }
    }

    /// Because OpenCV is BGR not RGB...
    /// Because OpenCV uses (boost?) Vector not rust Vec
    pub fn color_cv(&self) -> Vec3b {
        let mut rev = self.color();
        rev.reverse();
        Vec3b::from(rev)
    }

    pub fn color_rgb(&self) -> Rgb {
        let color = self.color();
        Rgb::new(
            color[0].try_into().unwrap(),
            color[1].try_into().unwrap(),
            color[2].try_into().unwrap(),
        )
    }
}

pub const LOOKUP_IMAGE_ORDER: [Pixel; 9] = [
    Pixel::Iron,
    Pixel::Copper,
    Pixel::Stone,
    Pixel::Coal,
    Pixel::Uranium,
    Pixel::Water,
    Pixel::CrudeOil,
    //
    Pixel::Empty,
    Pixel::EdgeWall,
];

pub fn generate_lookup_image() {
    let img = image::RgbImage::new(LOOKUP_IMAGE_ORDER.len() as u32, 1);

    let path = Path::new("work/out0/lookup.png");
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(&file);

    let buf: Vec<u8> = LOOKUP_IMAGE_ORDER.iter().flat_map(|e| e.color()).collect();

    let encoder = PngEncoder::new(writer);
    encoder
        .write_image(&buf, LOOKUP_IMAGE_ORDER.len() as u32, 1, ColorType::Rgb8)
        .unwrap();
}
