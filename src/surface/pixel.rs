use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use opencv::core::{Scalar, Vec3b};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::AsRefStr;

#[derive(Serialize, Deserialize, AsRefStr, Debug, PartialEq, Clone)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
pub enum Pixel {
    IronOre = 25,
    CopperOre = 50,
    Stone = 75,
    Coal = 100,
    UraniumOre = 125,
    Water = 150,
    CrudeOil = 175,
    //
    Empty = 0,
    EdgeWall = 200,
}

impl Pixel {
    pub fn color(&self) -> [u8; 3] {
        match self {
            Pixel::IronOre => [0x68, 0x82, 0x90],
            Pixel::CopperOre => [0xc8, 0x62, 0x30],
            Pixel::Stone => [0xb0, 0x98, 0x68],
            Pixel::Coal => [0x5e, 0x62, 0x66],
            Pixel::UraniumOre => [0x0b, 0x20, 0x00],
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

    pub fn scalar(self) -> Scalar {
        let id = self as u8;
        Scalar::from(id as i32)
    }
}

pub const LOOKUP_IMAGE_ORDER: [Pixel; 9] = [
    Pixel::IronOre,
    Pixel::CopperOre,
    Pixel::Stone,
    Pixel::Coal,
    Pixel::UraniumOre,
    Pixel::Water,
    Pixel::CrudeOil,
    //
    Pixel::Empty,
    Pixel::EdgeWall,
];

pub fn generate_lookup_image() {
    // let img = image::RgbImage::new(LOOKUP_IMAGE_ORDER.len() as u32, 1);

    let path = Path::new("work/out0/lookup.png");
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(&file);

    let buf: [u8; 9] = LOOKUP_IMAGE_ORDER.map(|e| e as u8);
    // let buf: Vec<u8> = LOOKUP_IMAGE_ORDER.iter().flat_map(|e| e.color()).collect();

    let encoder = PngEncoder::new(writer);
    encoder
        .write_image(&buf, LOOKUP_IMAGE_ORDER.len() as u32, 1, ColorType::Rgb8)
        .unwrap();
}
