use crate::opencv::draw_text_cv;
use crate::surfacev::vsurface::VSurfacePixel;
use facto_loop_miner_fac_engine::opencv_re::core::{CV_8U, Mat, Point, Scalar, Size, Vector};
use facto_loop_miner_fac_engine::opencv_re::imgcodecs::imwrite;

pub fn pixel_widget_main() {}

fn pixel_widget(pixels: VSurfacePixel) {
    let size = pixels.get_diameter() as i32;
    let mut mat = Mat::new_size_with_default(
        Size {
            height: size,
            width: size,
        },
        CV_8U,
        Scalar::all(0.0),
    )
    .unwrap();
    draw_text_cv(
        &mut mat,
        "hello",
        Point { x: 0, y: 0 },
        Scalar::all(1.0),
        10,
        3,
    );
    imwrite("output.png", &mat, &Vector::new()).unwrap();
}
