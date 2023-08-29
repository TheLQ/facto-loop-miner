use crate::LOCALE;
use num_format::ToFormattedString;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
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
        println!("-- Opened Data file in {} seconds", duration.as_secs());
        println!("-- {} Tile", data.tile.len().to_formatted_string(&LOCALE),);
        println!(
            "-- {} Resource",
            data.resource.len().to_formatted_string(&LOCALE),
        );
        data
    }
}

pub trait LuaEntity {
    fn name(&self) -> &str;
    fn position(&self) -> &Position;
}

#[derive(Serialize, Deserialize)]
pub struct LuaResource {
    #[serde(rename = "type")]
    pub lua_type: String,
    pub name: String,
    pub position: Position,
}

impl LuaEntity for LuaResource {
    fn name(&self) -> &str {
        &self.name
    }
    fn position(&self) -> &Position {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct LuaTile {
    pub name: String,
    pub position: Position,
}

impl LuaEntity for LuaTile {
    fn name(&self) -> &str {
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
    println!("Reading entity data {} ...", path.display());
    let file = File::open(path).unwrap();
    let buf_reader = BufReader::new(file);
    let result = simd_json::serde::from_reader(buf_reader).unwrap();
    result
}
