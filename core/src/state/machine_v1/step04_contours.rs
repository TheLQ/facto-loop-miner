use crate::PixelKdTree;
use crate::opencv::combine_rects_into_big_rect;
use crate::state::err::XMachineResult;
use crate::state::machine::{Step, StepParams};
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surfacev::vpatch::VPatch;
use crate::surfacev::vsurface::VSurface;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::{VPOINT_ONE, VPoint};
use facto_loop_miner_fac_engine::opencv_re::core::{
    Point, Rect, ToInputArray, ToInputOutputArray, Vector,
};
use facto_loop_miner_fac_engine::opencv_re::imgcodecs::imwrite;
use facto_loop_miner_fac_engine::opencv_re::imgproc::{
    CHAIN_APPROX_NONE, LINE_8, RETR_EXTERNAL, bounding_rect, find_contours, rectangle,
};
use facto_loop_miner_fac_engine::opencv_re::prelude::*;
use itertools::Itertools;
use kiddo::{Manhattan, NearestNeighbour};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::path::Path;
use tracing::{debug, info, trace};

// const WRITE_DEBUG_IMAGE: bool = false;

/// For fun, detect resource patches in image with OpenCV.
pub struct Step04 {}

impl Step04 {
    pub fn new_boxed() -> Box<dyn Step> {
        Box::new(Step04 {})
    }
}

// type lookup table https://stackoverflow.com/questions/10167534/how-to-find-out-what-type-of-a-mat-object-is-with-mattype-in-opencv
impl Step for Step04 {
    fn name(&self) -> &'static str {
        "step04-contours"
    }

    fn transformer(&self, params: StepParams) -> XMachineResult<()> {
        let mut surface = VSurface::load_from_last_step(&params)?;

        let disk_patches = detector(&surface, &params.step_out_dir);
        surface.add_patches(disk_patches);

        // if WRITE_DEBUG_IMAGE {
        //     write_surface_with_all_patches_wrapped(&mut surface);
        // }

        for pixel in Pixel::iter_resource() {
            let mut max_width = 0;
            let mut max_height = 0;
            let mut total = 0;
            for patch in surface.get_patches_slice() {
                if patch.resource != pixel {
                    continue;
                }
                let diff = patch.area.as_size();
                max_width = max_width.max(diff.x());
                max_height = max_height.max(diff.y());
                total += 1;
            }
            info!(
                "Resource {:?} patches {} max_width {} max_height {}",
                pixel, total, max_width, max_height
            );

            // // validate
            // let surface_points: Vec<VPoint> = surface
            //     .get_pixels_all()
            //     .filter_map(|v| {
            //         if v.pixel() == &pixel {
            //             Some(v.get_xy()[0])
            //         } else {
            //             None
            //         }
            //     })
            //     .collect();
            // let patch_points = surface
            //     .get_patches_slice()
            //     .iter()
            //     .filter(|v| v.resource == pixel)
            //     .flat_map(|v| &v.pixel_indexes)
            //     .cloned()
            //     .collect_vec();
            // let mut area_points = surface
            //     .get_patches_slice()
            //     .iter()
            //     .filter(|v| v.resource == pixel)
            //     .flat_map(|v| v.area.get_points())
            //     .collect_vec();
            // area_points.sort();
            //
            // let mut bad = 0;
            // for surface_point in &surface_points {
            //     if !patch_points.contains(surface_point) {
            //         bad += 1;
            //     }
            // }
            // info!(
            //     "pixel_indexes pixel {:?} bad {} total {}",
            //     pixel,
            //     bad,
            //     surface_points.len()
            // );
            //
            // let mut bad = 0;
            // for patch_point in &patch_points {
            //     if !area_points.contains(patch_point) {
            //         bad += 1;
            //     }
            // }
            // info!(
            //     "area pixel {:?} bad {} total {}",
            //     pixel,
            //     bad,
            //     surface_points.len()
            // );
        }
        // if 1 + 1 == 2 {
        //     exit(1);
        // }
        surface.save(&params.step_out_dir)?;

        Ok(())
    }
}

// #[allow(dead_code)]
// fn write_surface_with_all_patches_wrapped(surface: &mut VSurface) {
//     let mut img_gen = surface.to_pixel_cv_image(None);
//     let mut img =
//     draw_patch_border(
//         &mut img,
//         surface.get_patches_slice().iter().map(|e| e.area.to_rect()),
//     );
// }

fn write_png(path: &Path, img: &impl ToInputArray) {
    debug!("Wrote debug image {}", path.display());
    imwrite(path.to_str().unwrap(), img, &Vector::new()).unwrap();
}

fn detector(surface_meta: &VSurface, out_dir: &Path) -> Vec<VPatch> {
    let mut patches: Vec<VPatch> = Vec::new();
    for pixel in Pixel::iter_resource() {
        let detected_patches = detect_pixel(surface_meta, out_dir, pixel);
        patches.extend(detected_patches.into_iter());
    }
    patches
}

