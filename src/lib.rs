pub mod chip8;

use flexi_logger::{LogSpecBuilder, Logger};
use log::LevelFilter;

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
    emulator: EmulatorKind
}

struct State {
    emulator: Box<dyn Emulator>
}

// -------------- //
// Initialisation //
// -------------- //

impl Conf {
    pub fn new(emulator: EmulatorKind, rom_path: String, debug_view: bool) -> Conf {
        Conf { debug_view, rom_path, emulator }
    }

    fn init_logger(&self) -> Result<(), Box<dyn Error>> {
        let mut builder = LogSpecBuilder::new();
        builder.default(LevelFilter::Debug);

        Logger::with(builder.build()).start()?;
        Ok(())
    }

    fn init_emulator(&self) -> Result<Box<dyn Emulator>, Box<dyn Error>> {
        let rom = fs::read(&self.rom_path)?;

        let mut emulator = match self.emulator {
            EmulatorKind::Chip8 =>
                Box::new(chip8::Chip8::new())
        };

        emulator.load_rom(rom);
        Ok(emulator)
    }

    fn to_state(self) -> Result<State, Box<dyn Error>> {
        self.init_logger()?;
        let emulator = self.init_emulator()?;
        Ok(State { emulator })
    }
}

// ------------------- //
// Program Entry Point //
// ------------------- //

pub fn run(conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut state = conf.to_state()?;
    loop { state.emulator.cycle() }
}
