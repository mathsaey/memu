use std::error::Error;

use flexi_logger::writers::LogWriter;
use flexi_logger::{DeferredNow, FormatFunction, Level, Record};

use crate::Conf;
use crate::logger;
use crate::Emulator;
use crate::MissingFeatureError;

pub struct LogWrapper();
pub struct DebugView();
pub type Frame = ();
pub type Rect = ();

pub trait Debug {
    fn debug_view(&self, _frame: &mut Frame, _area: Rect) {}
}

impl DebugView {
    pub fn new(conf: &Conf) -> Result<DebugView, Box<dyn Error>> {
        if conf.debug_view {
            Err(Box::new(MissingFeatureError(String::from("debug-view"))))
        } else {
            Ok(DebugView())
        }
    }

    pub fn log_writer(&self) -> Option<Box<LogWrapper>> {
        None
    }

    pub fn log_handle(&mut self, handle: flexi_logger::ReconfigurationHandle) {
    }

    pub fn draw(&mut self, emulator: &Box<dyn Emulator>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl LogWriter for LogWrapper {
    fn write(&self, _now: &mut DeferredNow, _record: &Record) -> std::io::Result<()> {
        Ok(())
    }

    fn format(&mut self, _format: FormatFunction) {
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::max()
    }
}