fn detect_pixel(surface_meta: &VSurface, out_dir: &Path, pixel: Pixel) -> Vec<VPatch> {
    surface_meta.log_pixel_stats("detect_pixel");
    let mut img_gen = surface_meta.to_pixel_cv_image(Some(pixel));
    let mut img = img_gen.as_mat();
    let size = img.size().unwrap();
    debug!(
        "Read size {}x{} type {}",
        size.width,
        size.height,
        img.typ(),
    );

    let mut patch_rects = detect_patch_rectangles(&img);
    debug!("Found {} patch rects", patch_rects.len());

    let patch_corner_cloud = map_patch_corners_to_kdtree(patch_rects.iter());
    detect_merge_nearby_patches(&mut patch_rects, &patch_corner_cloud);

    draw_patch_border(&mut img, patch_rects.iter().cloned());
    let debug_image_name = format!("cv-{}.png", pixel.as_ref());
    write_png(&out_dir.join(debug_image_name), &img);

    patch_rects
        .into_iter()
        .map(|patch_rect| {
            let radius_offset =
                VPoint::new(surface_meta.get_radius_i32(), surface_meta.get_radius_i32());

            // debug!("from {patch_rect:?}");
            let start = VPoint::from_rect_start(&patch_rect) - radius_offset;
            assert!(
                !surface_meta.is_point_out_of_bounds(&start),
                "start {start} not in radius {}",
                surface_meta.get_radius()
            );
            // debug!("start {start:?}");
            let end = start + VPoint::new(patch_rect.width, patch_rect.height);
            // if patch_rect.width != 1 && patch_rect.height != 1 {
            //     end -= VPOINT_ONE;
            // }

            // todo: this is trimmed in later
            // assert!(
            //     !surface_meta.is_point_out_of_bounds(&end),
            //     "end {end} not in radius {}",
            //     surface_meta.get_radius()
            // );
            // debug!("end   {end:?}");

            #[cfg(why_no_work)]
            {
                let patch_area = VArea::from_arbitrary_points_pair(start, end);
                let raw_area_points = patch_area.get_points();

                let any_out_of_bounds = raw_area_points
                    .iter()
                    .filter(|p| surface_meta.is_point_out_of_bounds(p))
                    .collect_vec();
                let mut debug_any = any_out_of_bounds
                    .iter()
                    .map(|v| format!("{}", v))
                    .join(", ");
                debug_any.truncate(500);
                assert_eq!(
                    any_out_of_bounds.len(),
                    0,
                    "points out of bounds {debug_any}",
                );

                let points = raw_area_points
                    .into_iter()
                    .filter(|p| surface_meta.get_pixel(p) == pixel)
                    .collect_vec();
                assert_ne!(points.len(), 0, "no points found for pixel");

                // recreate bounding area
                let test_patch_area = VArea::from_arbitrary_points(&points);
                // assert_eq!(
                //     test_patch_area, patch_area,
                //     "cv rect doesn't match found points"
                // );
                if test_patch_area != patch_area {
                    warn!("initial patch rect {:?}", patch_rect);
                    panic!(
                        "cv rect doesn't match found points\n\
                    from : {patch_area:?}\n\
                    recal: {test_patch_area:?}\n\
                    from  size: {:?}\n\
                    recal size: {:?}",
                        patch_area.as_size(),
                        test_patch_area.as_size(),
                    );
                }
                VPatch::new(patch_area, pixel, points)
            }

            {
                // For some reason the opencv patch rects aren't exactly accurate
                // Instead make a bigger box and shrink to known points via the surface

                let offset = VPoint::new(3, 3);
                let start = (start - offset).trim_max(surface_meta.point_top_left() + VPOINT_ONE);
                let end = (end + offset).trim_min(surface_meta.point_bottom_right() - VPOINT_ONE);

                let points = VArea::from_arbitrary_points_pair(start, end)
                    .get_points()
                    .into_iter()
                    .filter(|p| surface_meta.get_pixel(p) == pixel)
                    .collect_vec();
                let patch_area = VArea::from_arbitrary_points(&points);

                VPatch::new(patch_area, pixel, points)
            }
        })
        .collect()
}

fn detect_patch_rectangles(base: &impl ToInputArray) -> Vec<Rect> {
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
        // RETR_LIST,
        CHAIN_APPROX_NONE,
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
        .into_iter()
        .map(|contour| bounding_rect(&contour).unwrap())
        .collect();

    rects
}

/// Merge, for example Oil wells patches into a single Oil patch.
fn detect_merge_nearby_patches(patch_rects: &mut Vec<Rect>, cloud: &PixelKdTree) {
    const SEARCH_UNIT: u32 = 25;

    // Query kdtree for all nearby points for every point
    let mut search_distance = SEARCH_UNIT;
    let mut last_nearby_points_count = 0;
    let mut empty_count = 0;
    let mut within_search: Vec<Vec<NearestNeighbour<_, _>>>;
    loop {
        within_search = patch_rects
            .iter()
            .map(|patch_rect| {
                cloud.within::<Manhattan>(
                    &VPoint::from_rect_start(patch_rect).to_slice_f32(),
                    search_distance as f32,
                )
            })
            .collect();

        let nearby_points_count = within_search.len();
        match last_nearby_points_count.cmp(&nearby_points_count) {
            Ordering::Less => {
                trace!(
                    "increasing search distance to {} because found from {} to {} points",
                    search_distance, last_nearby_points_count, nearby_points_count
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

    let within_search: Vec<Vec<NearestNeighbour<_, _>>> = patch_rects
        .iter()
        .map(|patch_rect| {
            cloud.within::<Manhattan>(
                &VPoint::from_rect_start(patch_rect).to_slice_f32(),
                search_square_size as f32,
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

fn draw_patch_border(img: &mut impl ToInputOutputArray, rects: impl Iterator<Item = Rect>) {
    for rect in rects {
        rectangle(img, rect, Pixel::Highlighter.scalar_cv(), 2, LINE_8, 0).unwrap();
    }
}

pub fn map_patch_corners_to_kdtree<'a>(patch_rects: impl Iterator<Item = &'a Rect>) -> PixelKdTree {
    let mut tree: PixelKdTree = PixelKdTree::new();
    for (patch_counter, patch_rect) in patch_rects.enumerate() {
        tree.add(&map_rect_corner_to_slice(patch_rect), patch_counter);
    }
    tree
}

pub fn map_rect_corner_to_slice(rect: &Rect) -> [f32; 2] {
    [rect.x as f32, rect.y as f32]
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
