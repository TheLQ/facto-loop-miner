use tracing::debug;

use crate::{
    blueprint::bpfac::{FacBpInteger, entity::FacBpEntity, position::FacBpPosition},
    common::names::FacEntityName,
    game_entities::{
        belt_under::FacEntBeltUnderType, direction::FacDirectionEighth, module::FacModule,
    },
};

use super::vpoint::VPoint;

pub trait FacEntity: FacArea + std::fmt::Debug {
    fn name(&self) -> &FacEntityName;

    fn into_boxed(self) -> Box<impl FacEntity>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn to_fac_usize(&self, entity_number: usize, position: &VPoint) -> FacBpEntity {
        self.to_fac(entity_number.try_into().unwrap(), position)
    }

    fn to_fac(&self, entity_number: FacBpInteger, position: &VPoint) -> FacBpEntity {
        let facpos = self.to_fac_position(position);
        debug!(
            "blueprint pos {:6} facpos {:10} {:?}",
            position.display(),
            facpos.display(),
            self
        );
        FacBpEntity {
            entity_number,
            name: self.name().to_fac_name(),
            position: facpos,
            direction: self.to_fac_direction(),
            neighbours: None,
            recipe: self.to_fac_recipe(),
            items: self.to_fac_items(),
            utype: self.to_fac_belt_under_type(),
        }
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        None
    }

    fn to_fac_recipe(&self) -> Option<String> {
        None
    }

    fn to_fac_items(&self) -> Option<Vec<FacModule>> {
        None
    }

    fn to_fac_belt_under_type(&self) -> Option<FacEntBeltUnderType> {
        None
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

    pub const fn rectangle(width: usize, height: usize) -> Self {
        Size { width, height }
    }

    pub fn height(&self) -> &usize {
        &self.height
    }

    pub fn width(&self) -> &usize {
        &self.width
    }
}

pub trait FacArea {
    // fn area(&self) -> Vec<VPoint>;

    fn rectangle_size(&self) -> Size;

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition;
}

pub trait SquareArea {
    fn area_diameter() -> usize;
}

impl<T: SquareArea> FacArea for T {
    // fn area(&self) -> Vec<VPoint> {
    //     let diameter = T::area_diameter();
    //     let mut res = vec![VPoint::zero(); diameter];
    //     for x in 0..diameter {
    //         for y in 0..diameter {
    //             res[y * diameter + x] = VPoint::new_usize(x, y)
    //         }
    //     }
    //     res
    // }

    fn rectangle_size(&self) -> Size {
        Size::square(T::area_diameter())
    }

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        position.to_fac_with_offset(T::area_diameter() as f32 / 2.0)
    }
}

pub fn unwrap_options_to_option_vec<T: Clone>(input: &[Option<T>]) -> Option<Vec<T>> {
    let res: Vec<T> = input.iter().filter_map(|v| v.to_owned()).collect();
    if res.is_empty() { None } else { Some(res) }
}
