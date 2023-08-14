use crate::gamedata::lua::{EasyBox, LuaData, LuaEntity};
use crate::surface::pixel::Pixel;
use crate::surface::surface::Surface;
use crate::{LOCALE, TILES_PER_CHUNK};
use num_format::ToFormattedString;
use std::collections::HashMap;
use std::path::Path;

pub fn build_image(data: LuaData, image_path: &Path) {
    let mut img = Surface::new(data.area_box.width, data.area_box.height);
    let mut entity_metrics: HashMap<String, u32> = HashMap::new();

    // println!("Loading {} resources...", data.resource.len());
    // translate_entities_to_image(
    //     &data.resource,
    //     &data.area_box,
    //     &mut img,
    //     &mut entity_metrics,
    // );

    println!("Loading {} tiles...", data.tile.len());
    translate_entities_to_image(&data.tile, &data.area_box, &mut img, &mut entity_metrics);

    for (name, count) in &entity_metrics {
        println!(
            "-- Added {}\t\t{} ",
            name,
            count.to_formatted_string(&LOCALE)
        );
    }

    draw_mega_box(&data.area_box, &mut img, &mut entity_metrics);

    img.save(image_path);
}

fn translate_entities_to_image<E>(
    entities: &[E],
    entity_box: &EasyBox,
    img: &mut Surface,
    entity_metrics: &mut HashMap<String, u32>,
) where
    E: LuaEntity,
{
    for entity in entities {
        img.set_pixel(
            Pixel::parse(entity.name()),
            entity_box.absolute_x_f32(entity.position().x),
            entity_box.absolute_y_f32(entity.position().y),
        );
        increment_metric(entity_metrics, &entity.name());
    }
}

fn draw_mega_box(area_box: &EasyBox, img: &mut Surface, entity_metrics: &mut HashMap<String, u32>) {
    let tiles: isize = 15 * TILES_PER_CHUNK as isize;
    let banner_width = 10;
    let edge_neg = -tiles - banner_width;
    let edge_pos = tiles + banner_width;
    println!("edge {} to {}", edge_neg, edge_pos);
    // lazy way
    for root_x in edge_neg..edge_pos {
        for root_y in edge_neg..edge_pos {
            if (root_x < -tiles || root_x > tiles) && (root_y < -tiles || root_y > tiles) {
                img.set_pixel(
                    Pixel::EdgeWall,
                    area_box.absolute_x_i32(root_x as i32),
                    area_box.absolute_y_i32(root_y as i32),
                );
                increment_metric(entity_metrics, "loop-box");
            }
        }
    }
    increment_metric(entity_metrics, "fff-box");
    println!("megabox?")
}

fn increment_metric(entity_metrics: &mut HashMap<String, u32>, metric_name: &str) {
    entity_metrics
        .entry(metric_name.to_string())
        .and_modify(|v| *v += 1)
        .or_insert(1);
}
