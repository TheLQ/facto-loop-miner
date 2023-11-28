use crate::gamedata::lua::{LuaData, LuaThing};
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
        let input_path = lua_dir.join("mega-dump.json");

        let data = LuaData::open(&input_path);
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
    let mut bottom_left = Cell::new(Point2f { x: 0.0, y: 0.0 });
    let mut top_right = Cell::new(Point2f { x: 0.0, y: 0.0 });
    find_radius_max(&data.entities, &mut bottom_left, &mut top_right);
    find_radius_max(&data.tiles, &mut bottom_left, &mut top_right);

    let mut max_radius = 0.0f32;
    max_radius = max_radius.max(bottom_left.get().x.abs());
    max_radius = max_radius.max(bottom_left.get().y.abs());
    max_radius = max_radius.max(top_right.get().x);
    max_radius = max_radius.max(top_right.get().y);

    // spacing
    max_radius += 10.0;

    max_radius
}
fn find_radius_max(
    things: &Vec<impl LuaThing>,
    bottom_left: &mut Cell<Point2f>,
    top_right: &mut Cell<Point2f>,
) {
    for thing in things {
        let pos = thing.position();
        let bottom_left_v = bottom_left.get();
        bottom_left.set(Point2f {
            x: bottom_left_v.x.min(pos.x),
            y: bottom_left_v.y.min(pos.y),
        });
        let top_right_v = top_right.get();
        bottom_left.set(Point2f {
            x: top_right_v.x.max(pos.x),
            y: top_right_v.y.max(pos.y),
        });
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
