use crate::surfacev::err::{VError, VResult};
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use opencv::core::{Scalar, Vec3b};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::{AsRefStr, EnumIter};

#[derive(Serialize, Deserialize, AsRefStr, EnumIter, Debug, PartialEq, Clone, Eq, Hash)]
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
    Rail = 225,
    Highlighter = 250,
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
            Pixel::Rail => [0xB9, 0x7A, 0x57],
            Pixel::Highlighter => [0x3C, 0xB6, 0xDE],
        }
    }

    pub fn from_string(input: &str) -> VResult<Self> {
        match input {
            "iron-ore" => Ok(Pixel::IronOre),
            "copper-ore" => Ok(Pixel::CopperOre),
            "stone" => Ok(Pixel::Stone),
            "coal" => Ok(Pixel::Coal),
            "uranium-ore" => Ok(Pixel::UraniumOre),
            "water" => Ok(Pixel::Water),
            "crude-oil" => Ok(Pixel::CrudeOil),
            _ => Err(VError::UnknownName {
                name: input.to_string(),
                backtrace: Backtrace::force_capture(),
            }),
        }
    }

    /// Because OpenCV is BGR not RGB...
    /// Because OpenCV uses (boost?) Vector not rust Vec
    pub fn color_cv(&self) -> Vec3b {
        let mut rev = self.color();
        rev.reverse();
        Vec3b::from(rev)
    }

    pub fn scalar_cv(self) -> Scalar {
        let id = self as u8;
        Scalar::from(id as i32)
    }

    pub fn nearby_patch_search_distance(&self, search_area: i32) -> i32 {
        match self {
            Pixel::CrudeOil => 300,
            _ => search_area,
        }
    }

    pub fn is_resource(&self) -> bool {
        matches!(
            self,
            Pixel::IronOre
                | Pixel::CopperOre
                | Pixel::Stone
                | Pixel::CrudeOil
                | Pixel::Coal
                | Pixel::UraniumOre
        )
    }
}

pub const LOOKUP_IMAGE_ORDER: [Pixel; 10] = [
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
    Pixel::Rail,
];

pub fn generate_lookup_image() {
    // let img = image::RgbImage::new(LOOKUP_IMAGE_ORDER.len() as u32, 1);

    let path = Path::new("work/out0/lookup.png");
    let file = File::create(path).unwrap();
    let writer = BufWriter::new(&file);

    let buf = LOOKUP_IMAGE_ORDER.map(|e| e as u8);
    // let buf: Vec<u8> = LOOKUP_IMAGE_ORDER.iter().flat_map(|e| e.color()).collect();

    let encoder = PngEncoder::new(writer);
    encoder
        .write_image(&buf, LOOKUP_IMAGE_ORDER.len() as u32, 1, ColorType::Rgb8)
        .unwrap();
}
