use std::env::args;

use facto_loop_miner_fac_engine::blueprint::converter::decode_blueprint_string;

fn main() {
    let mut args = args();
    args.next().unwrap();

    let bp_string = args.next().expect("Give bp string arg");
    decode_blueprint_string(bp_string).unwrap();
}
