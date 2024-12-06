use std::io::Read;

use base64ct::Encoding;
use flate2::read::ZlibDecoder;

pub fn decode_blueprint_string(bp_string_raw: String) {
    let (version_byte, bp_string) = bp_string_raw.split_at(1);
    let raw_zlib = base64ct::Base64::decode_vec(&bp_string).unwrap();

    let mut raw_json = String::new();
    let mut zlib = ZlibDecoder::new(raw_zlib.as_slice());
    zlib.read_to_string(&mut raw_json).unwrap();

    println!("version {}", version_byte);
    println!("content {}", raw_json);
}
