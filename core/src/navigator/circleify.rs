use crate::opencv::mat_into_points;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_fac_engine::common::vpoint::VPoint;
use facto_loop_miner_fac_engine::opencv_re::core::{CV_8U, Mat, Point, Scalar};
use facto_loop_miner_fac_engine::opencv_re::imgproc::{FILLED, LINE_4};
use facto_loop_miner_fac_engine::opencv_re::prelude::*;
use itertools::Itertools;
use tracing::trace;

/// We already dragged in opencv, why not use it's easy circle function
// pub fn draw_circle(radius: u32) -> &'static [VPoint] {
fn draw_circle(radius: u32) -> Vec<VPoint> {
    // static CACHE: Arc<HashMap<u32, Vec<VPoint>>> = Arc::new(HashMap::new());
    // if let Some(exist) = unsafe { CACHE.get(&radius) } {
    //     return exist;
    // }

    let radius_i32 = radius as i32;
    let diameter = (radius_i32 * 2) + /*extra rounding*/1;

    let mut mat = unsafe { Mat::new_rows_cols(diameter, diameter, CV_8U).unwrap() };
    let flag_color = u8::MAX;
    let center = Point {
        x: radius_i32,
        y: radius_i32,
    };
    let center_v = VPoint::from_cv_point(center);
    facto_loop_miner_fac_engine::opencv_re::imgproc::circle(
        &mut mat,
        center,
        radius_i32,
        Scalar::all(flag_color.into()),
        FILLED,
        LINE_4,
        0,
    )
    .unwrap();

    mat_into_points(mat, flag_color, center_v)

    // mat.iter::<u8>().unwrap().filter_map(|(point, value)| {
    //     if value == flag_color {
    //         Some(VPoint::from_cv_point(point) - center_v)
    //     } else {
    //         None
    //     }
    // })

    // unsafe {
    //     CACHE.insert(radius, points);
    //     CACHE.get(&radius).unwrap()
    // }
}

pub fn draw_circle_around(point: &VPoint, radius: u32) -> Vec<VPoint> {
    let watch = BasicWatch::start();
    let circle = draw_circle(radius);
    let res = circle.into_iter().map(|v| v + *point).collect_vec();
    trace!("gen {radius} ({} items) in {watch}", res.len());
    res
}

#[cfg(test)]
mod test {
    use crate::navigator::circleify::draw_circle_around;
    use facto_loop_miner_fac_engine::common::varea::VArea;
    use facto_loop_miner_fac_engine::common::vpoint::VPoint;

    #[test]
    fn test_around() {
        let circle = draw_circle_around(&VPoint::new(100, 100), 10);
        let area = VArea::from_arbitrary_points(&circle);
        assert_eq!(area.point_top_left(), VPoint::new(90, 90), "{area:?}");
        assert_eq!(area.point_bottom_right(), VPoint::new(110, 110), "{area:?}");
    }
}
