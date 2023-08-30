use crate::surface::pixel::Pixel;
use opencv::core::{in_range, Point, Scalar, Vector};
use opencv::imgcodecs::{imread, imwrite, IMREAD_COLOR};
use opencv::imgproc::{
    bounding_rect, find_contours, rectangle, CHAIN_APPROX_SIMPLE, LINE_8, RETR_EXTERNAL,
};
use opencv::prelude::*;
use std::path::Path;

pub fn start(work_dir: &Path) {
    let out_dir = work_dir.join("out0");

    let mut current_image = read_image(&out_dir.join("step01-base.png"), 0);
    for i in 0..2 {
        current_image = match i {
            0 => filter_iron(current_image),
            1 => find_objects(current_image),
            _ => panic!("uhhh"),
        };
        println!("step {} done", i);
    }

    write_image(&work_dir.join("opencv.png"), &current_image);

    // build_opencv_lookup(&out_dir, 0);
    // load_and_AND_rgb(&out_dir);

    // let mut contours: Vector<Vector<Point>> = Vector::default();
    // let offset = Point { x: 0, y: 0 };
    // find_contours(
    //     &input_filtered,
    //     &mut contours,
    //     RETR_LIST,
    //     CHAIN_APPROX_NONE,
    //     offset,
    // )
    // .unwrap();

    // threshold(&mst, )
    // cvt_color(mst, COLOR_BGR2GRAY)

    // let rect = min_area_rect()
    // let not_rect = box_points()
}

fn filter_iron(base: Mat) -> Mat {
    // min
    let color = Pixel::IronOre.color_cv();
    // let mut input_filtered = Mat::clone(&base).unwrap();
    let mut input_filtered = unsafe { Mat::new_rows_cols(base.rows(), base.cols(), 0).unwrap() };
    println!("filter_iron...");
    in_range(&base, &color, &color, &mut input_filtered).unwrap();
    input_filtered
}

fn find_objects(mut base: Mat) -> Mat {
    let mut contours: Vector<Vector<Point>> = Vector::default();
    let offset = Point { x: 0, y: 0 };
    // RETR_LIST - May make rectangles inside rectangles, other multiple rectangles at whisps
    // RETR_CCOMP - Same???
    //
    // CHAIN_APPROX_NONE - Stores all points. unnecessary
    // CHAIN_APPROX_SIMPLE - Only store corners, which are the relevant points
    // CHAIN_APPROX_TC89_L1 - Didn't do anything????
    find_contours(
        &base,
        &mut contours,
        RETR_EXTERNAL,
        CHAIN_APPROX_SIMPLE,
        offset,
    )
    .unwrap();

    println!("found contours {}", contours.len());

    // let color = Pixel::EdgeWall.color_cv();
    for contour in contours {
        let rect = bounding_rect(&contour).unwrap();
        rectangle(
            &mut base,
            rect,
            Scalar::from((100f64, 100f64, 100f64, 100f64)),
            2,
            LINE_8,
            0,
        )
        .unwrap();
    }

    // x,y,w,h = cv.boundingRect(cnt)
    // cv.rectangle(img,(x,y),(x+w,y+h),(0,255,0),2)
    base
}

fn read_image(path: &Path, extra_convert_flags: i32) -> Mat {
    let path = path.to_str().unwrap();

    println!("Reading...");
    let img = imread(&path, IMREAD_COLOR | extra_convert_flags).unwrap();
    let size = img.size().unwrap();
    println!(
        "Read {} size {}x{} type {}",
        path,
        size.width,
        size.height,
        img.typ()
    );
    img
}

fn write_image(path: &Path, img: &Mat) {
    let path = path.to_str().unwrap();

    imwrite(&path, img, &Vector::new()).unwrap();
    let size = img.size().unwrap();
    println!(
        "Write {} size {}x{} type {}",
        path,
        size.width,
        size.height,
        img.typ()
    );
}
