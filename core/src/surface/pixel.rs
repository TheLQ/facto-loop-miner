use crate::surfacev::err::{VError, VResult};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use opencv::core::{Scalar, Vec3b};
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::IntoEnumIterator;
use strum::{AsRefStr, EnumIter};

#[derive(
    Serialize,
    Deserialize,
    AsRefStr,
    EnumIter,
    Debug,
    PartialEq,
    Clone,
    Eq,
    Hash,
    Copy,
    PartialOrd,
    Ord,
    enum_map::Enum,
)]
#[repr(u8)]
pub enum Pixel {
    Empty = 10,
    IronOre = 20,
    CopperOre = 21,
    Stone = 22,
    Coal = 23,
    UraniumOre = 24,
    Water = 25,
    CrudeOil = 26,
    //
    SteelChest = 70,
    //
    EdgeWall = 200,
    Rail = 225,
    Highlighter = 250,
}

impl Pixel {
    pub fn into_id(self) -> u8 {
        self as u8
    }

    pub fn id(&self) -> u8 {
        *self as u8
    }

    pub fn color(&self) -> [u8; 3] {
        match self {
            Pixel::IronOre => [0x6b, 0x86, 0x94],
            Pixel::CopperOre => [0xce, 0x61, 0x31],
            Pixel::Stone => [0xad, 0x9a, 0x6b],
            Pixel::Coal => [0x5e, 0x62, 0x66],
            Pixel::UraniumOre => [0x0b, 0x20, 0x00],
            Pixel::Water => [0xFF, 0xFF, 0xFF],
            Pixel::CrudeOil => [0x20, 0x66, 0xFF],
            //
            Pixel::SteelChest => [0x5c, 0x60, 0x66], //grey?
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
            //
            "steel-chest" => Ok(Pixel::SteelChest),
            _ => Err(VError::UnknownName {
                name: input.to_string(),
                backtrace: Backtrace::force_capture(),
            }),
        }
    }

    pub fn to_facto_string(&self) -> VResult<&str> {
        match self {
            Pixel::IronOre => Ok("iron-ore"),
            Pixel::CopperOre => Ok("copper-ore"),
            Pixel::Stone => Ok("stone"),
            Pixel::Coal => Ok("coal"),
            Pixel::UraniumOre => Ok("uranium-ore"),
            Pixel::Water => Ok("water"),
            Pixel::CrudeOil => Ok("crude-oil"),
            Pixel::SteelChest => Ok("steel-chest"),
            _ => Err(VError::UnknownName {
                name: self.as_ref().to_string(),
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
        let id = self.into_id();
        Scalar::from(id as i32)
    }

    pub fn nearby_patch_search_distance(&self, search_area: i32) -> i32 {
        match self {
            Pixel::CrudeOil => 300,
            _ => search_area,
        }
    }

    pub const fn is_resource(&self) -> bool {
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

    pub fn iter_resource() -> impl Iterator<Item = Self> {
        Self::iter().filter(Pixel::is_resource)
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

    let buf = LOOKUP_IMAGE_ORDER.map(|e| e.id());
    // let buf: Vec<u8> = LOOKUP_IMAGE_ORDER.iter().flat_map(|e| e.color()).collect();

    let encoder = PngEncoder::new(writer);
    encoder
        .write_image(
            &buf,
            LOOKUP_IMAGE_ORDER.len() as u32,
            1,
            ExtendedColorType::Rgb8,
        )
        .unwrap();
}
