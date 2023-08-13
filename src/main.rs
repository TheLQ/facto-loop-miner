mod data;
mod img;

use crate::data::{open_data, DataFile, EasyBox, LuaEntity};
use crate::img::{color_for_resource, ImageArray};
use num_format::Locale;
use std::path::Path;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const LOCALE: Locale = Locale::en;

fn main() {
    println!("hello");
    let data = match open_data(
        Path::new("chunk500/filtered-resources.json"),
        Path::new("chunk500/filtered-tiles.json"),
    ) {
        Ok(v) => v,
        Err(e) => {
            println!("error {}", e);
            return;
        }
    };
    if 1 + 1 == 2 {
        create_image(data);
    } else {
        dump_data(data);
    }
}

fn create_image(data: DataFile) {
    let mut img = ImageArray::new(data.area_box.width, data.area_box.height);
    let mut printed_warnings = Vec::new();

    println!("Loading {} resources...", data.resource.len());
    translate_entities_to_image(
        &data.resource,
        &data.area_box,
        &mut img,
        &mut printed_warnings,
    );

    println!("Loading {} tiles...", data.tile.len());
    translate_entities_to_image(&data.tile, &data.area_box, &mut img, &mut printed_warnings);

    img.save(Path::new("out2.png"));
}

fn translate_entities_to_image<E>(
    entities: &[E],
    entity_box: &EasyBox,
    img: &mut ImageArray,
    printed_warnings: &mut Vec<String>,
) where
    E: LuaEntity,
{
    for entity in entities {
        match color_for_resource(&entity.name()) {
            Some(color) => img.set_color(
                color,
                entity_box.absolute_x(entity.position().x),
                entity_box.absolute_y(entity.position().y),
            ),
            None => {
                let name = entity.name().to_string();
                if !printed_warnings.contains(&name) {
                    println!("unsupported resource type {}", entity.name());
                    printed_warnings.push(name)
                }
            }
        }
    }
}

fn dump_data(data: DataFile) {
    println!("data {}", data.tile.len())
}
