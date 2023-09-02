use crate::state::machine::search_step_history_dirs;
use kiddo::KdTree;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct DiskPatch {
    pub patches: HashMap<String, Vec<Patch>>,
}

impl DiskPatch {
    pub fn load_from_step_history(step_history_out_dirs: &Vec<PathBuf>) -> Self {
        let recent_surface =
            search_step_history_dirs(step_history_out_dirs.clone().into_iter(), "patches.json");

        let io = BufReader::new(File::open(recent_surface).unwrap());
        simd_json::from_reader(io).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
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
}

impl From<Rect> for Patch {
    fn from(rect: Rect) -> Self {
        Patch::from(&rect)
    }
}

impl From<&Rect> for Patch {
    fn from(rect: &Rect) -> Self {
        Patch {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}
pub type PixelKdTree = KdTree<f32, 2>;
pub fn map_patch_corners_to_kdtree(patch_rects: impl Iterator<Item = Patch>) -> PixelKdTree {
    let mut tree: PixelKdTree = KdTree::new();
    let mut patch_counter = 0;
    for patch_rect in patch_rects {
        tree.add(&patch_rect.corner_slice(), patch_counter);
        patch_counter = patch_counter + 1;
    }
    tree
}
