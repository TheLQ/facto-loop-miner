use crate::surface::pixel::Pixel;
use crate::LOCALE;
use num_format::ToFormattedString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Surface {
    buffer: Vec<Pixel>,
    width: u32,
    height: u32,
}

const BTYES_PER_PIXEL: usize = 3;

impl Surface {
    pub fn new(width: u32, mut height: u32) -> Self {
        height = height + 1;
        let size = width * height * (BTYES_PER_PIXEL as u32);
        let buffer = (0..size).map(|_| Pixel::Empty).collect();
        println!("Image buffer size {}", size.to_formatted_string(&LOCALE));
        Surface {
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
