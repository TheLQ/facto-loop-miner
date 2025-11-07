use std::cell::LazyCell;
use std::iter::IntoIterator;
use xana_commons_rs::XanaCommonsLogConfig;

const TRACE_NO_ADMIRAL_NETWORK: &str = "trace,\
facto_loop_miner_fac_engine::admiral::executor::client=debug,\
facto_loop_miner_fac_engine::admiral::lua_command::lua_batch=debug,\
facto_loop_miner_fac_engine::game_blocks::rail_hope_single=debug";

const FILTER_NON_MAIN_THREADS: bool = true;

pub fn log_init_trace() {
    xana_commons_rs::log_init_trace(log_config())
}

pub fn log_init_debug() {
    xana_commons_rs::log_init_debug(log_config())
}

fn log_config() -> XanaCommonsLogConfig {
    XanaCommonsLogConfig {
        map_huge_crate_names: [
            ("facto_loop_miner", "core"),
            ("facto_loop_miner_io", "io"),
            ("facto_loop_miner_fac_engine", "engine"),
        ]
        .into_iter()
        .collect(),
        filter_non_main_threads: false,
        extra_filter_env: "",
    }
}
