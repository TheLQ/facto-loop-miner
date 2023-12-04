use crate::surface::pixel::Pixel;
use crate::surfacev::varea::VArea;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Serialize, Deserialize)]
pub struct VPatch {
    pub resource: Pixel,
    pub area: VArea,
    pub pixel_indexes: Vec<usize>,
}

impl VPatch {
    pub fn new_from_rect(rect: Rect, resource: Pixel) -> Self {
        if !resource.is_resource() {
            panic!("not a resource {:?}", resource);
        }
        const ALERT: i32 = 1000;
        // assert!(
        //     rect.width < ALERT && rect.height < ALERT,
        //     "rect probably too big {:?}",
        //     rect
        // );
        if rect.width > ALERT || rect.height > ALERT {
            warn!("rect probably too big {:?}", rect);
        }
        VPatch {
            resource,
            area: VArea::new_from_rect(&rect),
            pixel_indexes: Vec::new(),
        }
    }
}
