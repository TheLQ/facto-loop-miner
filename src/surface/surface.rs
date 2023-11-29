use crate::simd_diff::SurfaceDiff;
use crate::state::machine::search_step_history_dirs;
use crate::surface::game_locator::GameLocator;
use crate::surface::pixel::Pixel;
use crate::{PixelKdTree, LOCALE};
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num_format::ToFormattedString;
use opencv::core::{rotate, Mat, Point, Point_, Range, ROTATE_90_COUNTERCLOCKWISE};
use opencv::imgproc::{get_font_scale_from_height, put_text, FONT_HERSHEY_SIMPLEX, LINE_8};
use opencv::prelude::*;
use serde::{Deserialize, Serialize};
use std::arch::x86_64::__m256i;
use std::fs::{read, write, File};
use std::io::{BufReader, BufWriter};
use std::mem;
use std::mem::transmute;
use std::path::{Path, PathBuf};

pub type PointU32 = Point_<u32>;

#[derive(Serialize, Deserialize)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub area_box: GameLocator,
    #[serde(skip)]
    pub buffer: Vec<Pixel>,
    #[serde(skip)]
    pub collision_mask: Vec<__m256i>,
}

const NAME_PREFIX: &str = "surface-";

impl Surface {
    pub fn new(width: u32, mut height: u32) -> Self {
        height = height + 1;
        let size = width * height;
        let buffer = (0..size).map(|_| Pixel::Empty).collect();
        tracing::debug!("Image buffer size {}", size.to_formatted_string(&LOCALE));
        Surface {
            buffer,
            width,
            height,
            area_box: GameLocator::default(),
            collision_mask: Vec::new(),
        }
    }

    pub fn get_pixel_point_i32(&self, point: Point) -> &Pixel {
        self.get_pixel_i32(point.x, point.y)
    }

    pub fn get_pixel_point_u32(&self, point: &PointU32) -> &Pixel {
        self.get_pixel(point.x, point.y)
    }

