use std::io::{Read, Write};

use base64ct::{Base64, Encoding};
use flate2::{Compression, read::ZlibDecoder};

use crate::{
    blueprint::bpfac::blueprint::{BpFacBlueprint, BpFacBlueprintWrapper},
    err::{FError, FResult},
};

const VERSION_BYTE: &str = "0";

pub fn decode_blueprint_string(bp_string_raw: impl AsRef<str>) -> FResult<BpFacBlueprintWrapper> {
    let (version_byte, bp_string) = bp_string_raw.as_ref().split_at(1);
    assert_eq!(version_byte, VERSION_BYTE, "unexpected Blueprint version");

    let compressed = base64ct::Base64::decode_vec(&bp_string).unwrap();

    let mut raw_json = String::new();
    let mut zlib = ZlibDecoder::new(compressed.as_slice());
    zlib.read_to_string(&mut raw_json).unwrap();

    Ok(serde_json::from_str(&raw_json)?)
}

pub fn encode_blueprint_to_string(blueprint: &BpFacBlueprintWrapper) -> FResult<String> {
    let json = serde_json::to_string(blueprint)?;

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

    use super::{decode_blueprint_string, encode_blueprint_to_string};

    #[test]
    fn round_trip_basic() {
        let input = include_str!("../../test_blueprints/rail_station_skeleton");

        let decoded = decode_blueprint_string(input).unwrap();

        // We cannot compare the raw base64 due to ordering differences? Instead compare the structs
        // (may by useless now though)
        let recoded =
            decode_blueprint_string(encode_blueprint_to_string(&decoded).unwrap()).unwrap();

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
