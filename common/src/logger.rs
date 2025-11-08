use xana_commons_rs::{MapHugeCrateName, XanaCommonsLogConfig};

const TRACE_NO_ADMIRAL_NETWORK: &str = "trace,\
facto_loop_miner_fac_engine::admiral::executor::client=debug,\
facto_loop_miner_fac_engine::admiral::lua_command::lua_batch=debug,\
facto_loop_miner_fac_engine::game_blocks::rail_hope_single=debug";

pub fn log_init_trace() {
    xana_commons_rs::log_init_trace(log_config(Some(TRACE_NO_ADMIRAL_NETWORK)))
}

pub fn log_init_debug() {
    xana_commons_rs::log_init_debug(log_config(None))
}

fn log_config(extra_filter_env: Option<&'static str>) -> XanaCommonsLogConfig<FactoLogConfig> {
    XanaCommonsLogConfig {
        map_huge_crate_names: Some(FactoLogConfig),
        filter_non_main_threads: true,
        extra_filter_env: extra_filter_env.unwrap_or(""),
    }
}

struct FactoLogConfig;
impl MapHugeCrateName for FactoLogConfig {
    fn map_huge(&self, input: &str) -> Option<&'static str> {
        match input {
            "facto_loop_miner" => Some("core"),
            "facto_loop_miner_io" => Some("io"),
            "facto_loop_miner_fac_engine" => Some("engine"),
            _ => None,
        }
    }
}
