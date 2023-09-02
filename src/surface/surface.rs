use crate::surface::easybox::EasyBox;
use crate::surface::pixel::Pixel;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{read, write, File};
use std::io::{BufReader, BufWriter};
use std::mem::transmute;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Surface {
    #[serde(skip)]
    buffer: Vec<Pixel>,
    pub width: u32,
    pub height: u32,
    pub area_box: EasyBox,
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
            area_box: EasyBox::default(),
        }
    }

    pub fn set_pixel(&mut self, pixel: Pixel, x: u32, y: u32) {
        let i: usize = (self.width * y + x).try_into().unwrap();
        if self.buffer[i] != Pixel::Empty {
            println!(
                "[warn] unexpected existing pixel {}x{} data {:?} trying {:?}",
                x, y, self.buffer[i], pixel
            )
        }
        self.buffer[i] = pixel;
    }

    #[allow(dead_code)]
    pub fn load(out_dir: &Path) -> Self {
        let mut surface = Surface::load_meta(out_dir);

        let dat_path = dat_path(&out_dir);
        let buffer = read(&dat_path).unwrap();
        println!("read buffer from {}", &dat_path.display());

        surface.buffer = unsafe { transmute(buffer) };
        surface
    }

    pub fn load_meta(out_dir: &Path) -> Self {
        let meta_path = meta_path(&out_dir);
        let meta_reader = BufReader::new(File::open(&meta_path).unwrap());
        let surface: Surface = simd_json::serde::from_reader(meta_reader).unwrap();
        println!("read size from {}", &meta_path.display());

        surface
    }

    pub fn save(&self, out_dir: &Path) {
        println!("Saving RGB dump image to {}", out_dir.display());
        if !out_dir.is_dir() {
            panic!("dir does not exist {}", out_dir.display());
        }

        self.save_raw(out_dir);
        self.save_colorized(out_dir, NAME_PREFIX);
    }

    fn save_raw(&self, out_dir: &Path) {
        let bytes: &Vec<u8> = unsafe { transmute(&self.buffer) };

        let dat_path = dat_path(&out_dir);
        println!("writing to {}", dat_path.display());
        write(&dat_path, bytes).unwrap();

        let meta_path = meta_path(&out_dir);
        println!("writing to {}", &meta_path.display());
        let meta_writer = BufWriter::new(File::create(&meta_path).unwrap());
        simd_json::serde::to_writer(meta_writer, self).unwrap();
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

    #[allow(dead_code)]
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

fn meta_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}meta.json", NAME_PREFIX))
}
