use flexi_logger::{style, DeferredNow, LogSpecBuilder, Logger};
use log::{LevelFilter, Record};
use std::error::Error;

use super::debug_view::DebugView;

pub fn setup(debug_view: &DebugView) -> Result<(), Box<dyn Error>> {
    let mut builder = LogSpecBuilder::new();
    builder
        .default(LevelFilter::Error)
        .module("memu", LevelFilter::Trace);

    let mut logger = Logger::with(builder.build());
    logger = logger
        .format_for_stderr(padded_colored_format)
        .format_for_writer(padded_colored_format);

    // Redirect log output to the debug view if it's enabled.
    // logger = match debug_view {
    //     Some(view) => logger
    //             .log_target(LogTarget::FileAndWriter(
    //                 cursive_flexi_logger_view::cursive_flexi_logger(&view)
    //             )),
    //     None =>
    //         logger
    // };

    logger.start()?;
    Ok(())
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
