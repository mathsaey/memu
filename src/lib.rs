// Emulator-agnostic modules
mod debug_view;
mod logger;

// Emulators
mod chip8;

use log::*;

use structopt::clap::arg_enum;
use structopt::StructOpt;

use std::error::Error;
use std::fs;

use debug_view::{DebugView, Frame, Rect};

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
    pub enum EmulatorKind {
        Chip8,
    }
}

// -------------- //
// Emulator Trait //
// -------------- //

pub trait Emulator {
    fn clock_rate(&self) -> usize;
    fn screen_dimensions(&self) -> (usize, usize);

    fn load_rom(&mut self, rom: Vec<u8>);
    fn cycle(&mut self) -> bool;

    fn screen_buffer(&self) -> &[u32];

    fn draw_debug(&self, frame: &mut Frame, area: Rect) {
        let text = [tui::widgets::Text::raw("Debug view not implemented")];
        let par =
            tui::widgets::Paragraph::new(text.iter()).alignment(tui::layout::Alignment::Center);

        frame.render_widget(par, area);
    }
}

// -------------------- //
// Initialisation Logic //
// -------------------- //

fn init_emulator(conf: &Conf) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
    let mut emulator = match conf.emulator {
        EmulatorKind::Chip8 => Box::new(crate::chip8::Chip8::new()),
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

    debug_view.draw(&emulator)?;

    loop {
        let event = debug_view.wait_for_key()?;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: _,
            }) => {
                emulator.cycle();
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
