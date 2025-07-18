use crate::gamedata::compressed_export::parse_exported_lua_data;
use crate::surface::pixel::Pixel;
use crate::surfacev::fast_metrics::{FastMetric, FastMetrics};
use facto_loop_miner_common::LOCALE;
use facto_loop_miner_common::duration::BasicWatch;
use facto_loop_miner_fac_engine::blueprint::bpfac::position::FacBpPosition;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use std::fs::read;
use std::path::Path;
use tracing::{debug, info};

pub fn read_lua_tiles(input_dir: &Path) -> Vec<LuaEntity> {
    let read_watch = BasicWatch::start();
    let input_path = input_dir.join("big-entities-a.json");
    let mut raw_input = read(&input_path).unwrap();
    debug!(
        "-- Read Lua export JSON {} in {}",
        input_path.display(),
        read_watch
    );

    // let data_inner: ExportCompressedVec = open_data_file(&input_path);
    // let data_inner: ExportCompressedv2 = open_data_file(&input_path, raw_input);
    let entities = parse_exported_lua_data(&mut raw_input, |name, x, y| LuaEntity {
        name: Pixel::from_string(&name).unwrap(),
        position: FacBpPosition { x, y },
    })
    .unwrap();
    info!(
        "-- Loaded Lua {} entities in {}",
        entities.len().to_formatted_string(&LOCALE),
        read_watch
    );

    let mut metric = FastMetrics::new("lua load init".to_string());
    for entity in &entities {
        metric.increment(FastMetric::VSurface_Pixel(entity.name));
    }
    metric.log_final();

    // debug!("-- sample 0 {:?}", data.entities[0]);
    // debug!("-- sample 1 {:?}", data.entities[1]);

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

    entities
}

pub trait LuaThing {
    fn name(&self) -> &Pixel;
    fn position(&self) -> &FacBpPosition;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuaEntity {
    // #[serde(rename = "type")]
    // pub lua_type: String,
    pub name: Pixel,
    #[serde(rename = "pos")]
    pub position: FacBpPosition,
}

impl LuaThing for LuaEntity {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &FacBpPosition {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct LuaTile {
    pub name: Pixel,
    pub position: FacBpPosition,
}

impl LuaThing for LuaTile {
    fn name(&self) -> &Pixel {
        &self.name
    }
    fn position(&self) -> &FacBpPosition {
        &self.position
    }
}
