use crate::state::machine::StepParams;
use crate::surface::game_locator::GameLocator;
use crate::surface::pixel::Pixel;
use crate::surface::surface::{PointU32, Surface};
use crate::surfacev::vpatch::VPatch;
use crate::PixelKdTree;
use kiddo::KdTree;
use opencv::core::{Point, Rect, Rect_};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct DiskPatch {
    pub patches: HashMap<Pixel, Vec<Patch>>,
    pub area_box: GameLocator,
}

impl DiskPatch {
    pub fn save(&self, dir: &Path) {
        let path = &dir.join("patches.json");
        tracing::debug!("Wrote output patch dump to {}", path.display());
        let file = File::create(path).unwrap();
        simd_json::to_writer(BufWriter::new(file), self).unwrap();
    }
}

const JSON_NAME: &str = "patches.json";

impl DiskPatch {
    pub fn load_from_step_history(step_history_out_dirs: &StepParams) -> Self {
        // let recent_surface =
        //     search_step_history_dirs(step_history_out_dirs.iter().cloned(), JSON_NAME);
        let recent_surface = step_history_out_dirs.previous_step_dir().join(JSON_NAME);
        if !recent_surface.exists() {
            panic!("missing file {}", recent_surface.display());
        }
        Self::load_from_dir(&recent_surface)
    }

    pub fn load_from_dir(dir: &Path) -> Self {
        let new = dir;
        if !new.exists() {
            panic!("missing {}", &new.display());
        }
        let io = BufReader::new(
            File::open(new)
                .map_err(|e| {
                    tracing::debug!("failed for {}", dir.display());
                    e
                })
                .unwrap(),
        );
        simd_json::from_reader(io).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Patch {
    pub fn corner_slice(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    pub fn corner_point_i32(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn corner_point_u32(&self) -> PointU32 {
        PointU32 {
            x: self.x as u32,
            y: self.y as u32,
        }
    }

    pub fn corner_point_slice_f32(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    pub fn patch_to_rect(&self) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn remove_resource_from_surface_square(&self, pixel: &Pixel, surface: &mut Surface) -> u32 {
        let mut metric = 0;
        for remove_x in self.x..self.x + self.width {
            for remove_y in self.y..self.y + self.height {
                let remove_x = remove_x as u32;
                let remove_y = remove_y as u32;
                if surface.get_pixel(remove_x, remove_y) == pixel {
                    surface.set_pixel(Pixel::Empty, remove_x, remove_y);
                    metric += 1;
                }
            }
        }
        metric
    }
}

impl From<Rect_<i32>> for Patch {
    fn from(rect: Rect_<i32>) -> Self {
        Patch::from(&rect)
    }
}

impl From<&Rect_<i32>> for Patch {
    fn from(rect: &Rect_<i32>) -> Self {
        Patch {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl PartialOrd<Self> for Patch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Patch {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.x as u64 * self.y as u64 * self.width as u64 * self.height as u64;
        let b = other.x as u64 * other.y as u64 * other.width as u64 * self.height as u64;
        a.cmp(&b)
    }
}

pub fn map_patch_map_to_kdtree(
    patches: &HashMap<Pixel, Vec<Patch>>,
) -> HashMap<Pixel, PixelKdTree> {
    patches
        .iter()
        .map(|(key_pixel, patches_for_key)| {
            (
                key_pixel.clone(),
                map_patch_corners_to_kdtree_ref(patches_for_key.iter()),
            )
        })
        .collect()
}

pub fn map_patch_corners_to_kdtree_ref<'a>(
    patch_rects: impl Iterator<Item = &'a Patch>,
) -> PixelKdTree {
    let mut tree: PixelKdTree = KdTree::new();
    for (patch_counter, patch_rect) in patch_rects.enumerate() {
        tree.add(&patch_rect.corner_slice(), patch_counter);
    }
    tree
}

pub fn map_patch_corners_to_kdtree<'a>(patch_rects: impl Iterator<Item = &'a Rect>) -> PixelKdTree {
    let mut tree: PixelKdTree = KdTree::new();
    for (patch_counter, patch_rect) in patch_rects.enumerate() {
        tree.add(&map_rect_corner_to_slice(&patch_rect), patch_counter);
    }
    tree
}

pub fn map_vpatch_to_kdtree<'a>(patch_rects: impl Iterator<Item = &'a VPatch>) -> PixelKdTree {
    let mut tree: PixelKdTree = KdTree::new();
    for (patch_counter, patch_rect) in patch_rects.enumerate() {
        tree.add(&patch_rect.area.start.to_slice_f32(), patch_counter);
    }
    tree
}

pub fn map_rect_corner_to_slice(rect: &Rect) -> [f32; 2] {
    [rect.x as f32, rect.y as f32]
}
