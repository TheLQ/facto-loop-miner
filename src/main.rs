mod data;
mod img;

use crate::data::open_data;
use crate::img::{color_for_resource, ImageArray};
use std::path::Path;

fn main() {
    println!("hello");
    let data = match open_data(Path::new("filtered-resources2.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("error {}", e);
            return;
        }
    };
    println!("pixels {}", data.resource.len());

    let width = data.resource_box.max_x - data.resource_box.min_x;
    let height = data.resource_box.max_y - data.resource_box.min_y;
    println!("resolution {}x{}", width, height);

    let mut img = ImageArray::new(width, height);

    let mut printed_warnings = Vec::new();
    for resource in data.resource {
        match color_for_resource(&resource.name) {
            Some(color) => img.set_color(
                color,
                resource.position.x.floor() as u32,
                resource.position.y.floor() as u32,
            ),
            None => {
                if !printed_warnings.contains(&resource.name) {
                    println!("unsupported resource type {}", resource.name);
                    printed_warnings.push(resource.name.clone())
                }
            }
        }
    }

    img.save(Path::new("out2.png"));
}
