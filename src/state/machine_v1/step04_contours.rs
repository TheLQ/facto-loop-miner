use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use fixed::types::extra::U0;
use fixed::FixedU32;
use itertools::Itertools;
use kiddo::float::distance::squared_euclidean;
use kiddo::float::neighbour::Neighbour;
use kiddo::KdTree;
use opencv::core::{Point, Rect, Scalar, Vector};
use opencv::imgcodecs::imwrite;
use opencv::imgproc::{
    bounding_rect, find_contours, rectangle, CHAIN_APPROX_SIMPLE, LINE_8, RETR_EXTERNAL,
};
use opencv::prelude::*;
use std::fmt::Display;
use std::fs::read;
use std::path::Path;

type PixelKdTree = KdTree<f32, 2>;

pub struct Step04 {}

impl Step04 {
    pub fn new() -> Box<dyn Step> {
        Box::new(Step04 {})
    }
}

// type lookup table https://stackoverflow.com/questions/10167534/how-to-find-out-what-type-of-a-mat-object-is-with-mattype-in-opencv
impl Step for Step04 {
    fn name(&self) -> String {
        "step04-contours".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let previous_step_dir = params.step_history_out_dirs.last().unwrap();

        let surface_meta = Surface::load_meta(&previous_step_dir);
        let surface_raw_path = previous_step_dir.join("surface-raw.dat");
        println!("Loading {}", surface_raw_path.display());
        let mut img = load_raw_image(
            &surface_raw_path,
            surface_meta.height as usize,
            surface_meta.width as usize,
            Pixel::IronOre,
        );
        let size = img.size().unwrap();
        println!(
            "Read {} size {}x{} type {}",
            surface_raw_path.display(),
            size.width,
            size.height,
            img.typ()
        );

        detector(&mut img);

        write_png(&params.step_out_dir.join("cv.png"), &img);
    }
}

