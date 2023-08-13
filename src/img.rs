use crate::LOCALE;
use num_format::ToFormattedString;
use std::path::Path;

pub struct ImageArray {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}

const BTYES_PER_PIXEL: u8 = 3;

// const BACKGROUND_COLOR: u32 = 0x3B4F3CFF;
const BACKGROUND_COLOR: u32 = 0x00000040;

impl ImageArray {
    pub fn new(width: u32, mut height: u32) -> Self {
        height = height + 1;
        let size = width * height * (BTYES_PER_PIXEL as u32);
        let buffer: Vec<u8> = (0..size).map(|_| 0x00).collect();
        println!("Image buffer size {}", size.to_formatted_string(&LOCALE));
        ImageArray {
            buffer,
            width,
            height,
        }
    }

    pub fn set_color(&mut self, color: u32, x: u32, y: u32) {
        let pieces: [u8; 4] = color.to_be_bytes();

        let i: usize = (self.width * y + x).try_into().unwrap();
        let subpixel_i = i * (BTYES_PER_PIXEL as usize);
        // println!("size {}", self.buffer.len());
        for n in 0..((BTYES_PER_PIXEL - 1) as usize) {
            self.buffer[subpixel_i + n] = pieces[n];
        }
    }

    pub fn save(self, path: &Path) {
        println!("Saving image to {}", path.display());
        // let converted = unsafe { self.buffer.align_to::<u8>().1 };
        let converted = &self.buffer;
        // let raw = self.buffer.as_slice();
        // let converted = raw as &[u8];
        // let converted = convert(raw);
        image::save_buffer(
            path,
            converted,
            self.width,
            self.height,
            image::ColorType::Rgb8,
        )
        .unwrap();
        println!("Saved");
    }
}

pub fn color_for_resource(resource_name: &str) -> Option<u32> {
    return match resource_name {
        "iron-ore" => Some(0x68829040),
        "copper-ore" => Some(0xc8623040),
        "stone" => Some(0xb0986840),
        "coal" => Some(0x00000040),
        "uranium-ore" => Some(0x0b20040),
        "water" => Some(0xFFFFFF40),
        _ => None,
    };
}

fn color_from_hex(hex_code: &str) -> u32 {
    let hex_without_hash = &hex_code[1..];
    // println!("testing {}", hex_without_hash);
    u32::from_str_radix(hex_without_hash, 16).unwrap()
}
