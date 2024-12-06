use crate::common::names::FacEntityName;

use super::vpoint::VPoint;

pub trait FacEntity: FacArea {
    fn name(&self) -> &FacEntityName;

    fn into_boxed(self) -> Box<impl FacEntity>
    where
        Self: Sized,
    {
        Box::new(self)
        // self.name().as_ref().to_string()
    }
}

pub struct Size {
    width: usize,
    height: usize,
}

impl Size {
    pub const fn square(size: usize) -> Self {
        Size {
            height: size,
            width: size,
        }
    }

    pub fn height(&self) -> &usize {
        &self.height
    }

    pub fn width(&self) -> &usize {
        &self.width
    }
}

pub trait FacArea {
    fn area(&self) -> Vec<VPoint>;

    fn rectangle_size(&self) -> Size;
}

pub trait SquareArea {
    fn area_diameter() -> usize;
}

impl<T: SquareArea> FacArea for T {
    fn area(&self) -> Vec<VPoint> {
        let diameter = T::area_diameter();
        let mut res = vec![VPoint::zero(); diameter];
        for x in 0..diameter {
            for y in 0..diameter {
                res[y * diameter + x] = VPoint::new_usize(x, y)
            }
        }
        res
    }

    fn rectangle_size(&self) -> Size {
        Size {
            height: T::area_diameter(),
            width: T::area_diameter(),
        }
    }
}
