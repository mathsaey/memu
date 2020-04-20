use flexi_logger::{style, DeferredNow, LogSpecBuilder, LogTarget, Logger, ReconfigurationHandle};
use log::{LevelFilter, Record};
use std::error::Error;

use super::debug_view::DebugView;
use super::Conf;

pub fn setup(conf: &Conf, debug_view: &mut DebugView) -> Result<(), Box<dyn Error>> {
    let mut builder = LogSpecBuilder::new();
    builder
        .default(LevelFilter::Warn)
        .module("memu", conf.log_level);

    let mut logger = Logger::with(builder.build());
    logger = logger
        .format_for_stderr(padded_colored_format)
        .format_for_writer(padded_plain_format);

    // Redirect log output to the debug view if it's enabled.
    logger = match debug_view.log_writer() {
        Some(writer) => logger.log_target(LogTarget::Writer(writer)),
        None => logger,
    };

    let handle = logger.start()?;
    debug_view.log_handle(handle);
    Ok(())
}

pub fn disable(handle: &mut ReconfigurationHandle) {
    let mut builder = LogSpecBuilder::new();
    builder.module("memu", LevelFilter::Error);
    handle.push_temp_spec(builder.build());
}

pub fn enable(handle: &mut ReconfigurationHandle) {
    handle.pop_temp_spec();
}

fn padded_colored_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{:<5} [{:<25}] {}",
        style(level, level),
        record.module_path().unwrap_or("<unnamed>"),
        style(level, record.args())
    )
}

fn padded_plain_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{:<5} [{:<25}] {}",
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}
