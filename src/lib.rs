pub mod chip8;

use flexi_logger::{LogSpecBuilder, Logger, DeferredNow, style};
use log::{LevelFilter, Record};
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
    emulator: EmulatorKind
}

struct State {
    emulator: Box<dyn Emulator>
}

// ------------- //
// Logger Format //
// ------------- //

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
// -------------- //
// Initialisation //
// -------------- //

impl Conf {
    pub fn new(emulator: EmulatorKind, rom_path: String, debug_view: bool) -> Conf {
        Conf { debug_view, rom_path, emulator }
    }

    fn init_logger(&self) -> Result<(), Box<dyn Error>> {
        let mut builder = LogSpecBuilder::new();
        builder.default(LevelFilter::Trace);

        Logger::with(builder.build())
            .format_for_stderr(padded_colored_format)
            .start()?;
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
