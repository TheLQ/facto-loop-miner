use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use opencv::core::Mat;
use std::fs::read;
use std::path::Path;

pub fn load_raw_image_with_surface(
    path: &Path,
    surface_meta: &Surface,
    pixel_opt: Option<&Pixel>,
) -> Mat {
    load_raw_image(
        path,
        surface_meta.height as usize,
        surface_meta.width as usize,
        pixel_opt,
    )
}

pub fn load_raw_image(path: &Path, rows: usize, height: usize, pixel_opt: Option<&Pixel>) -> Mat {
    let mut surface_raw = read(path).unwrap();

    if let Some(pixel) = pixel_opt {
        let pixel_id = pixel.clone() as u8;
        // let mut found_ids: Vec<u8> = Vec::new();
        for pixel_raw in surface_raw.iter_mut() {
            // if !found_ids.contains(pixel_raw) {
            //     println!("found {}", pixel_raw);
            //     found_ids.push(pixel_raw.clone());
            // }
            if pixel_id != *pixel_raw {
                *pixel_raw = 0;
            }
        }
    }

    /*let img = unsafe {
        let state_ptr: *mut c_void = &mut surface_raw as *mut _ as *mut c_void;
        Mat::new_rows_cols_with_data(
            surface_meta.width as i32,
            surface_meta.height as i32,
            0,
            state_ptr,
            0,
        )
    }
    .unwrap();*/
    // let img = imread(surface_raw_path.as_os_str().to_str().unwrap(), 0).unwrap();
    Mat::from_slice_rows_cols(&surface_raw, rows, height).unwrap()
}

pub fn load_raw_image_from_slice(surface_meta: &Surface, raw: &[u8]) -> Mat {
    Mat::from_slice_rows_cols(
        raw,
        surface_meta.height as usize,
        surface_meta.width as usize,
    )
    .unwrap()
}
