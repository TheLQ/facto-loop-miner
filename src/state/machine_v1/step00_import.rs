use crate::gamedata::lua::{LuaData, LuaEntity};
use crate::state::machine::{Step, StepParams};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use std::path::{Path, PathBuf};

pub struct Step00 {
    input_files: Vec<PathBuf>,
}

impl Step00 {
    pub fn new(root_dir: &Path) -> Box<dyn Step> {
        Box::new(Step00 {
            input_files: Vec::from([
                root_dir.join("chunk500/filtered-resources.json"),
                root_dir.join("chunk500/filtered-tiles.json"),
            ]),
        })
    }
}

impl Step for Step00 {
    fn name(&self) -> String {
        "step00-import".to_string()
    }

    fn transformer(&self, params: StepParams) {
        let lua_dir = Path::new("work/chunk500");
        let data = LuaData::open(
            &lua_dir.join("filtered-resources.json"),
            &lua_dir.join("filtered-tiles.json"),
        );

        let mut surface = Surface::new(data.area_box.width, data.area_box.height);

        println!("Loading {} resources...", data.resource.len());
        translate_entities_to_image(&data.resource, &data.area_box, &mut surface, &params);

        println!("Loading {} tiles...", data.tile.len());
        translate_entities_to_image(&data.tile, &data.area_box, &mut surface, &params);

        surface.save(&params.step_out_dir);
    }

    // fn force_transformer_run(&self) {
    //     let rebuild_data = self.input_files
    //         .map(|v| state.image_needs_rebuild(v))
    //         .iter()
    //         .fold(true, |acc, v| if acc { *v } else { false });
    //     let output_dir = root_dir.join(Path::new("map0"));
    // }
}

fn translate_entities_to_image<E>(
    entities: &[E],
    entity_box: &EasyBox,
    img: &mut Surface,
    params: &StepParams,
) where
    E: LuaEntity,
{
    for entity in entities {
        img.set_pixel(
            Pixel::parse(entity.name()),
            entity_box.absolute_x_f32(entity.position().x),
            entity_box.absolute_y_f32(entity.position().y),
        );
        params.metrics.borrow_mut().increment(&entity.name());
    }
}
