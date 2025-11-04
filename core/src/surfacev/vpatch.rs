use crate::surface::pixel::Pixel;
use derivative::Derivative;
use facto_loop_miner_fac_engine::common::varea::VArea;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Derivative, Eq, PartialOrd, Ord)]
#[derivative(Debug)]
pub struct VPatch {
    pub resource: Pixel,
    pub area: VArea,
    #[derivative(Debug = "ignore")]
    pub pixel_indexes: Vec<VPoint>,
}

const ALERT: i32 = 1000;

impl VPatch {
    pub fn new(area: VArea, resource: Pixel, pixel_indexes: Vec<VPoint>) -> Self {
        if !resource.is_resource() {
            panic!("not a resource {:?}", resource);
        }
        let size_point = area.as_size();
        if size_point.x() > ALERT || size_point.y() > ALERT {
            panic!("rect probably too big {:?} with size {size_point}", area);
        }
        VPatch {
            resource,
            area,
            pixel_indexes,
        }
    }

    // pub fn new_from_rect(rect: Rect, resource: Pixel, pixel_indexes: Vec<VPoint>) -> Self {
    //     if !resource.is_resource() {
    //         panic!("not a resource {:?}", resource);
    //     }
    //     // assert!(
    //     //     rect.width < ALERT && rect.height < ALERT,
    //     //     "rect probably too big {:?}",
    //     //     rect
    //     // );
    //     if rect.width > ALERT || rect.height > ALERT {
    //         warn!("rect probably too big {:?}", rect);
    //     }
    //     VPatch {
    //         resource,
    //         area: VArea::from_rect(&rect),
    //         pixel_indexes,
    //     }
    // }

    // pub fn normalize_step_rail(&self) -> Self {
    //     let mut new = self.clone();
    //     new.area = self.area.normalize_step_rail();
    //     new
    // }
}
