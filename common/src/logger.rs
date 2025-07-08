use chrono::Local;
use nu_ansi_term::{Color, Style};
use std::env;
use tracing::{Event, Level, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

const TRACE_NO_ADMIRAL_NETWORK: &str = "trace,\
facto_loop_miner_fac_engine::admiral::executor::client=debug,\
facto_loop_miner_fac_engine::admiral::lua_command::lua_batch=debug,\
facto_loop_miner_fac_engine::game_blocks::rail_hope_single=debug";

pub fn log_init_trace() {
    log_init_internal(TRACE_NO_ADMIRAL_NETWORK);
    // log_init_internal("trace");
}

pub fn log_init_debug() {
    log_init_internal("debug");
}

fn log_init_internal(default_env: &str) {
    let env_var = env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| default_env.into());
    let env_layer = EnvFilter::builder().parse(env_var).expect("bad env");

    // let print_layer = tracing_subscriber::fmt::Layer::default().compact();
    let print_layer = tracing_subscriber::fmt::Layer::default().event_format(LoopFormatter);

    Registry::default().with(env_layer).with(print_layer).init()
}

/// kustom Formatter because the compact formatter is not configurably compact enough
/// Sadly needs a lot re-implemented
/// https://github.com/tokio-rs/tracing/blob/e63ef57f3d686abe3727ddd586eb9af73d6715b7/tracing-subscriber/src/fmt/format/pretty.rs#L175
struct LoopFormatter;

impl<S, N> FormatEvent<S, N> for LoopFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut f: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        // Time without gigantic date, when we only run for at max hours
        let time = Local::now();
        write!(f, "{}", time.time().format("%H:%M:%S%.6f"))?;

        // Copied from tracing-subscriber
        let level = match *event.metadata().level() {
            Level::TRACE => Color::Purple.paint("TRACE"),
            Level::DEBUG => Color::Blue.paint("DEBUG"),
            Level::INFO => Color::Green.paint("INFO "),
            Level::WARN => Color::Yellow.paint("WARN "),
            Level::ERROR => Color::Red.paint("ERROR"),
        };
        write!(f, "{level} ")?;

        // Mostly copied from tracing-subscriber
        let current_thread = std::thread::current();
        match current_thread.name() {
            Some("main") => {
                // this is noise normally
            }
            Some(name) => {
                write!(f, "[{}] ", name)?;
            }
            None => {
                write!(f, "[u{:0>2?}] ", current_thread.id())?;
            }
        }

        // execution threads, add offset text
        let span = event
            .parent()
            .and_then(|id| ctx.span(id))
            .or_else(|| ctx.lookup_current());
        let scope = span.into_iter().flat_map(|span| span.scope());
        for span in scope {
            if span.metadata().name() == EXECUTOR_TAG && current_thread.name() == Some("main") {
                write!(f, "[exe] ")?;
            }
        }

        // Compress gigantic crate names
        let dimmed = Style::new().dimmed();
        let target_raw = meta.target();
        let target = match target_raw.split_once(":").unwrap_or((target_raw, "")) {
            ("facto_loop_miner", path) => &format!("core{path}"),
            ("facto_loop_miner_io", path) => &format!("io{path}"),
            ("facto_loop_miner_fac_engine", path) => &format!("engine{path}"),
            _ => target_raw,
        };
        write!(f, "{}{} ", dimmed.paint(target), dimmed.paint(":"))?;

        // Hope this does spans
        ctx.format_fields(f.by_ref(), event)?;

        // newline
        writeln!(f)
    }
}

pub const EXECUTOR_TAG: &str = "executor";