    pub fn get_pixel_i32(&self, x: i32, y: i32) -> &Pixel {
        anti_bad_pixel_i32(x, y);
        let i = self.xy_to_index(x as u32, y as u32);
        &self.buffer[i]
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> &Pixel {
        let i = self.xy_to_index(x, y);
        &self.buffer[i]
    }

    pub fn set_pixel_point_u32(&mut self, pixel: Pixel, point: PointU32) {
        self.set_pixel(pixel, point.x, point.y)
    }

    pub fn set_pixel_point_i32(&mut self, pixel: Pixel, point: Point) {
        self.set_pixel_i32(pixel, point.x, point.y)
    }

    pub fn set_pixel_i32(&mut self, pixel: Pixel, x: i32, y: i32) {
        anti_bad_pixel_i32(x, y);
        self.set_pixel(pixel, x as u32, y as u32)
    }

    pub fn set_pixel(&mut self, pixel: Pixel, x: u32, y: u32) {
        let _old = self.replace_pixel(pixel, x, y);
        // if old != Pixel::Empty {
        //     tracing::debug!(
        //         "[warn] unexpected existing pixel {}x{} data {:?} trying {:?}",
        //         x, y, old, pixel
        //     )
        // }
    }

    pub fn replace_pixel(&mut self, pixel: Pixel, x: u32, y: u32) -> Pixel {
        let i = self.xy_to_index(x, y);
        mem::replace(&mut self.buffer[i], pixel)
    }

    pub fn xy_in_range_point_u32(&self, point: &PointU32) -> bool {
        self.xy_in_range(point.x, point.y)
    }

    pub fn xy_in_range(&self, x: u32, y: u32) -> bool {
        if x >= self.width {
            false
        } else if y >= self.height {
            false
        } else {
            true
        }
    }

    pub fn xy_to_index_point_u32(&self, point: PointU32) -> usize {
        self.xy_to_index(point.x, point.y)
    }

    pub fn xy_to_index(&self, x: u32, y: u32) -> usize {
        if x > self.width {
            panic!("width {} x {}", self.width, x);
        } else if y > self.height {
            panic!("height {} y {}", self.height, y);
        }
        (self.width * y + x).try_into().unwrap()
    }

    pub fn index_to_xy(&self, i: usize) -> PointU32 {
        let y = i as u32 / self.width;
        let x = i as u32 % self.width;
        if self.xy_to_index(x, y) != i {
            panic!("unexpected {}x{} for {}", x, y, i);
        }
        PointU32 { x, y }
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
        tracing::debug!("read buffer from {}", &dat_path.display());

        // compress [15,0,99,0] to 0b1010

        surface.buffer = unsafe { transmute(buffer) };
        surface
    }

    pub fn load_meta(out_dir: &Path) -> Self {
        let meta_path = meta_path(&out_dir);
        let meta_reader = BufReader::new(File::open(&meta_path).unwrap());
        let surface: Surface = simd_json::serde::from_reader(meta_reader).unwrap();
        tracing::debug!("read size from {}", &meta_path.display());

        surface
    }

    pub fn save(&self, out_dir: &Path) {
        tracing::debug!("Saving RGB dump image to {}", out_dir.display());
        if !out_dir.is_dir() {
            panic!("dir does not exist {}", out_dir.display());
        }

        self.save_raw(out_dir);
        self.save_colorized(out_dir, NAME_PREFIX);
    }

    fn save_raw(&self, out_dir: &Path) {
        let bytes: &Vec<u8> = unsafe { transmute(&self.buffer) };

        let dat_path = dat_path(&out_dir);
        tracing::debug!("writing to {}", dat_path.display());
        write(&dat_path, bytes).unwrap();

        let meta_path = meta_path(&out_dir);
        tracing::debug!("writing to {}", &meta_path.display());
        let meta_writer = BufWriter::new(File::create(&meta_path).unwrap());
        simd_json::serde::to_writer(meta_writer, self).unwrap();
    }

    fn save_colorized(&self, out_dir: &Path, name_prefix: &str) {
        let mut output: Vec<u8> = vec![0; self.buffer.len() * 3];
        for (i, pixel) in self.buffer.iter().enumerate() {
            let color = &pixel.color();
            let start = i * color.len();
            output[start] = color[0];
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
        write(path, rgb).unwrap();

        tracing::debug!(
            "Saved {} byte RGB array to {}",
            rgb.len().to_formatted_string(&LOCALE),
            path.display(),
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
        tracing::debug!(
            "Saved {} byte image to {}",
            size.to_formatted_string(&LOCALE),
            path.display(),
        );
    }

    pub fn get_buffer_to_cv(&self) -> Mat {
        let raw_buffer: &[u8] = unsafe { transmute(self.buffer.as_slice()) };
        Mat::from_slice_rows_cols(&raw_buffer, self.height as usize, self.width as usize).unwrap()
    }

    pub fn slice_u8_to_pixel(img: &[u8]) -> &[Pixel] {
        unsafe { transmute(img) }
    }

    pub fn set_buffer_from_cv(&mut self, img: Mat) {
        let buf: &[Pixel] = Surface::slice_u8_to_pixel(img.data_bytes().unwrap());
        self.buffer = Vec::from(buf);
    }

    pub fn surface_diff(&self) -> SurfaceDiff {
        SurfaceDiff::from_surface(self)
    }

    pub fn crop(&self, crop_radius_from_center: i32) -> Self {
        let x_start = self.area_box.game_centered_x_i32(-crop_radius_from_center);
        let x_end = self.area_box.game_centered_x_i32(crop_radius_from_center);
        let y_start = self.area_box.game_centered_y_i32(-crop_radius_from_center);
        let y_end = self.area_box.game_centered_y_i32(crop_radius_from_center);

        let img = self.get_buffer_to_cv();
        let cropped = img
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
        let crop_box = GameLocator {
            min_x: -crop_radius_from_center,
            max_x: crop_radius_from_center,
            min_y: -crop_radius_from_center,
            max_y: crop_radius_from_center,
            width: (crop_radius_from_center * 2) as u32,
            height: (crop_radius_from_center * 2) as u32,
        };
        if cropped.rows() != crop_box.height as i32 {
            panic!(
                "expected height {} rows {}",
                crop_box.height,
                cropped.rows()
            );
        } else if cropped.cols() != crop_box.width as i32 {
            panic!("expected width {} cols {}", crop_box.width, cropped.cols());
        }

        // draw_text_cv(&mut cropped, "cv(0,0)", Point::new(0, 100));
        // let rows = cropped.rows();
        // draw_text_cv(&mut cropped, "cv(0,1)", Point::new(0, rows - 50));
        // draw_text_cv(
        //     &mut cropped,
        //     "g(-1,0)",
        //     Point {
        //         x: crop_box.absolute_x_i32(-crop_radius_from_center) as i32,
        //         y: crop_box.absolute_y_i32(0) as i32,
        //     },
        // );
        // draw_text_cv(
        //     &mut cropped,
        //     "g(0,1)",
        //     Point {
        //         x: crop_box.absolute_x_i32(0) as i32,
        //         y: crop_box.absolute_y_i32(crop_radius_from_center - 50) as i32,
        //     },
        // );

        let mut surface = Surface {
            buffer: Vec::new(),
            width: cropped.cols() as u32,
            height: cropped.rows() as u32,
            area_box: crop_box,
            collision_mask: Vec::new(),
        };
        surface.set_buffer_from_cv(cropped);

        surface
    }

    pub fn draw_text(&mut self, text: &str, origin: Point) {
        let mut mat = self.get_buffer_to_cv();
        draw_text_cv(&mut mat, text, origin);
        self.set_buffer_from_cv(mat)
    }

    pub fn pixel_to_kdtree(&self, filter_pixel: &Pixel) -> PixelKdTree {
        let mut added: Vec<[f32; 2]> = Vec::new();
        for (pos, cur_pixel) in self.buffer.iter().enumerate() {
            if cur_pixel == filter_pixel {
                let pos = self.index_to_xy(pos);
                added.push([pos.x as f32, pos.y as f32]);
            }
        }
        (&added).into()
    }

    pub fn draw_square(&mut self, pixel: &Pixel, square_size: usize, origin: &PointU32) {
        for i in 0..square_size {
            for j in 0..square_size {
                self.set_pixel(pixel.clone(), origin.x + i as u32, origin.y + j as u32);
            }
        }
    }
}

fn anti_bad_pixel_i32(x: i32, y: i32) {
    if x < 0 {
        panic!("x < 0");
    } else if y < 0 {
        panic!("y < 0");
    }
}

pub fn draw_text_cv(img: &mut Mat, text: &str, origin: Point) {
    tracing::debug!("drawing {} at {:?}", text, origin);
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

pub fn draw_text_vertical_cv(_img: &mut Mat, text: &str, origin: Point) {
    tracing::debug!("drawing {} at {:?}", text, origin);
    // "cv(0,0)" is roughly 500x150
    let mut text_img = unsafe { Mat::new_rows_cols(500, 1000, 0).unwrap() };
    put_text(
        &mut text_img,
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

    let mut rotated_text_img = unsafe { Mat::new_rows_cols(1000, 500, 0).unwrap() };
    rotate(&text_img, &mut rotated_text_img, ROTATE_90_COUNTERCLOCKWISE).unwrap();
}

fn dat_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}raw.dat", NAME_PREFIX))
}

fn meta_path(out_dir: &Path) -> PathBuf {
    out_dir.join(format!("{}meta.json", NAME_PREFIX))
}
