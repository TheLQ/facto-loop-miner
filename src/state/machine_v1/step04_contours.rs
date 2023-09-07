use crate::opencv::load_raw_image_with_surface;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::patch::{map_patch_corners_to_kdtree, DiskPatch, Patch};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::PixelKdTree;
use kiddo::float::distance::squared_euclidean;
use kiddo::float::neighbour::Neighbour;
use opencv::core::{Point, Rect, Vector};
use opencv::imgcodecs::imwrite;
use opencv::imgproc::{
    bounding_rect, find_contours, rectangle, CHAIN_APPROX_SIMPLE, LINE_8, RETR_EXTERNAL,
};
use opencv::prelude::*;
use std::fmt::Display;
use std::mem::transmute;
use std::path::Path;
use strum::IntoEnumIterator;

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

        let surface = Surface::load(&previous_step_dir);

        let surface_raw_path = previous_step_dir.join("surface-raw.dat");
        println!("Loading {}", surface_raw_path.display());
        let disk_patches = detector(&surface_raw_path, &surface, &params.step_out_dir);

        disk_patches.save(&params.step_out_dir);

        // write_surface_with_all_patches_wrapped(&mut surface, &disk_patches);
        // surface.save(&params.step_out_dir);
    }
}

#[allow(dead_code)]
fn write_surface_with_all_patches_wrapped(surface: &mut Surface, disk_patches: &DiskPatch) {
    let mut img = surface.get_buffer_to_cv();
    for (_pixel, patches) in &disk_patches.patches {
        let vec: Vec<Rect> = patches
            .into_iter()
            .map(|patch| patch.patch_to_rect())
            .collect();
        draw_patch_border(&mut img, vec);
    }
    let raw: &[Pixel] = unsafe { transmute(img.data_bytes().unwrap()) };
    surface.buffer = Vec::from(raw);
}

fn write_png(path: &Path, img: &Mat) {
    println!("Wrote debug image {}", path.display());
    imwrite(path.to_str().unwrap(), img, &Vector::new()).unwrap();
}

fn detector(surface_raw_path: &Path, surface_meta: &Surface, out_dir: &Path) -> DiskPatch {
    let mut disk = DiskPatch::default();
    for pixel in Pixel::iter() {
        if pixel.is_resource() {
            let patches = detect_pixel(surface_raw_path, surface_meta, out_dir, &pixel);
            disk.patches.insert(pixel, patches);
        }
    }
    disk.area_box = surface_meta.area_box.clone();
    disk
}

fn detect_pixel(
    surface_raw_path: &Path,
    surface_meta: &Surface,
    out_dir: &Path,
    pixel: &Pixel,
) -> Vec<Patch> {
    let mut img = load_raw_image_with_surface(surface_raw_path, surface_meta, Some(pixel));
    let size = img.size().unwrap();
    println!(
        "Read {} size {}x{} type {}",
        surface_raw_path.display(),
        size.width,
        size.height,
        img.typ()
    );

    let mut patch_rects = detect_patch_rectangles(&img);

    let cloud = map_patch_corners_to_kdtree(patch_rects.iter().map(Patch::from));
    detect_merge_nearby_patches(&mut patch_rects, &cloud, pixel);

    draw_patch_border(&mut img, patch_rects.iter().cloned().collect());
    let debug_image_name = format!("cv-{}.png", pixel.as_ref());
    write_png(&out_dir.join(debug_image_name), &img);

    patch_rects.into_iter().map(Patch::from).collect()
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
    // metricify("contour sizes", contours.iter().map(|v| v.len()));

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

fn detect_merge_nearby_patches(patch_rects: &mut Vec<Rect>, cloud: &PixelKdTree, pixel: &Pixel) {
    let mut search_square_size = 0;
    // find largest size
    for patch in patch_rects.iter() {
        search_square_size = search_square_size.max(patch.width);
        search_square_size = search_square_size.max(patch.height);
    }
    search_square_size = search_square_size + 1;
    // arbitrary size, for some reason within 1 diameter for IronOre still finds max 5...
    search_square_size = pixel.nearby_patch_search_distance(search_square_size);

    let within_search: Vec<Vec<Neighbour<_, _>>> = patch_rects
        .iter()
        .map(|patch| {
            cloud.within(
                &Patch::from(patch).corner_slice(),
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

fn draw_patch_border(img: &mut Mat, rects: Vec<Rect>) {
    for rect in rects {
        rectangle(img, rect, Pixel::Highlighter.scalar_cv(), 2, LINE_8, 0).unwrap();
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn rect_to_string(rect: &Rect) -> String {
    format!(
        "h {} w {} x {} y {}",
        rect.height, rect.width, rect.x, rect.y
    )
}