fn load_raw_image(path: &Path, rows: usize, height: usize, pixel: Pixel) -> Mat {
    let mut surface_raw = read(path).unwrap();
    let pixel_id = pixel as u8;

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

fn write_png(path: &Path, img: &Mat) {
    imwrite(path.to_str().unwrap(), img, &Vector::new()).unwrap();
}

fn detector(base: &mut Mat) {
    let mut patch_rects = detect_patch_rectangles(base);

    let first_debug = patch_rects.get(0).unwrap();
    println!("{}", rect_to_string(first_debug));

    let cloud = map_patch_corners_to_kdtree(&patch_rects);
    detect_merge_nearby_patches(&mut patch_rects, &cloud);

    draw_patch_border(base, patch_rects.into_iter());
}

fn detect_patch_rectangles(base: &Mat) -> Vec<Rect> {
    let mut contours: Vector<Vector<Point>> = Vector::default();
    let offset = Point { x: 0, y: 0 };
    // RETR_LIST - May make rectangles inside rectangles, other multiple rectangles at whisps
    // RETR_CCOMP - Same???
    //
    // CHAIN_APPROX_NONE - Stores all points. unnecessary
    // CHAIN_APPROX_SIMPLE - Only store corners, which are the relevant points
    // CHAIN_APPROX_TC89_L1 - Didn't do anything????
    find_contours(
        base,
        &mut contours,
        RETR_EXTERNAL,
        CHAIN_APPROX_SIMPLE,
        offset,
    )
    .unwrap();

    println!("found contours {}", contours.len());
    metricify("contour sizes", contours.iter().map(|v| v.len()));

    // let first_debug = contours.get(0).unwrap();
    // println!(
    //     "countour 1 {}",
    //     first_debug
    //         .iter()
    //         .map(|point| format!("{}x{}", point.x, point.y))
    //         .join(", ")
    // );

    let rects: Vec<Rect> = contours
        .iter()
        .map(|contour| bounding_rect(&contour).unwrap())
        .collect();
    rects
}

fn map_patch_corners_to_kdtree(patch_rects: &Vec<Rect>) -> PixelKdTree {
    let mut tree: PixelKdTree = KdTree::new();
    let mut patch_counter = 0;
    for patch_rect in patch_rects {
        tree.add(&rect_corner_to_slice(patch_rect), patch_counter);
        patch_counter = patch_counter + 1;
    }
    tree
}

const MERGE_WITHIN_DIAMETERS: i32 = 100;

fn detect_merge_nearby_patches(patch_rects: &mut Vec<Rect>, cloud: &PixelKdTree) {
    let mut search_square_size = 0;
    // find largest size
    for patch in patch_rects.iter() {
        search_square_size = search_square_size.max(patch.width);
        search_square_size = search_square_size.max(patch.height);
    }
    // arbitrary size, for some reason within 1 diameter for IronOre still finds max 5...
    search_square_size = search_square_size * MERGE_WITHIN_DIAMETERS;

    let within_search: Vec<Vec<Neighbour<_, _>>> = patch_rects
        .iter()
        .map(|patch| {
            cloud.within(
                &rect_corner_to_slice(patch),
                search_square_size as f32,
                &squared_euclidean,
            )
        })
        .collect();

    Metrics::process(
        "within",
        within_search.iter().map(|input| input.len().to_string()),
    );

    let mut combine_replacements: Vec<Rect> = Vec::new();
    let mut combine_mask: Vec<bool> = vec![false; patch_rects.len()];
    'search: for within in &within_search {
        if within.len() <= 1 {
            continue;
        }
        // TODO: First mergeable wins. This isn't always the optimal merge. Close enough
        for neighbor in within {
            if combine_mask[neighbor.item] {
                continue 'search;
            }
        }
        for neighbor in within {
            combine_mask[neighbor.item] = true;
        }

        let nearby_rects: Vec<Rect> = within
            .iter()
            .map(|neighbor| patch_rects[neighbor.item])
            .collect();

        // DIY...
        // let mut super_rect = Rect::default();
        // for nearby_rect in nearby_rects {
        //     super_rect.x = super_rect.x.min(nearby_rect.x);
        //     super_rect.y = super_rect.y.min(nearby_rect.y);
        //     // warning: type abuse
        //     super_rect.x = super_rect.x.min(nearby_rect.x);
        // }

        // lazy opencv way
        let mut corners: Vec<Point> = Vec::new();
        for nearby_rect in &nearby_rects {
            corners.push(Point::new(nearby_rect.x, nearby_rect.y));
            corners.push(Point::new(
                nearby_rect.x + nearby_rect.width,
                nearby_rect.y + nearby_rect.height,
            ));
        }
        let super_rect = bounding_rect(&Vector::from_slice(&corners)).unwrap();

        // println!("====");
        // println!(
        //     "for {}",
        //     (&nearby_rects)
        //         .iter()
        //         .map(|rect| rect_to_string((rect)))
        //         .join(", ")
        // );
        // println!("got super {}", rect_to_string(&super_rect));

        combine_replacements.push(super_rect);
    }

    println!("patches init {}", patch_rects.len());
    for (pos, mask) in combine_mask.iter().enumerate().rev() {
        if *mask {
            patch_rects.remove(pos);
        }
    }
    println!("patches with mergeable removed {}", patch_rects.len());

    // patch_rects.clear();
    for super_rect in combine_replacements {
        patch_rects.push(super_rect);
    }
    println!("patches with merge replacements {}", patch_rects.len());
}

fn draw_patch_border(img: &mut Mat, rects: impl Iterator<Item = Rect>) {
    for rect in rects {
        rectangle(img, rect, Scalar::from(100f64), 2, LINE_8, 0).unwrap();
    }
}

fn rect_corner_to_slice(patch: &Rect) -> [f32; 2] {
    [patch.x as f32, patch.y as f32]
}

fn metricify<I, T>(name: &str, input: I)
where
    I: Iterator<Item = T>,
    T: Display + PartialEq + Clone,
{
    let mut found: Vec<T> = Vec::new();
    for val in input {
        if !found.contains(&val) {
            println!("{} {}", name, &val);
            found.push(val.clone());
        }
    }
}

fn rect_to_string(rect: &Rect) -> String {
    format!(
        "h {} w {} x {} y {}",
        rect.height, rect.width, rect.x, rect.y
    )
}
