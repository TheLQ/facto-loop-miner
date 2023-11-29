use crate::gamedata::compressed_export::ExportCompressedVec;
use crate::surface::pixel::Pixel;
use crate::LOCALE;
use num_format::ToFormattedString;
use opencv::core::Point2f;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::read;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

#[derive(Serialize, Deserialize)]
pub struct LuaData {
    pub entities: Vec<LuaEntity>,
    pub tiles: Vec<LuaTile>,
}

impl LuaData {
    pub fn open(input_dir: &Path) -> Self {
        let start_time = Instant::now();

        let input_path = input_dir.join("big-entities-a.json");
        let data_inner: ExportCompressedVec = open_data_file(&input_path);

        let data = LuaData {
            tiles: Vec::new(),
            entities: data_inner.item_chunks().collect(),
        };
        info!("Read {} items", data.entities.len());

        let duration = Instant::now() - start_time;
        debug!("-- Opened Data file in {} seconds", duration.as_secs());
        debug!("-- {} Tile", data.tiles.len().to_formatted_string(&LOCALE));
        debug!(
            "-- {} Resource",
            data.entities.len().to_formatted_string(&LOCALE),
        );
        debug!("-- sample 0 {:?}", data.entities[0]);
        debug!("-- sample 1 {:?}", data.entities[1]);

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

pub trait LuaThing {
    fn name(&self) -> &Pixel;
    fn position(&self) -> &LuaPoint;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuaEntity {
    // #[serde(rename = "type")]
    // pub lua_type: String,
    pub name: Pixel,
    #[serde(rename = "pos")]
    pub position: LuaPoint,
}

impl LuaThing for LuaEntity {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &LuaPoint {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct LuaTile {
    pub name: Pixel,
    pub position: LuaPoint,
}

impl LuaThing for LuaTile {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &LuaPoint {
        &self.position
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuaPoint {
    pub x: f32,
    pub y: f32,
}

impl LuaPoint {
    pub fn to_point2f(&self) -> Point2f {
        Point2f {
            x: self.x,
            y: self.y,
        }
    }
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
