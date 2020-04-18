use std::error::Error;
use std::io;

use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::*;
use tui::style::*;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

type Backend = CrosstermBackend<io::Stdout>;
type Terminal = tui::Terminal<Backend>;

// Types for types that implement View
pub type Frame<'a> = tui::Frame<'a, Backend>;
pub type Rect = tui::layout::Rect;

// Allow various class to the view
pub trait View {
    fn draw(&self, frame: &mut Frame, area: Rect);
}

pub struct DebugView(Option<Inner>);

struct Inner {
    terminal: Terminal,
    log_wrapper: LogWrapper
}

impl DebugView {
    pub fn new(enable: bool) -> Result<DebugView, Box<dyn Error>> {
        if enable {
            let inner = Inner::new()?;
            Ok(DebugView(Some(inner)))
        } else {
            Ok(DebugView(None))
        }
    }

    pub fn log_writer(&self) -> Option<Box<LogWrapper>> {
        self.0.as_ref().map(|inner| inner.logbox())
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(inner) = self.0.as_mut() {
            inner.draw()?;
        }
        Ok(())
    }
}

impl Inner {
    #[inline]
    fn new() -> Result<Inner, Box<dyn Error>> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        let log_wrapper = LogWrapper::new();

        terminal.clear()?;
        Ok(Inner{terminal, log_wrapper})
    }

    #[inline]
    fn logbox(&self) -> Box<LogWrapper> {
        Box::new(self.log_wrapper.clone())
    }

    #[inline]
    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let buffer = self.log_wrapper.clone();

        self.terminal.draw(|mut frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(LOG_CAP as u16 + 2)].as_ref())
                .split(frame.size());

            buffer.draw(&mut frame, chunks[1]);
        })?;
        Ok(())
    }
}

// --------------- //
// Logging Support //
// --------------- //

use flexi_logger::writers::LogWriter;
use flexi_logger::{DeferredNow, Record, FormatFunction};

const LOG_CAP: usize = 5;

type LogBuffer = Arc<Mutex<VecDeque<String>>>;

#[derive(Clone)]
pub struct LogWrapper{
    buffer: LogBuffer,
    format: FormatFunction
}

impl LogWrapper {
    fn new() -> LogWrapper {
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_CAP)));
        let format = flexi_logger::colored_default_format;
        LogWrapper{buffer, format}
    }
}

impl LogWriter for LogWrapper {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        let mut writer: Vec<u8> = Vec::new();
        let format = self.format;

        format(&mut writer, now, record).unwrap();
        let string = String::from_utf8(writer).unwrap();

        if buffer.len() == LOG_CAP {
            buffer.pop_back();
        }
        buffer.push_front(string);

        Ok(())
    }

    fn format(&mut self, format: FormatFunction) {
        self.format = format;
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::max()
    }
}

impl View for LogWrapper {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let buffer = self.buffer.lock().unwrap();
        let items = buffer.iter().map(|i| Text::raw(i));
        let list = List::new(items)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }
}
