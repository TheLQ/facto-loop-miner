use crate::surface::pixel::Pixel;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::opencv_re::core::{Mat, Point, Rect, Vector};
use facto_loop_miner_fac_engine::opencv_re::imgproc::bounding_rect;
use std::fs::read;
use std::path::Path;
use tracing::debug;

pub fn load_raw_image_with_surface(
    path: &Path,
    surface_meta: &VSurface,
    pixel_opt: Option<&Pixel>,
) -> Mat {
    let side_length = surface_meta.get_diameter();
    debug!("side length {}", side_length);
    debug!("Loading {}", surface_meta);
    debug!("path {}", path.display());
    load_cv_from_path_filtered(path, side_length, side_length, pixel_opt)
}

fn load_cv_from_path_filtered(
    path: &Path,
    rows: usize,
    columns: usize,
    filter: Option<&Pixel>,
) -> Mat {
    let mut surface_raw = read(path).unwrap();
    load_cv_from_buffer_filtered(&mut surface_raw, rows, columns, filter)
}

fn load_cv_from_buffer_filtered(
    buffer: &mut [u8],
    rows: usize,
    columns: usize,
    filter: Option<&Pixel>,
) -> Mat {
    debug!("buffer {}", buffer.len());
    if let Some(pixel) = filter {
        let pixel_id = pixel.id();
        // let mut found_ids: Vec<u8> = Vec::new();
        for pixel_raw in buffer.iter_mut() {
            // if !found_ids.contains(pixel_raw) {
            //     tracing::debug!("found {}", pixel_raw);
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

fn load_cv_from_buffer(buffer: &[u8], rows: usize, columns: usize) -> Mat {
    Mat::from_slice_rows_cols(buffer, rows, columns).unwrap()
}

fn load_raw_image_from_slice(surface_meta: &VSurface, raw: &[u8]) -> Mat {
    load_cv_from_buffer(
        raw,
        surface_meta.get_diameter(),
        surface_meta.get_diameter(),
    )
}

pub fn get_cv_bounding_rect(points: Vec<Point>) -> Rect {
    bounding_rect(&Vector::from_slice(&points)).unwrap()
}

pub fn combine_rects_into_big_rect<'a>(rects: impl IntoIterator<Item = &'a Rect>) -> Rect {
    let mut corners: Vec<Point> = Vec::new();
    for nearby_rect in rects {
        corners.push(VPoint::new(nearby_rect.x, nearby_rect.y).to_cv_point());
        corners.push(
            VPoint::new(
                nearby_rect.x + nearby_rect.width,
                nearby_rect.y + nearby_rect.height,
            )
            .to_cv_point(),
        );
    }
    get_cv_bounding_rect(corners)
}
