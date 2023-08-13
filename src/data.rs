use num_format::{Locale, ToFormattedString};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

const LOCALE: Locale = Locale::en;

#[derive(Serialize, Deserialize)]
pub struct DataFile {
    pub resource: Vec<LuaResource>,
    pub tile: Vec<LuaTile>,
    #[serde(default)]
    pub area_box: EasyBox,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct EasyBox {
    pub max_x: i32,
    pub max_y: i32,
    pub min_x: i32,
    pub min_y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for EasyBox {
    fn default() -> Self {
        EasyBox {
            max_x: 0,
            max_y: 0,
            min_x: 0,
            min_y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl EasyBox {
    pub fn absolute_x(&self, game_center_x: f32) -> u32 {
        (game_center_x.floor() as i32 - self.min_x) as u32
    }

    pub fn absolute_y(&self, game_center_y: f32) -> u32 {
        (game_center_y.floor() as i32 - self.min_x) as u32
    }
}

pub fn open_data(resource: &Path, tile: &Path) -> Result<DataFile, Box<dyn Error>> {
    let start_time = Instant::now();

    let mut data = DataFile {
        area_box: EasyBox::default(),
        resource: open_data_file::<Vec<LuaResource>>(resource),
        tile: open_data_file::<Vec<LuaTile>>(tile),
    };
    println!("Reading Complete");

    find_edge_box(&data.resource, &mut data.area_box);
    find_edge_box(&data.tile, &mut data.area_box);

    let duration = Instant::now() - start_time;
    println!("-- Opened Data file in {} seconds", duration.as_secs());
    println!("-- {} Tile", data.tile.len().to_formatted_string(&LOCALE),);
    println!(
        "-- {} Resource",
        data.resource.len().to_formatted_string(&LOCALE),
    );
    println!("-- {:?}", data.area_box);
    Ok(data)
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

fn find_edge_box<E>(entities: &[E], entity_box: &mut EasyBox)
where
    E: LuaEntity,
{
    for entity in entities {
        entity_box.max_x = max(entity_box.max_x, entity.position().x.round() as i32);
        entity_box.max_y = max(entity_box.max_y, entity.position().y.round() as i32);
        entity_box.min_x = min(entity_box.min_x, entity.position().x.round() as i32);
        entity_box.min_y = min(entity_box.min_y, entity.position().y.round() as i32);
    }
    entity_box.width = (entity_box.max_x - entity_box.min_x).try_into().unwrap();
    entity_box.height = (entity_box.max_y - entity_box.min_y).try_into().unwrap();
}
