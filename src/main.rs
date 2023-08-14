#![feature(convert_float_to_int)]

mod data;
mod state;
mod surface;

use crate::data::{open_data, DataFile, EasyBox, LuaEntity};
use crate::state::State;
use crate::surface::{pixel::Pixel, surface::Surface};
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;
use std::path::Path;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;

fn main() {
    println!("hello");

    let state = State::new(Path::new("work/state.json"));

    let sources = [
        Path::new("chunk500/filtered-resources.json"),
        Path::new("chunk500/filtered-tiles.json"),
    ];

    let rebuild_data = sources
        .map(|v| state.image_needs_rebuild(v))
        .iter()
        .fold(true, |acc, v| if acc { *v } else { false });
    if rebuild_data {
        println!("Source JSON changed, rebuilding");
        let data = match open_data(sources[0], sources[1]) {
            Ok(v) => v,
            Err(e) => {
                println!("error {}", e);
                return;
            }
        };
    }

    if 1 + 1 == 2 {
        create_image(data);
    } else {
        dump_data(data);
    }
}

fn create_image(data: DataFile) {
    let mut img = Surface::new(data.area_box.width, data.area_box.height);
    let mut entity_metrics: HashMap<String, u32> = HashMap::new();

    println!("Loading {} resources...", data.resource.len());
    translate_entities_to_image(
        &data.resource,
        &data.area_box,
        &mut img,
        &mut entity_metrics,
    );

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

    img.save(Path::new("out2.png.raw"));
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

fn dump_data(data: DataFile) {
    println!("data {}", data.tile.len())
}
