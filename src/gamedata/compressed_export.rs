use crate::gamedata::lua::{LuaEntity, LuaPoint};
use crate::surface::pixel::Pixel;
use crate::surfacev::err::VResult;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use simd_json::{to_borrowed_value, BorrowedValue, StaticNode};
use std::fmt::Formatter;
use std::io;
use std::io::ErrorKind;
use std::marker::PhantomData;

pub fn parse_exported_lua_data(input: &mut [u8]) -> VResult<Vec<LuaEntity>> {
    let mut result = Vec::new();

    let main_array = if let BorrowedValue::Array(raw) = simd_json::to_borrowed_value(input).unwrap()
    {
        raw
    } else {
        panic!("no wrapper array?")
    };

    for [name_value, x_value, y_value] in main_array.array_chunks() {
        let name = if let BorrowedValue::String(raw) = name_value {
            raw
        } else {
            panic!("not a name");
        };
        let x = if let BorrowedValue::Static(StaticNode::F64(raw)) = x_value {
            *raw as f32
        } else {
            panic!("not a name");
        };
        let y = if let BorrowedValue::Static(StaticNode::F64(raw)) = y_value {
            *raw as f32
        } else {
            panic!("not a name");
        };
        result.push(LuaEntity {
            name: Pixel::from_string(name)?,
            position: LuaPoint { x, y },
        });
    }

    Ok(result)
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ExportCompressedItemSuperv2 {
    Entry(String, f32, f32),
}

#[derive(Deserialize, Debug)]
pub struct ExportCompressedv2 {
    pub inner: Vec<ExportCompressedItemSuperv2>,
}

impl ExportCompressedv2 {
    pub fn item_chunks(self) -> impl Iterator<Item = LuaEntity> {
        if self.inner.len() % 3 != 0 {
            panic!("Unexpected data");
        }
        self.inner.into_iter().map(|v| match v {
            ExportCompressedItemSuperv2::Entry(name, x, y) => LuaEntity {
                name: Pixel::from_string(&name).unwrap(),
                position: LuaPoint { x, y },
            },
        })
    }
}

// fn deserialize_max<'de, T, D, E>(deserializer: D) -> Result<T, D::Error>
// where
//     T: Deserialize<'de> ,
//     D: Deserializer<'de>,
//     E: LuaEntity,
// {
//     struct MaxVisitor<T>(PhantomData<fn() -> T>);
//
//     impl<'de, T> Visitor<'de> for MaxVisitor<T>
//     where
//         T: Deserialize<'de> + Ord,
//     {
//         type Value = T;
//
//         fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
//             formatter.write_str("a flattened array")
//         }
//
//         fn visit_seq<A>(self, mut seq: A) -> Result<T, A::Error>
//         where
//             A: SeqAccess<'de>,
//         {
//             let mut result = Vec::new();
//             while let Some(name) = seq.next_element()? {
//                 if let Some(ExportCompressedItem::Position(x)) = seq.next_element()? {
//                     if let Some(ExportCompressedItem::Position(y)) = seq.next_element()? {
//                         result.push(LuaEntity {
//                             name,
//                             position: LuaPoint { x, y },
//                         })
//                     }
//                 }
//             }
//             Ok(result)
//         }
//     }
//
//     let visitor = MaxVisitor(PhantomData);
//     deserializer.deserialize_seq(visitor)
// }
