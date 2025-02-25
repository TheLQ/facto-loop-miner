use crate::{
    blueprint::bpfac::{
        FacBpInteger, entity::FacBpEntity, infinity::FacBpInfinitySettings,
        position::FacBpPosition, schedule::FacBpSchedule,
    },
    common::names::FacEntityName,
    game_entities::{
        belt_split::FacExtPriority, belt_under::FacEntBeltUnderType, direction::FacDirectionEighth,
        module::FacModule,
    },
};

use super::vpoint::VPoint;

pub trait FacEntity: FacArea + std::fmt::Debug {
    fn name(&self) -> FacEntityName;

    fn into_boxed(self) -> Box<dyn FacEntity>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    // fn to_blueprint_usize(
    //     &self,
    //     entity_number: usize,
    //     position: &VPoint,
    //     output: &FacItemOutput,
    // ) -> FacBpEntity {
    //     self.to_blueprint(entity_number.try_into().unwrap(), position, output)
    // }

    fn to_blueprint(&self, entity_number: Option<FacBpInteger>, position: &VPoint) -> FacBpEntity {
        // println!("to_bp vpoint {}", position.display());
        FacBpEntity {
            entity_number,
            name: self.name().to_fac_name(),
            position: self.to_fac_position(position),
            direction: self.to_fac_direction(),
            neighbours: None,
            recipe: self.to_fac_recipe().map(|v| v.to_fac_name()),
            items: self.to_fac_items(),
            utype: self.to_fac_belt_under_type(),
            station: self.to_fac_station(),
            infinity_settings: self.to_fac_infinity_settings(),
            schedule: self.to_fac_schedule(),
            input_priority: self.to_fac_input_priority(),
            output_priority: self.to_fac_output_priority(),
        }
    }

    fn to_fac_direction(&self) -> Option<FacDirectionEighth> {
        None
    }

    fn to_fac_recipe(&self) -> Option<FacEntityName> {
        None
    }

    fn to_fac_items(&self) -> Option<Vec<FacModule>> {
        None
    }

    fn to_fac_belt_under_type(&self) -> Option<FacEntBeltUnderType> {
        None
    }

    fn to_fac_station(&self) -> Option<String> {
        None
    }

    fn to_fac_infinity_settings(&self) -> Option<FacBpInfinitySettings> {
        None
    }

    fn to_fac_schedule(&self) -> Option<FacBpSchedule> {
        None
    }

    fn to_fac_input_priority(&self) -> FacExtPriority {
        FacExtPriority::None
    }

    fn to_fac_output_priority(&self) -> FacExtPriority {
        FacExtPriority::None
    }
}

#[derive(Debug)]
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

    fn to_fac_position(&self, position: &VPoint) -> FacBpPosition {
        let size = self.rectangle_size();
        position
            .to_fac_with_offset_rectangle(*size.width() as f32 / 2.0, *size.height() as f32 / 2.0)
    }

    fn from_fac_position(&self, position: &FacBpPosition) -> VPoint {
        let size = self.rectangle_size();
        position.to_vpoint_with_offset(*size.width() as f32 / 2.0, *size.height() as f32 / 2.0)
    }
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
}

pub fn unwrap_options_to_option_vec<T: Clone>(input: &[Option<T>]) -> Option<Vec<T>> {
    let res: Vec<T> = input.iter().filter_map(|v| v.to_owned()).collect();
    if res.is_empty() { None } else { Some(res) }
}
