use crate::{
    blueprint::bpfac::{BpFacInteger, entity::FacBpEntity, position::FacBpPosition},
    common::names::FacEntityName,
    game_entities::direction::FacDirectionEighth,
};

use super::vpoint::VPoint;

pub trait FacEntity: FacArea {
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

    fn to_fac(&self, entity_number: BpFacInteger, position: &VPoint) -> FacBpEntity {
        FacBpEntity {
            entity_number,
            name: self.name().to_fac_name(),
            position: self.to_fac_position(position),
            direction: self.to_fac_direction(),
            neighbours: None,
            recipe: self.to_fac_recipe(),
            items: None,
        }
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        None
    }

    fn to_fac_recipe(&self) -> Option<String> {
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

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition;
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

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        position.to_fac(T::area_diameter() as f32 / 2.0)
    }
}
