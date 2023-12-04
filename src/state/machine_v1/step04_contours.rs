use crate::opencv::{combine_rects_into_big_rect, get_cv_bounding_rect};
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::patch::{map_patch_corners_to_kdtree, Patch};
use crate::surface::pixel::Pixel;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use crate::PixelKdTree;
use itertools::Itertools;
use kiddo::float::distance::{manhattan, squared_euclidean};
use kiddo::float::neighbour::Neighbour;
use opencv::core::{Point, Rect, Vector};
use opencv::imgcodecs::imwrite;
use opencv::imgproc::{
    bounding_rect, find_contours, rectangle, CHAIN_APPROX_SIMPLE, LINE_8, RETR_EXTERNAL,
};
use opencv::prelude::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::path::Path;
use strum::IntoEnumIterator;
use tracing::{debug, info, trace};

const WRITE_DEBUG_IMAGE: bool = false;

/// For fun, detect resource patches in image with OpenCV.
pub struct Step04 {}

impl Step04 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step04 {})
    }
}

// type lookup table https://stackoverflow.com/questions/10167534/how-to-find-out-what-type-of-a-mat-object-is-with-mattype-in-opencv
impl Step for Step04 {
    fn name(&self) -> String {
        "step04-contours".to_string()
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        let disk_patches = detector(&surface, &params.step_out_dir);
        surface.add_patches(disk_patches);

        if WRITE_DEBUG_IMAGE {
            write_surface_with_all_patches_wrapped(&mut surface);
        }

        for pixel in Pixel::iter_resource() {
            let mut max_width = 0u32;
            let mut max_height = 0u32;
            let mut total = 0;
            for patch in surface.get_patches_iter() {
                if patch.resource != pixel {
                    continue;
                }
                max_width = max_width.max(patch.area.width);
                max_height = max_height.max(patch.area.height);
                total += 1;
            }
            info!(
                "Resource {:?} patches {} max_width {} max_height {}",
                pixel, total, max_width, max_height
            );
        }
        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

#[allow(dead_code)]
fn write_surface_with_all_patches_wrapped(surface: &mut VSurface) {
    let mut img = surface.to_pixel_cv_image(None);
    draw_patch_border(
        &mut img,
        surface
            .get_patches_iter()
            .into_iter()
            .map(|e| e.area.to_rect()),
    );
}

fn write_png(path: &Path, img: &Mat) {
    tracing::debug!("Wrote debug image {}", path.display());
    imwrite(path.to_str().unwrap(), img, &Vector::new()).unwrap();
}

fn detector(surface_meta: &VSurface, out_dir: &Path) -> Vec<VPatch> {
    let mut patches: Vec<VPatch> = Vec::new();
    for pixel in Pixel::iter_resource() {
        let detected_patches = detect_pixel(surface_meta, out_dir, &pixel);
        patches.extend(detected_patches.into_iter());
    }
    patches
}

fn detect_pixel(surface_meta: &VSurface, out_dir: &Path, pixel: &Pixel) -> Vec<VPatch> {
    let mut img = surface_meta.to_pixel_cv_image(Some(pixel.clone()));
    let size = img.size().unwrap();
    debug!(
        "Read size {}x{} type {}",
        size.width,
        size.height,
        img.typ(),
    );

    let mut patch_rects = detect_patch_rectangles(&img);

    let patch_corner_cloud = map_patch_corners_to_kdtree(patch_rects.iter());
    // detect_merge_nearby_patches(&mut patch_rects, &patch_corner_cloud);

    draw_patch_border(&mut img, patch_rects.iter().cloned());
    let debug_image_name = format!("cv-{}.png", pixel.as_ref());
    write_png(&out_dir.join(debug_image_name), &img);

    patch_rects
        .into_iter()
        .map(|e| VPatch::new_from_rect(e, pixel.clone()))
        .collect()
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

    tracing::debug!("found contours {}", contours.len());
    // metricify("contour sizes", contours.iter().map(|v| v.len()));

    // let first_debug = contours.get(0).unwrap();
    // tracing::debug!(
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

/// Merge, for example Oil wells patches into a single Oil patch.   
fn detect_merge_nearby_patches(patch_rects: &mut Vec<Rect>, cloud: &PixelKdTree) {
    const SEARCH_UNIT: u32 = 100;

    // Query kdtree for all nearby points for every point
    let mut search_distance = SEARCH_UNIT;
    let mut last_nearby_points_count = 0;
    let mut empty_count = 0;
    let mut within_search: Vec<Vec<Neighbour<_, _>>>;
    loop {
        within_search = patch_rects
            .iter()
            .map(|patch_rect| {
                cloud.within(
                    &VArea::new_from_rect(patch_rect).start.to_slice_f32(),
                    search_distance as f32,
                    &manhattan,
                )
            })
            .collect();

        let nearby_points_count = within_search.len();
        match last_nearby_points_count.cmp(&nearby_points_count) {
            Ordering::Less => {
                trace!(
                    "increasing search distance to {} because found from {} to {} points",
                    search_distance,
                    last_nearby_points_count,
                    nearby_points_count
                );
                last_nearby_points_count = nearby_points_count;
            }
            Ordering::Equal | Ordering::Greater => {
                if empty_count < 4 {
                    empty_count += 1;
                    trace!("empty {}", empty_count);
                } else {
                    trace!("too many empty");
                    break;
                }
            }
        }
        search_distance += SEARCH_UNIT;
    }

    // Combine nearby points into groups
    let mut within_index_groups: Vec<HashSet<usize>> = Vec::new();
    for within_entry in within_search {
        if within_entry.len() <= 1 {
            continue;
        }
        let within_indexes: HashSet<usize> = within_entry.iter().map(|e| e.item).collect();
        let group = within_index_groups.iter_mut().find(|within_index_group| {
            within_index_group
                .intersection(&within_indexes)
                .next()
                .is_some()
        });
        let group: &mut HashSet<usize> = if let Some(group) = group {
            group
        } else {
            within_index_groups.push(HashSet::new());
            within_index_groups.last_mut().unwrap()
        };

        group.extend(within_indexes);
    }
    debug!("made {} groups to merge", within_index_groups.len());

    // push new groups as a bigger rect containing the points
    for group in &within_index_groups {
        let super_rect =
            combine_rects_into_big_rect(group.iter().map(|rect_index| &patch_rects[*rect_index]));
        patch_rects.push(super_rect);
    }

    // remove old patch groups
    for pos in within_index_groups.iter().flatten().sorted().unique().rev() {
        patch_rects.remove(*pos);
    }
}

/// Merge, for example Oil wells patches into a single Oil patch.   
fn detect_merge_nearby_patches_slow(
    patch_rects: &mut Vec<Rect>,
    cloud: &PixelKdTree,
    pixel: &Pixel,
) {
    let mut search_square_size = 0;
    // find largest size
    // for patch in patch_rects.iter() {
    //     search_square_size = search_square_size.max(patch.width);
    //     search_square_size = search_square_size.max(patch.height);
    // }
    // search_square_size += 1;
    // arbitrary size, for some reason within 1 diameter for IronOre still finds max 5...
    search_square_size += pixel.nearby_patch_search_distance(search_square_size);

    let within_search: Vec<Vec<Neighbour<_, _>>> = patch_rects
        .iter()
        .map(|patch_rect| {
            cloud.within(
                &VArea::new_from_rect(patch_rect).start.to_slice_f32(),
                search_square_size as f32,
                &manhattan,
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
        let super_rect = combine_rects_into_big_rect(&nearby_rects);

        // tracing::debug!("====");
        // tracing::debug!(
        //     "for {}",
        //     (&nearby_rects)
        //         .iter()
        //         .map(|rect| rect_to_string((rect)))
        //         .join(", ")
        // );
        // tracing::debug!("got super {}", rect_to_string(&super_rect));

        combine_replacements.push(super_rect);
    }

    tracing::debug!("patches init {}", patch_rects.len());
    for (pos, mask) in combine_mask.iter().enumerate().rev() {
        if *mask {
            patch_rects.remove(pos);
        }
    }
    tracing::debug!("patches with mergeable removed {}", patch_rects.len());

    // patch_rects.clear();
    for super_rect in combine_replacements {
        patch_rects.push(super_rect);
    }
    tracing::debug!("patches with merge replacements {}", patch_rects.len());
}

fn draw_patch_border(img: &mut Mat, rects: impl Iterator<Item = Rect>) {
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
            tracing::debug!("{} {}", name, &val);
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
