use xana_commons_rs::{MapHugeCrateName, XanaCommonsLogConfig};

const TRACE_NO_ADMIRAL_NETWORK: [&str; 3] = [
    "facto_loop_miner_fac_engine::admiral::executor::client=debug",
    "facto_loop_miner_fac_engine::admiral::lua_command::lua_batch=debug",
    "facto_loop_miner_fac_engine::game_blocks::rail_hope_single=debug",
];

pub fn log_init_trace() {
    log_config(&[], false).log_init_trace()
}

pub fn log_init_trace_no_main_treads() {
    log_config(&[], true).log_init_trace()
}

pub fn log_init_debug() {
    log_config(&[], false).log_init_debug()
}

fn log_config(
    extra_filter_env: &'static [&'static str],
    filter_non_main_threads: bool,
) -> XanaCommonsLogConfig<FactoLogConfig> {
    XanaCommonsLogConfig::new_map_huge()
        .with_extra_filter_env(extra_filter_env)
        .with_filter_non_main_threads(filter_non_main_threads)
}

struct FactoLogConfig;
impl MapHugeCrateName for FactoLogConfig {
    fn map_huge(input: &str) -> Option<&'static str> {
        match input {
            "facto_loop_miner" => Some("core"),
            "facto_loop_miner_io" => Some("io"),
            "facto_loop_miner_fac_engine" => Some("engine"),
            _ => None,
        }
    }
}
