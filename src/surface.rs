use crate::LOCALE;
use num_format::ToFormattedString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct ImageArray {
    buffer: Vec<Pixel>,
    width: u32,
    height: u32,
}

#[derive(Clone)]
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

    pub fn lua_resource_name(&self) -> &str {
        match self {
            Pixel::Iron => "iron-ore",
            Pixel::Copper => "copper-ore",
            Pixel::Stone => "stone",
            Pixel::Coal => "coal",
            Pixel::Uranium => "uranium-ore",
            Pixel::Water => "water",
            Pixel::CrudeOil => "crude-oil",
            //
            Pixel::Empty => "loop-empty",
            Pixel::EdgeWall => "loop-edge",
        }
    }

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
            Pixel::EdgeWall => [0xAA, 0xAA, 0xAA],
        }
    }
}

const BTYES_PER_PIXEL: usize = 3;

impl ImageArray {
    pub fn new(width: u32, mut height: u32) -> Self {
        height = height + 1;
        let size = width * height * (BTYES_PER_PIXEL as u32);
        let buffer = (0..size).map(|_| Pixel::Empty).collect();
        println!("Image buffer size {}", size.to_formatted_string(&LOCALE));
        ImageArray {
            buffer,
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, pixel: Pixel, x: u32, y: u32) {
        let i: usize = (self.width * y + x).try_into().unwrap();
        let subpixel_i = i * BTYES_PER_PIXEL;
        // println!("size {}", self.buffer.len());
        for n in 0..(BTYES_PER_PIXEL - 1) {
            self.buffer[subpixel_i + n] = pixel.clone();
        }
    }

    pub fn save(&self, path: &Path) {
        println!("Saving image to {}", path.display());
        // let converted = unsafe { self.buffer.align_to::<u8>().1 };
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);

        let mut converted: Vec<u8> = vec![];
        for pixel in &self.buffer {
            // converted.extend_from_slice(&pixel.color());
            writer.write(&pixel.color()).unwrap();
        }
        // let raw = self.buffer.as_slice();
        // let converted = raw as &[u8];
        // let converted = convert(raw);
        // image::save_buffer(
        //     path,
        //     &converted,
        //     self.width,
        //     self.height,
        //     image::ColorType::Rgb8,
        // )
        // .unwrap();

        println!("Saved");
    }
}
