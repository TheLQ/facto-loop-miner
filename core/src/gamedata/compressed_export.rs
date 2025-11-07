use crate::surface::pixel::Pixel;
use crate::surfacev::err::VResult;
use facto_loop_miner_common::err_bt::PrettyUnwrapMyBacktrace;
use serde::Deserialize;
use simd_json::{BorrowedValue, OwnedValue, StaticNode, to_borrowed_value, to_owned_value};
use std::io;
use std::io::ErrorKind;

/// Parse compressed JSON from loop-miner-exporter mod in format `[name1, x1, y1, name2, ....]`
///
/// On 45 million entities, simd_json in raw borrowed mode parses in 9 seconds.
/// Serde simd_json with conversion takes 30 seconds.
pub fn parse_exported_lua_data<V, C>(input: &mut [u8], to_object: C) -> VResult<Vec<V>>
where
    C: Fn(Pixel, f32, f32) -> V,
{
    match 2 {
        // 0 => parse_exported_lua_data_simd_borrowed(input, to_object),
        2 => parse_exported_lua_data_simd_owned(input, to_object),
        _ => panic!("unknown"),
    }
}

fn parse_exported_lua_data_simd_borrowed<V, C>(input: &mut [u8], to_object: C) -> VResult<Vec<V>>
where
    C: Fn(Pixel, f32, f32) -> V,
{
    let main_array = if let Ok(BorrowedValue::Array(raw)) = to_borrowed_value(input) {
        raw
    } else {
        panic!("no wrapper array?")
    };

    let mut result = Vec::new();
    for [name_value, x_value, y_value] in main_array.into_iter().array_chunks() {
        let name = if let BorrowedValue::String(raw) = name_value {
            raw
        } else {
            panic!("not a name");
        };
        let x = if let BorrowedValue::Static(StaticNode::F64(raw)) = x_value {
            raw as f32
        } else {
            panic!("not x");
        };
        let y = if let BorrowedValue::Static(StaticNode::F64(raw)) = y_value {
            raw as f32
        } else {
            panic!("not y");
        };
        result.push(to_object(Pixel::from_string(&name).pretty_unwrap(), x, y));
    }
    Ok(result)
}

fn parse_exported_lua_data_simd_owned<V, C>(input: &mut [u8], to_object: C) -> VResult<Vec<V>>
where
    C: Fn(Pixel, f32, f32) -> V,
{
    let mut result = Vec::new();

    let main_array = if let Ok(OwnedValue::Array(raw)) = to_owned_value(input) {
        raw
    } else {
        panic!("no wrapper array?")
    };

    for [name_value, x_value, y_value] in main_array.into_iter().array_chunks() {
        let name = if let OwnedValue::String(raw) = name_value {
            raw
        } else {
            panic!("not a name");
        };
        let x = if let OwnedValue::Static(StaticNode::F64(raw)) = x_value {
            raw as f32
        } else {
            panic!("not x");
        };
        let y = if let OwnedValue::Static(StaticNode::F64(raw)) = y_value {
            raw as f32
        } else {
            panic!("not y");
        };
        result.push(to_object(Pixel::from_string(&name).pretty_unwrap(), x, y));
    }

    Ok(result)
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ExportCompressedItem {
    Name(String),
    Position(f32),
}

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

    fn as_string(&self) -> &str {
        match self {
            ExportCompressedItem::Name(v) => v,
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
struct ExportCompressedVec {
    inner: Vec<ExportCompressedItem>,
}

impl ExportCompressedVec {
    fn item_chunks<C, V>(self, to_object: &mut C) -> impl Iterator<Item = V> + '_
    where
        C: Fn(String, f32, f32) -> V,
    {
        if !self.inner.len().is_multiple_of(3) {
            panic!("Unexpected data");
        }
        self.inner
            .into_iter()
            .array_chunks()
            .map(|[name_raw, x_raw, y_raw]| {
                let name = name_raw.as_string().to_string();
                let x = x_raw.into_f32();
                let y = y_raw.into_f32();
                to_object(name, x, y)
            })
    }
}
