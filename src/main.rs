#![feature(convert_float_to_int)]

mod gamedata;
mod state;
mod surface;

use crate::gamedata::importer::build_image;
use crate::gamedata::lua::LuaData;
use crate::state::State;
use num_format::Locale;
use num_format::Locale::root;
use std::fs::read_dir;
use std::path::Path;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const LOCALE: Locale = Locale::en;
pub const TILES_PER_CHUNK: usize = 32;

fn main() {
    println!("hello");
    let root_dir = Path::new("work");

    let mut state = State::new(&root_dir.join(Path::new("state.json")));

    let sources = [
        &root_dir.join(Path::new("chunk500/filtered-resources.json")),
        &root_dir.join(Path::new("chunk500/filtered-tiles.json")),
    ];

    let rebuild_data = sources
        .map(|v| state.image_needs_rebuild(v))
        .iter()
        .fold(true, |acc, v| if acc { *v } else { false });
    let output_dir = root_dir.join(Path::new("map0"));
    if rebuild_data {
        println!("Source JSON changed, rebuilding");

        panic_if_output_not_empty(&output_dir);

        let data = LuaData::open(sources[0], sources[1]);

        if 1 + 1 == 2 {
            build_image(data, &output_dir.join("run00.rgb"));
        } else {
            dump_data(data);
        }
        state.disk_write();
    } else {
        todo!()
    }
}

fn panic_if_output_not_empty(dir: &Path) {
    let raw_entries = read_dir(dir).unwrap();
    let children_count: Vec<String> = raw_entries
        .into_iter()
        .map(|v| v.unwrap().file_name().into_string().unwrap())
        .collect();
    if !children_count.is_empty() {
        panic!(
            "output path {} not empty with files {}",
            dir.display(),
            children_count.join(" ")
        )
    }
}

fn dump_data(data: LuaData) {
    println!("data {}", data.tile.len())
}
