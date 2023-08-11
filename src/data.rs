use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct DataFile {
    pub resource: Vec<LuaResource>,
    // pub tile: Vec<LuaTile>,
    #[serde(default)]
    pub resource_box: EasyBox,
    // pub tile_box: EasyBox,
}

#[derive(Serialize, Deserialize)]
pub struct LuaResource {
    #[serde(rename = "type")]
    pub lua_type: String,
    pub name: String,
    pub position: Position,
}

#[derive(Serialize, Deserialize)]
pub struct LuaTile {
    pub name: String,
    pub position: Position,
}

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EasyBox {
    pub max_x: u32,
    pub max_y: u32,
    pub min_x: u32,
    pub min_y: u32,
}

impl Default for EasyBox {
    fn default() -> Self {
        EasyBox {
            max_x: 0,
            max_y: 0,
            min_x: 0,
            min_y: 0,
        }
    }
}

pub fn open_data(path: &Path) -> Result<DataFile, Box<dyn Error>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let mut data: DataFile = serde_json::from_reader(buf_reader)?;

    for resource in &data.resource {
        data.resource_box.max_x = max(data.resource_box.max_x, resource.position.x.round() as u32);
        data.resource_box.max_y = max(data.resource_box.max_y, resource.position.y.round() as u32);
        data.resource_box.min_x = min(data.resource_box.min_x, resource.position.x.round() as u32);
        data.resource_box.min_y = min(data.resource_box.min_y, resource.position.y.round() as u32);
    }

    Ok(data)
}
