use crate::gamedata::lua::{LuaData, LuaEntity};
use crate::state::machine::{Step, StepParams};
use crate::surface::game_locator::GameLocator;
use crate::surface::surface::Surface;
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
        let lua_dir = Path::new("work/chunk500");
        let data = LuaData::open(
            &lua_dir.join("filtered-resources.json"),
            &lua_dir.join("filtered-tiles.json"),
        );
        let mut area_box = GameLocator::default();
        area_box.expand_to(&data.resource);
        area_box.expand_to(&data.tile);

        let mut surface = Surface::new(area_box.width + 1, area_box.height);
        surface.area_box = area_box;

        tracing::debug!("Loading {} resources...", data.resource.len());
        translate_entities_to_image(&data.resource, &mut surface, &params);

        tracing::debug!("Loading {} tiles...", data.tile.len());
        translate_entities_to_image(&data.tile, &mut surface, &params);

        surface.save(&params.step_out_dir);
    }
}

fn translate_entities_to_image<E>(entities: &[E], img: &mut Surface, params: &StepParams)
where
    E: LuaEntity,
{
    for entity in entities {
        img.set_pixel(
            entity.name().clone(),
            img.area_box.game_centered_x_f32(entity.position().x),
            img.area_box.game_centered_y_f32(entity.position().y),
        );
        params
            .metrics
            .borrow_mut()
            .increment(entity.name().as_ref());
    }
}
