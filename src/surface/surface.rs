use crate::surface::pixel::Pixel;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Surface {
    buffer: Vec<Pixel>,
    width: u32,
    height: u32,
}

impl Surface {
    pub fn new(width: u32, mut height: u32) -> Self {
        height = height + 1;
        let size = width * height;
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
        if self.buffer[i] != Pixel::Empty {
            let existing_pixel = self.buffer[i].clone();
            println!(
                "[warn] unexpected existing pixel {}x{} data {:?} trying {:?}",
                x, y, existing_pixel, pixel
            )
        }
        self.buffer[i] = pixel.clone();
    }

    pub fn save(&self, path: &Path) {
        println!("Saving RGB dump image to {}", path.display());
        let mut output: Vec<u8> = vec![0; self.buffer.len() * 3];
        for (i, pixel) in self.buffer.iter().enumerate() {
            let color = &pixel.color();
            let start = i * color.len();
            output[start + 0] = color[0];
            output[start + 1] = color[1];
            output[start + 2] = color[2];
        }

        let file = File::create(path).unwrap();
        let writer = BufWriter::new(file);

        let encoder = PngEncoder::new(writer);
        encoder
            .write_image(&output, self.width, self.height, ColorType::Rgb8)
            .unwrap();

        // for pixel in &self.buffer {
        //     // converted.extend_from_slice(&pixel.color());
        //     writer.write(&pixel.color()).unwrap();
        // }
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
    }
}
