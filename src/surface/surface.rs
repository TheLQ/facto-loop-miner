use crate::surface::pixel::Pixel;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::Locale::da;
use num_format::ToFormattedString;
use std::fs;
use std::fs::{read, write, File};
use std::io::BufWriter;
use std::mem::transmute;
use std::path::{Path, PathBuf};

pub struct Surface {
    buffer: Vec<Pixel>,
    width: u32,
    height: u32,
}

const NAME_PREFIX: &str = "surface-";

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

    pub fn load(out_dir: &Path) -> Self {
        let dat_path = dat_path(&out_dir);
        let buffer = read(&dat_path).unwrap();
        println!("read buffer from {}", &dat_path.display());

        let size_path = size_path(&out_dir);
        let size_raw = read(&size_path).unwrap();
        let size = u32::from_le_bytes(size_raw[0..4].try_into().unwrap());
        println!("read size from {}", &size_path.display());

        Surface {
            buffer: unsafe { transmute(buffer) },
            width: size,
            height: size,
        }
    }

    pub fn save(&self, out_dir: &Path) {
        println!("Saving RGB dump image to {}", out_dir.display());
        if !out_dir.is_dir() {
            panic!("dir does not exist {}", out_dir.display());
        }

        self.save_raw(out_dir, NAME_PREFIX);
        self.save_colorized(out_dir, NAME_PREFIX);
    }

    fn save_raw(&self, out_dir: &Path, name_prefix: &str) {
        let bytes: &Vec<u8> = unsafe { transmute(&self.buffer) };

        let dat_path = dat_path(&out_dir);
        write(&dat_path, bytes).unwrap();
        println!("wrote to {}", dat_path.display());

        // if self.width != self.height {
        //     panic!("unexpected size {} x {}", self.width, self.height)
        // }
        let size_path = size_path(&out_dir);
        write(&size_path, self.width.to_le_bytes()).unwrap();
        println!("wrote to {}", size_path.display());
    }

    fn save_colorized(&self, out_dir: &Path, name_prefix: &str) {
        let mut output: Vec<u8> = vec![0; self.buffer.len() * 3];
        for (i, pixel) in self.buffer.iter().enumerate() {
            let color = &pixel.color();
            let start = i * color.len();
            output[start + 0] = color[0];
            output[start + 1] = color[1];
            output[start + 2] = color[2];
        }

        // self.save_rgb(
        //     &output,
        //     &out_dir
        //         .to_path_buf()
        //         .with_file_name(name_prefix.clone() + ".rgb"),
        // );

        self.save_png(&output, &out_dir.join(format!("{}full.png", name_prefix)));
    }

    fn save_rgb(&self, rgb: &[u8], path: &Path) {
        fs::write(path, rgb).unwrap();

        println!(
            "Saved {} byte RGB array to {}",
            rgb.len().to_formatted_string(&LOCALE),
            path.display()
        );
    }

    fn save_png(&self, rgb: &[u8], path: &Path) {
        let file = File::create(path).unwrap();
        let writer = BufWriter::new(&file);

        let encoder = PngEncoder::new(writer);
        encoder
            .write_image(&rgb, self.width, self.height, ColorType::Rgb8)
            .unwrap();
        let size = file.metadata().unwrap().len();
        println!(
            "Saved {} byte image to {}",
            size.to_formatted_string(&LOCALE),
            path.display()
        );
    }
}

fn dat_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}raw.dat", NAME_PREFIX))
}

fn size_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}size.txt", NAME_PREFIX))
}
