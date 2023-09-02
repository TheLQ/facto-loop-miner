use crate::state::machine::search_step_history_dirs;
use crate::surface::easybox::EasyBox;
use crate::surface::pixel::Pixel;
use crate::LOCALE;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use opencv::core::{Mat, Point, Range};
use opencv::imgproc::{get_font_scale_from_height, put_text, FONT_HERSHEY_SIMPLEX, LINE_8};
use opencv::prelude::*;
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
        if let Some(existing_pixel) = self.try_set_pixel(pixel.clone(), x, y) {
            println!(
                "[warn] unexpected existing pixel {}x{} data {:?} trying {:?}",
                x, y, existing_pixel, &pixel
            )
        }
    }

    pub fn try_set_pixel(&mut self, pixel: Pixel, x: u32, y: u32) -> Option<&Pixel> {
        let i: usize = (self.width * y + x).try_into().unwrap();
        if self.buffer[i] != Pixel::Empty {
            Some(&self.buffer[i])
        } else {
            self.buffer[i] = pixel;
            None
        }
    }

    pub fn load_from_step_history(step_history_out_dirs: &Vec<PathBuf>) -> Self {
        let recent_surface = search_step_history_dirs(
            step_history_out_dirs.clone().into_iter(),
            "surface-full.png",
        );
        Surface::load(recent_surface.parent().unwrap())
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

    pub fn to_mat(&self) -> Mat {
        let raw_buffer: &[u8] = unsafe { transmute(self.buffer.as_slice()) };
        Mat::from_slice_rows_cols(&raw_buffer, self.height as usize, self.width as usize).unwrap()
    }

    pub fn crop(&self, crop_radius_from_center: i32) -> Self {
        let x_start = self.area_box.absolute_x_i32(-crop_radius_from_center);
        let x_end = self.area_box.absolute_x_i32(crop_radius_from_center);
        let y_start = self.area_box.absolute_y_i32(-crop_radius_from_center);
        let y_end = self.area_box.absolute_y_i32(crop_radius_from_center);

        let img = self.to_mat();
        let mut cropped = img
            .apply(
                Range::new(y_start as i32, y_end as i32).unwrap(),
                Range::new(x_start as i32, x_end as i32).unwrap(),
            )
            .unwrap()
            // clone to new contiguous memory location
            .clone();
        let expected_size = crop_radius_from_center * 2;
        if cropped.rows() != expected_size {
            panic!("expected rows {} got {}", expected_size, cropped.rows());
        } else if cropped.cols() != expected_size {
            panic!("expected cols {} got {}", expected_size, cropped.cols());
        }
        draw_text_cv(&mut cropped, "cv(0,0)", Point::new(100, 100));

        let cropped_buffer: &[Pixel] = unsafe { transmute(cropped.data_bytes().unwrap()) };
        let surface = Surface {
            buffer: Vec::from(cropped_buffer),
            width: cropped.cols() as u32,
            height: cropped.rows() as u32,
            area_box: EasyBox {
                min_x: -crop_radius_from_center,
                max_x: crop_radius_from_center,
                min_y: -crop_radius_from_center,
                max_y: crop_radius_from_center,
                width: (crop_radius_from_center * 2) as u32,
                height: (crop_radius_from_center * 2) as u32,
            },
        };

        if cropped.rows() != surface.height as i32 {
            panic!("expected height {} rows {}", surface.height, cropped.rows());
        } else if cropped.cols() != surface.width as i32 {
            panic!("expected width {} cols {}", surface.width, cropped.cols());
        }

        surface
    }

    pub fn draw_text(&mut self, text: &str, origin: Point) {
        let mut mat = self.to_mat();
        draw_text_cv(&mut mat, text, origin);
        let buf: &[Pixel] = unsafe { transmute(mat.data_bytes().unwrap()) };
        self.buffer = Vec::from(buf);
    }
}

pub fn draw_text_cv(img: &mut Mat, text: &str, origin: Point) {
    put_text(
        img,
        text,
        origin,
        FONT_HERSHEY_SIMPLEX,
        get_font_scale_from_height(FONT_HERSHEY_SIMPLEX, 100, 10).unwrap(),
        Pixel::EdgeWall.scalar_cv(),
        10,
        LINE_8,
        false,
    )
    .unwrap();
}

fn dat_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}raw.dat", NAME_PREFIX))
}

fn meta_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}meta.json", NAME_PREFIX))
}
