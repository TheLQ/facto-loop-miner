use crate::surface::pixel::Pixel;
use crate::surfacev::varea::VArea;
use crate::surfacev::vpoint::VPoint;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VPatch {
    pub resource: Pixel,
    pub area: VArea,
    pub pixel_indexes: Vec<VPoint>,
}

const ALERT: i32 = 1000;

impl VPatch {
    pub fn new(area: VArea, resource: Pixel, pixel_indexes: Vec<VPoint>) -> Self {
        if !resource.is_resource() {
            panic!("not a resource {:?}", resource);
        }
        if area.width > ALERT as u32 || area.height > ALERT as u32 {
            warn!("rect probably too big {:?}", area);
        }
        VPatch {
            resource,
            area,
            pixel_indexes,
        }
    }

    pub fn new_from_rect(rect: Rect, resource: Pixel, pixel_indexes: Vec<VPoint>) -> Self {
        if !resource.is_resource() {
            panic!("not a resource {:?}", resource);
        }
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
            area: VArea::from_rect(&rect),
            pixel_indexes,
        }
    }

    pub fn normalize_patch_even_8x8(&self) -> Self {
        let mut new = self.clone();
        new.area = self.area.normalize_even_8x8();
        new
    }
}
