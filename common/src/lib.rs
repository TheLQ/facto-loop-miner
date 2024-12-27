use std::env;

use num_format::Locale;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

pub const LOCALE: Locale = Locale::en;

const TRACE_NO_ADMIRAL_NETWORK: &str =
    "trace,facto_loop_miner_fac_engine::admiral::executor::client=debug";

pub fn log_init_trace() {
    log_init_internal(TRACE_NO_ADMIRAL_NETWORK);
}

pub fn log_init_debug() {
    log_init_internal("debug");
}

fn log_init_internal(default_env: &str) {
    let subscriber = Registry::default();

    let env_var = env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| default_env.into());
    let env_layer = EnvFilter::builder().parse(env_var).expect("bad env");
    let subscriber = subscriber.with(env_layer);

    let filter_layer = Layer::default().compact();
    let subscriber = subscriber.with(filter_layer);

    subscriber.init()
}
