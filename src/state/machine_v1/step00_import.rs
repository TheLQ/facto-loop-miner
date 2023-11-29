use crate::gamedata::lua::{LuaData, LuaPoint, LuaThing};
use crate::state::machine::{Step, StepParams};
use crate::surface::easier_box::EasierBox;
use crate::surface::game_locator::GameLocator;
use crate::surface::surface::Surface;
use crate::surfacev::err::VResult;
use crate::surfacev::vpoint::VPoint;
use crate::surfacev::vsurface::VSurface;
use opencv::core::{Point, Point2f};
use std::cell::Cell;
use std::path::Path;
use tracing::info;

pub struct Step00 {}

impl Step00 {
    pub(crate) fn new() -> Box<dyn Step> {
        Box::new(Step00 {})
    }
}

impl Step for Step00 {
    fn name(&self) -> String {
        "step00-import".to_string()
    }

    /// Load Factorio Mod's exported map data JSON into a huge single image
    /// representing the whole map.
    fn transformer(&self, params: StepParams) {
        let lua_dir = Path::new("work/chunk1000");

        let data = LuaData::open(lua_dir);
        let radius = find_radius(&data) as u32;
        let mut surface = VSurface::new(radius);

        tracing::debug!("Loading {} resources...", data.entities.len());
        translate_entities_to_image(&data.entities, &mut surface, &params);

        tracing::debug!("Loading {} tiles...", data.tiles.len());
        translate_entities_to_image(&data.tiles, &mut surface, &params);

        surface.save(&params.step_out_dir);
    }
}

fn find_radius(data: &LuaData) -> f32 {
    let mut bottom_left = LuaPoint { x: 0.0, y: 0.0 };
    let mut top_right = LuaPoint { x: 0.0, y: 0.0 };
    find_radius_max(&data.entities, &mut bottom_left, &mut top_right);
    find_radius_max(&data.tiles, &mut bottom_left, &mut top_right);

    info!(
        "Defined box top_right {:?} bottom_left {:?}",
        top_right, bottom_left
    );

    let mut max_radius = 0.0f32;
    max_radius = max_radius.max(bottom_left.x.abs());
    max_radius = max_radius.max(bottom_left.y.abs());
    max_radius = max_radius.max(top_right.x);
    max_radius = max_radius.max(top_right.y);

    // spacing
    max_radius += 10.0;

    max_radius
}
fn find_radius_max(
    things: &Vec<impl LuaThing>,
    bottom_left: &mut LuaPoint,
    top_right: &mut LuaPoint,
) {
    for thing in things {
        let pos = thing.position();
        *bottom_left = LuaPoint {
            x: bottom_left.x.min(pos.x),
            y: bottom_left.y.min(pos.y),
        };
        *top_right = LuaPoint {
            x: top_right.x.max(pos.x),
            y: top_right.y.max(pos.y),
        };
    }
}

fn translate_entities_to_image<E>(
    entities: &[E],
    surface: &mut VSurface,
    params: &StepParams,
) -> VResult<()>
where
    E: LuaThing,
{
    for entity in entities {
        surface.set_pixel(
            VPoint::from_f32_with_offset(entity.position().to_point2f(), 0.5)?,
            entity.name().clone(),
        );
        params
            .metrics
            .borrow_mut()
            .increment(entity.name().as_ref());
    }
    Ok(())
}
