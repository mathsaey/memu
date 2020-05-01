// Emulator-agnostic modules
mod debug_view;
mod display;
mod logger;

// Emulators
#[cfg(feature = "chip8")]
mod chip8;

use log::*;

use structopt::clap::arg_enum;
use structopt::StructOpt;

use std::error::Error;
use std::fmt;
use std::fs;

use debug_view::{DebugView, Frame, Rect};
use display::Display;

// ------------- //
// Configuration //
// ------------- //

#[derive(StructOpt)]
#[structopt(name = "memu")]
pub struct Conf {
    /// Show the current state of the emulator in the console
    #[structopt(short = "D", long)]
    debug_view: bool,
    #[structopt(
        short, long, case_insensitive = true, hide_default_value = true,
        possible_values= &["trace", "debug", "info", "warn", "error", "off"],
        default_value="warn", default_value_if("debug-view", None, "trace"),
    )]
    /// The log level to use. Defaults to `trace` if `--debug_view` is set, or `warn` otherwise
    log_level: LevelFilter,
    #[structopt(possible_values = &EmulatorKind::variants(), case_insensitive = true)]
    /// Emulator to use
    emulator: EmulatorKind,
    /// Path to the rom to emulate
    rom_path: String,
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum EmulatorKind {
        Chip8,
    }
}

// ------ //
// Errors //
// ------ //

#[derive(Debug)]
pub struct MissingFeatureError(String);

impl fmt::Display for MissingFeatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memu was built without {} support", self.0)
    }
}

impl Error for MissingFeatureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

// -------------- //
// Emulator Trait //
// -------------- //

pub trait Emulator {
    fn clock_rate(&self) -> usize;

    fn load_rom(&mut self, rom: Vec<u8>);
    fn cycle(&mut self) -> bool;

    fn screen_dimensions(&self) -> (usize, usize);
    fn draw_screen(&self) -> Vec<u32>;

    #[cfg(feature = "debug-view")]
    fn draw_debug(&self, frame: &mut Frame, area: Rect) {
        let text = [tui::widgets::Text::raw("Debug view not implemented")];
        let par =
            tui::widgets::Paragraph::new(text.iter()).alignment(tui::layout::Alignment::Center);

        frame.render_widget(par, area);
    }

    #[cfg(not(feature = "debug-view"))]
    fn draw_debug(&self, _frame: &mut Frame, _area: Rect) {
    }
}

// -------------------- //
// Initialisation Logic //
// -------------------- //

#[cfg(feature = "chip8")]
fn init_chip8(_: EmulatorKind) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
    Ok(Box::new(chip8::Chip8::new()))
}
#[cfg(not(feature = "chip8"))]
fn init_chip8(kind: EmulatorKind) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
    Err(Box::new(MissingFeatureError(kind.to_string())))
}

fn init_emulator(conf: &Conf) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
    let mut emulator = match conf.emulator {
        EmulatorKind::Chip8 => init_chip8(conf.emulator)?,
    };

    info!("Loading rom: `{}`", &conf.rom_path);
    let rom = fs::read(&conf.rom_path)?;
    emulator.load_rom(rom);

    Ok(emulator)
}

// ---- //
// Main //
// ---- //

use crossterm::event::{Event, KeyCode, KeyEvent};

pub fn run(conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut debug_view = DebugView::new(&conf)?;
    logger::setup(&conf, &mut debug_view)?;

    let mut emulator = init_emulator(&conf)?;
    let mut display = Display::new(&conf, &emulator)?;

    display.update(&emulator)?;
    debug_view.draw(&emulator)?;

    loop {
        let event = debug_view.wait_for_key()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: _,
            }) => {
                if emulator.cycle() {
                    display.update(&emulator)?;
                }
                debug_view.draw(&emulator)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: _,
            }) => break,
            Event::Resize(_, _) => debug_view.draw(&emulator)?,
            _ => continue,
        }
    }

    Ok(())
}
