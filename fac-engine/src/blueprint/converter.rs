use std::borrow::BorrowMut;
use std::io::{Read, Write};

use base64ct::{Base64, Encoding};
use flate2::{Compression, read::ZlibDecoder};

use crate::{blueprint::bpfac::blueprint::FacBpBlueprintWrapper, err::FResult};

const VERSION_BYTE: &str = "0";

/// https://wiki.factorio.com/Blueprint_string_format
pub fn decode_blueprint_string(bp_string_raw: impl AsRef<str>) -> FResult<FacBpBlueprintWrapper> {
    let (version_byte, bp_string) = bp_string_raw.as_ref().split_at(1);
    assert_eq!(version_byte, VERSION_BYTE, "unexpected Blueprint version");

    let compressed = Base64::decode_vec(bp_string).expect("Valid Base64?");

    let mut raw_json = String::new();
    let mut zlib = ZlibDecoder::new(compressed.as_slice());
    zlib.read_to_string(&mut raw_json)
        .expect("Valid zlib after base64?");

    // println!("JSON: {}", raw_json);

    Ok(serde_json::from_str(&raw_json)?)
}

/// https://wiki.factorio.com/Blueprint_string_format
pub fn encode_blueprint_to_string_dangerous_index(
    blueprint: &FacBpBlueprintWrapper,
) -> FResult<String> {
    _encode_blueprint_to_string(blueprint)
}

pub fn encode_blueprint_to_string_auto_index(
    mut blueprint: impl BorrowMut<FacBpBlueprintWrapper>,
) -> FResult<String> {
    let blueprint = blueprint.borrow_mut();
    let mut auto_index = /*lua...*/1;
    for entity in &mut blueprint.blueprint.entities {
        if entity.entity_number == None {
            entity.entity_number = Some(auto_index);
            auto_index += 1;
        } else {
            panic!("TODO: existing number")
        }
    }
    _encode_blueprint_to_string(blueprint)
}

fn _encode_blueprint_to_string(blueprint: &FacBpBlueprintWrapper) -> FResult<String> {
    let json = serde_json::to_string(blueprint)?;
    // println!("JSONify {}", json);

    let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
    zlib.write_all(json.as_bytes()).unwrap();
    let compressed = zlib.finish().unwrap();

    let mut encoded = Base64::encode_string(&compressed);
    encoded.insert_str(0, VERSION_BYTE);
    Ok(encoded)
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
