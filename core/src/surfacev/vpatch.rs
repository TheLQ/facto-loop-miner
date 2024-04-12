use crate::surface::pixel::Pixel;
use crate::surfacev::varea::VArea;
use opencv::core::Rect;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
            area: VArea::from_rect(&rect),
            pixel_indexes: Vec::new(),
        }
    }

    pub fn normalize_patch_6x6(&self) -> Self {
        let x_adjust = self.area.start.x() % 6;
        let y_adjust = self.area.start.y() % 6;

        let mut new = self.clone();
        new.area = VArea {
            start: self.area.start.move_xy(-x_adjust, -y_adjust),
            height: (new.area.height as i32 + y_adjust) as u32,
            width: (new.area.width as i32 + x_adjust) as u32,
        };
        new.area.start.assert_8x8_position();
        new
    }
}
