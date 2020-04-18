// Emulator-agnostic modules
mod debug_view;
mod logger;

// Emulators
mod chip8;

use debug_view::DebugView;

use log::*;
use std::error::Error;
use std::fs;

pub enum EmulatorKind {
    Chip8,
}

pub trait Emulator {
    fn load_rom(&mut self, rom: Vec<u8>);
    fn cycle(&mut self);
}

pub struct Conf {
    debug_view: bool,
    rom_path: String,
    emulator: EmulatorKind,
}

struct State {
    emulator: Box<dyn Emulator>,
    debug_view: DebugView,
}

// -------------- //
// Initialisation //
// -------------- //

impl Conf {
    pub fn new(emulator: EmulatorKind, rom_path: String, debug_view: bool) -> Conf {
        Conf {
            debug_view,
            rom_path,
            emulator,
        }
    }

    fn init_emulator(&self) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
        let mut emulator = match self.emulator {
            EmulatorKind::Chip8 => Box::new(chip8::Chip8::new()),
        };

        info!("Loading rom: `{}`", self.rom_path);
        let rom = fs::read(&self.rom_path)?;
        emulator.load_rom(rom);

        Ok(emulator)
    }

    fn to_state(self) -> Result<State, Box<dyn Error>> {
        let debug_view = DebugView::new(self.debug_view)?;
        logger::setup(&debug_view)?;

        let emulator = self.init_emulator()?;
        Ok(State {
            emulator,
            debug_view,
        })
    }
}

// ------------------- //
// Program Entry Point //
// ------------------- //

pub fn run(conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut state = conf.to_state()?;

    for _ in 1..30 {
        state.emulator.cycle()
    }
    Ok(())
}