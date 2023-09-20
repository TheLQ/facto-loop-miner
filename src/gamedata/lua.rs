use crate::surface::pixel::Pixel;
use crate::LOCALE;
use num_format::ToFormattedString;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::read;
use std::path::Path;
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct LuaData {
    pub resource: Vec<LuaResource>,
    pub tile: Vec<LuaTile>,
}

impl LuaData {
    pub fn open(resource: &Path, tile: &Path) -> Self {
        let start_time = Instant::now();

        let data = LuaData {
            resource: open_data_file(resource),
            tile: open_data_file(tile),
        };

        let duration = Instant::now() - start_time;
        tracing::debug!("-- Opened Data file in {} seconds", duration.as_secs());
        tracing::debug!("-- {} Tile", data.tile.len().to_formatted_string(&LOCALE));
        tracing::debug!(
            "-- {} Resource",
            data.resource.len().to_formatted_string(&LOCALE),
        );

        // let mut printed: Vec<String> = Vec::new();
        // for tile in &data.tile {
        //     let name = tile.name().to_string();
        //     if !printed.contains(&name) {
        //         tracing::debug!("type {}", &name);
        //         printed.push(name);
        //     }
        // }
        // for tile in &data.resource {
        //     let name = tile.lua_type.to_string();
        //     if !printed.contains(&name) {
        //         tracing::debug!("type {}", &name);
        //         printed.push(name);
        //     }
        // }

        data
    }
}

pub trait LuaEntity {
    fn name(&self) -> &Pixel;
    fn position(&self) -> &Position;
}

#[derive(Serialize, Deserialize)]
pub struct LuaResource {
    // #[serde(rename = "type")]
    // pub lua_type: String,
    pub name: Pixel,
    pub position: Position,
}

impl LuaEntity for LuaResource {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &Position {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct LuaTile {
    pub name: Pixel,
    pub position: Position,
}

impl LuaEntity for LuaTile {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &Position {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

fn open_data_file<T>(path: &Path) -> T
where
    T: DeserializeOwned,
{
    tracing::debug!("Reading entity data {} ...", path.display());
    // let file = File::open(path).unwrap();
    // let buf_reader = BufReader::new(file);
    // let result = simd_json::serde::from_reader(buf_reader).unwrap();

    // nope, slice is mutated
    // let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let mut raw = read(path).unwrap();
    let result = simd_json::serde::from_slice(raw.as_mut_slice()).unwrap();
    result
}
