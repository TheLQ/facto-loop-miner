use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct DiskPatch {
    pub patches: HashMap<String, Vec<Patch>>,
}

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Patch {
    pub fn from_rect(rect: Rect) -> Self {
        Patch {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}
