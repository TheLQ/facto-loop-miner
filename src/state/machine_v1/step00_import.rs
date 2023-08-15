use crate::gamedata::lua::{EasyBox, LuaData, LuaEntity};
use crate::state::machine::Step;
use crate::surface::metric::Metrics;
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::LOCALE;
use num_format::ToFormattedString;
use std::collections::HashMap;
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

    fn dependency_files(&self) -> Option<Vec<PathBuf>> {
        Some(self.input_files.clone())
    }

    fn transformer(&self, surface: &mut Surface, data: &mut LuaData, metrics: &mut Metrics) {
        *surface = Surface::new(data.area_box.width, data.area_box.height);

        println!("Loading {} resources...", data.resource.len());
        translate_entities_to_image(&data.resource, &data.area_box, surface, metrics);

        println!("Loading {} tiles...", data.tile.len());
        translate_entities_to_image(&data.tile, &data.area_box, surface, metrics);
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
    metrics: &mut Metrics,
) where
    E: LuaEntity,
{
    for entity in entities {
        img.set_pixel(
            Pixel::parse(entity.name()),
            entity_box.absolute_x_f32(entity.position().x),
            entity_box.absolute_y_f32(entity.position().y),
        );
        metrics.increment(&entity.name());
    }
}
