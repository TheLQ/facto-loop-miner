use crate::state::machine::{Step, StepParams};
use crate::surface::surface::Surface;
use opencv::core::Vector;
use opencv::imgcodecs::{imread, imwrite};
use opencv::prelude::*;
use std::ffi::c_void;
use std::fs::read;

pub struct Step04 {}

impl Step04 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step04 {})
    }
}

impl Step for Step04 {
    fn name(&self) -> String {
        "step04-contours".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let previous_step_dir = params.step_history_out_dirs.last().unwrap();

        let surface_meta = Surface::load_meta(&previous_step_dir);

        let surface_raw_path = previous_step_dir.join("surface-raw.dat");
        println!("Loading {}", surface_raw_path.display());
        let mut surface_raw = read(&surface_raw_path).unwrap();
        println!("size {}", surface_raw.len());

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
        let img = Mat::from_slice_rows_cols(
            &surface_raw,
            surface_meta.height as usize,
            surface_meta.width as usize,
        )
        .unwrap();

        // let img = imread(surface_raw_path.as_os_str().to_str().unwrap(), 0).unwrap();
        let size = img.size().unwrap();
        println!(
            "Read {} size {}x{} type {}",
            surface_raw_path.display(),
            size.width,
            size.height,
            img.typ()
        );

        imwrite(
            params.step_out_dir.join("cv.png").to_str().unwrap(),
            &img,
            &Vector::new(),
        )
        .unwrap();
    }
}
