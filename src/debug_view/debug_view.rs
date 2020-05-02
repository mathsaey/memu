use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};

use std::io::{stdout, Stdout, Write};

use flexi_logger::ReconfigurationHandle;

use crossterm::{execute, terminal};

use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::*;
use tui::widgets::*;

use crate::Conf;
use crate::logger;
use crate::Emulator;

// ----- //
// Trait //
// ----- //

pub trait Debug {
    fn debug_view(&self, frame: &mut Frame, area: Rect) {
        let text = [tui::widgets::Text::raw("Debug view not implemented")];
        let par =
            tui::widgets::Paragraph::new(text.iter()).alignment(tui::layout::Alignment::Center);

        frame.render_widget(par, area);
    }
}

// ------- //
// Wrapper //
// ------- //

// Types for types that implement View
pub type Frame<'a> = tui::Frame<'a, Backend>;
pub type Rect = tui::layout::Rect;

pub struct DebugView(Option<Inner>);

impl DebugView {
    pub fn new(conf: &Conf) -> Result<DebugView, Box<dyn Error>> {
        if conf.debug_view {
            let inner = Inner::new()?;
            Ok(DebugView(Some(inner)))
        } else {
            Ok(DebugView(None))
        }
    }

    pub fn log_writer(&self) -> Option<Box<LogWrapper>> {
        self.0.as_ref().map(|inner| inner.logbox())
    }

    pub fn log_handle(&mut self, handle: ReconfigurationHandle) {
        if let Some(inner) = self.0.as_mut() {
            inner.log_handle = Some(handle);
        }
    }

    pub fn draw(&mut self, emulator: &Box<dyn Emulator>) -> Result<(), Box<dyn Error>> {
        if let Some(inner) = self.0.as_mut() {
            inner.draw(emulator)?;
        }
        Ok(())
    }
}

// ---------- //
// Debug View //
// ---------- //

type Backend = CrosstermBackend<Stdout>;
type Terminal = tui::Terminal<Backend>;

struct Inner {
    terminal: Terminal,
    log_wrapper: LogWrapper,
    log_handle: Option<ReconfigurationHandle>,
}

impl Inner {
    #[inline]
    fn new() -> Result<Inner, Box<dyn Error>> {
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        let log_wrapper = LogWrapper::new();

        execute!(stdout(), terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        terminal.clear()?;
        terminal.hide_cursor()?;

        Ok(Inner {
            terminal,
            log_wrapper,
            log_handle: None,
        })
    }

    #[inline]
    fn logbox(&self) -> Box<LogWrapper> {
        Box::new(self.log_wrapper.clone())
    }

    #[inline]
    fn draw(&mut self, emulator: &Box<dyn Emulator>) -> Result<(), Box<dyn Error>> {
        // Disable logging while we draw the view
        logger::disable(self.log_handle.as_mut().unwrap());

        let log_buffer = self.log_wrapper.buffer.clone();

        self.terminal.draw(|mut frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(LOG_CAP as u16 + 2)].as_ref())
                .split(frame.size());

            draw_log(log_buffer, &mut frame, chunks[1]);
            emulator.debug_view(&mut frame, chunks[0]);
        })?;

        logger::enable(self.log_handle.as_mut().unwrap());
        Ok(())
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(stdout(), terminal::LeaveAlternateScreen).unwrap();
    }
}

// --------------- //
// Logging Support //
// --------------- //

use flexi_logger::writers::LogWriter;
use flexi_logger::{DeferredNow, FormatFunction, Level, Record};

const LOG_CAP: usize = 8;

type LogBuffer = Arc<Mutex<VecDeque<(String, Style)>>>;

#[derive(Clone)]
pub struct LogWrapper {
    buffer: LogBuffer,
    format: FormatFunction,
}

impl LogWrapper {
    fn new() -> LogWrapper {
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_CAP)));
        let format = flexi_logger::colored_default_format;
        LogWrapper { buffer, format }
    }
}

impl LogWriter for LogWrapper {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        let mut writer: Vec<u8> = Vec::new();
        let format = self.format;

        format(&mut writer, now, record).unwrap();
        let string = String::from_utf8(writer).unwrap();

        let col = match record.level() {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::White,
            Level::Debug => Color::Gray,
            Level::Trace => Color::DarkGray,
        };
        let style = Style::new().fg(col);

        if buffer.len() == LOG_CAP {
            buffer.pop_back();
        }
        buffer.push_front((string, style));

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

fn draw_log(buffer: LogBuffer, frame: &mut Frame, area: Rect) {
    let buffer = buffer.lock().unwrap();
    let items = buffer.iter().map(|t| Text::styled(t.0.clone(), t.1));
    let list = List::new(items)
        .block(Block::default().title("Log").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}
