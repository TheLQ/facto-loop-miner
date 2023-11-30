use crate::gamedata::lua::{LuaEntity, LuaPoint};
use crate::surface::pixel::Pixel;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::ErrorKind;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ExportCompressedItem {
    Name(String),
    Position(f32),
}

#[allow(clippy::match_wildcard_for_single_variants)]
impl ExportCompressedItem {
    fn into_pixel(self) -> Pixel {
        match self {
            ExportCompressedItem::Name(v) => Pixel::from_string(&v)
                .map_err(|_| io::Error::new(ErrorKind::NotFound, v))
                .unwrap(),
            _ => {
                panic!("unexpected value")
            }
        }
    }

    fn into_f32(self) -> f32 {
        match self {
            ExportCompressedItem::Position(v) => v,
            _ => {
                panic!("unexpected value")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ExportCompressedVec {
    inner: Vec<ExportCompressedItem>,
}

impl ExportCompressedVec {
    pub fn item_chunks(self) -> impl Iterator<Item = LuaEntity> {
        if self.inner.len() % 3 != 0 {
            panic!("Unexpected data");
        }
        self.inner
            .into_iter()
            .array_chunks()
            .map(|[name_raw, x_raw, y_raw]| {
                let name = name_raw.into_pixel();
                let x = x_raw.into_f32();
                let y = y_raw.into_f32();
                LuaEntity {
                    name,
                    position: LuaPoint { x, y },
                }
            })
    }
}
