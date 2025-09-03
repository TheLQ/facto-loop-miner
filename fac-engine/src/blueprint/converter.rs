use std::backtrace::Backtrace;
use std::borrow::BorrowMut;
use std::io::{Read, Write};

use crate::blueprint::bpfac::blueprint::FacBpBlueprintWrapper;
use base64ct::{Base64, Encoding};
use flate2::{Compression, read::ZlibDecoder};
use thiserror::Error;

const VERSION_BYTE: &str = "0";

/// https://wiki.factorio.com/Blueprint_string_format
pub fn decode_blueprint_string(
    bp_string_raw: impl AsRef<str>,
) -> ConvertResult<FacBpBlueprintWrapper> {
    let (version_byte, bp_string) = bp_string_raw.as_ref().split_at(1);
    assert_eq!(version_byte, VERSION_BYTE, "unexpected Blueprint version");

    let compressed = Base64::decode_vec(bp_string).expect("Valid Base64?");

    let mut raw_json = String::new();
    let mut zlib = ZlibDecoder::new(compressed.as_slice());
    zlib.read_to_string(&mut raw_json)?;

    // println!("JSON: {}", raw_json);

    Ok(serde_json::from_str(&raw_json)?)
}

/// https://wiki.factorio.com/Blueprint_string_format
pub fn encode_blueprint_to_string_dangerous_index(
    blueprint: &FacBpBlueprintWrapper,
) -> ConvertResult<String> {
    _encode_blueprint_to_string(blueprint)
}

pub fn encode_blueprint_to_string_auto_index(
    mut blueprint: impl BorrowMut<FacBpBlueprintWrapper>,
) -> ConvertResult<String> {
    let blueprint = blueprint.borrow_mut();
    let mut auto_index = /*lua...*/1;
    for entity in &mut blueprint.blueprint.entities {
        if entity.entity_number.is_none() {
            entity.entity_number = Some(auto_index);
            auto_index += 1;
        } else {
            panic!("TODO: existing number")
        }
    }
    _encode_blueprint_to_string(blueprint)
}

fn _encode_blueprint_to_string(blueprint: &FacBpBlueprintWrapper) -> ConvertResult<String> {
    let json = serde_json::to_string(blueprint)?;
    // println!("JSONify {}", json);

    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
    zlib.write_all(json.as_bytes())?;
    let compressed = zlib.finish()?;

    let mut encoded = Base64::encode_string(&compressed);
    encoded.insert_str(0, VERSION_BYTE);
    Ok(encoded)
}

pub type ConvertResult<T> = Result<T, ConvertError>;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Serde {e:?}")]
    Serde {
        #[from]
        e: serde_json::Error,
        backtrace: Backtrace,
    },
    #[error("IO {e:?}")]
    IO {
        #[from]
        e: std::io::Error,
        backtrace: Backtrace,
    },
}

#[cfg(test)]
mod test {
    use std::{fs::File, path::Path};

    use super::{decode_blueprint_string, encode_blueprint_to_string_dangerous_index};

    #[test]
    fn round_trip_basic() {
        let input = include_str!("../../test_blueprints/rail_station_skeleton");

        let decoded = decode_blueprint_string(input).unwrap();

        // We cannot compare the raw base64 due to ordering differences? Instead compare the structs
        // (may by useless now though)
        let recoded =
            decode_blueprint_string(encode_blueprint_to_string_dangerous_index(&decoded).unwrap())
                .unwrap();

        if decoded != recoded {
            let decoded_json_path = Path::new("debug-decoded.json");
            serde_json::to_writer_pretty(&File::create(decoded_json_path).unwrap(), &decoded)
                .unwrap();

            let encoded_json_path = Path::new("debug-encoded.json");
            serde_json::to_writer_pretty(&File::create(encoded_json_path).unwrap(), &decoded)
                .unwrap();

            panic!(
                "bad output, see {} and {}",
                decoded_json_path.display(),
                encoded_json_path.display(),
            )
        }
    }
}
