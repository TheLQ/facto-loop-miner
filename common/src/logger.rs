use xana_commons_rs::{MapHugeCrateName, XanaCommonsLogConfig};

const TRACE_NO_ADMIRAL_NETWORK: &str = "trace,\
facto_loop_miner_fac_engine::admiral::executor::client=debug,\
facto_loop_miner_fac_engine::admiral::lua_command::lua_batch=debug,\
facto_loop_miner_fac_engine::game_blocks::rail_hope_single=debug";

pub fn log_init_trace() {
    log_config(TRACE_NO_ADMIRAL_NETWORK).log_init_trace()
}

pub fn log_init_debug() {
    log_config("").log_init_debug()
}

fn log_config(extra_filter_env: &'static str) -> XanaCommonsLogConfig<FactoLogConfig> {
    XanaCommonsLogConfig::new_map_huge()
        .with_extra_filter_env(extra_filter_env)
        .with_filter_non_main_threads(true)
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
