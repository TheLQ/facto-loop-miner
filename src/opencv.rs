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
    load_cv_from_path_filtered(
        path,
        surface_meta.height as usize,
        surface_meta.width as usize,
        pixel_opt,
    )
}

pub fn load_cv_from_path_filtered(
    path: &Path,
    rows: usize,
    columns: usize,
    filter: Option<&Pixel>,
) -> Mat {
    let mut surface_raw = read(path).unwrap();
    load_cv_from_buffer_filtered(&mut surface_raw, rows, columns, filter)
}

pub fn load_cv_from_buffer_filtered(
    buffer: &mut [u8],
    rows: usize,
    columns: usize,
    filter: Option<&Pixel>,
) -> Mat {
    if let Some(pixel) = filter {
        let pixel_id = pixel.clone() as u8;
        // let mut found_ids: Vec<u8> = Vec::new();
        for pixel_raw in buffer.iter_mut() {
            // if !found_ids.contains(pixel_raw) {
            //     tracing::debug("found {}", pixel_raw);
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
            surface_meta.columns as i32,
            0,
            state_ptr,
            0,
        )
    }
    .unwrap();*/
    // let img = imread(surface_raw_path.as_os_str().to_str().unwrap(), 0).unwrap();
    load_cv_from_buffer(buffer, rows, columns)
}

pub fn load_cv_from_buffer(buffer: &[u8], rows: usize, columns: usize) -> Mat {
    Mat::from_slice_rows_cols(&buffer, rows, columns).unwrap()
}

pub fn load_raw_image_from_slice(surface_meta: &Surface, raw: &[u8]) -> Mat {
    load_cv_from_buffer(
        raw,
        surface_meta.height as usize,
        surface_meta.width as usize,
    )
}
