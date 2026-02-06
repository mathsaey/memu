// Emulator-agnostic modules
mod debug_view;
mod logger;
mod utils;

// Emulators
#[cfg(feature = "chip8")]
mod chip8;

use ggez::{conf::*, input::keyboard::*, *};
use log::*;

use clap::{Parser, ValueEnum};

use std::error::Error;
use std::fmt;
use std::fs;

use debug_view::{Debug, DebugView};

// ------------- //
// Configuration //
// ------------- //

#[derive(Parser)]
#[structopt(name = "memu")]
pub struct Conf {
    /// Show the current state of the emulator in the console
    #[arg(short = 'D', long)]
    debug_view: bool,
    #[arg(
        short, long, hide_default_value = true, value_enum,
        default_value="warn", default_value_if("debug-view", "true", "trace"),
    )]
    /// The log level to use. Defaults to `trace` if `--debug_view` is set, or `warn` otherwise
    log_level: LevelFilter,
    #[arg(value_enum)]
    /// Emulator to use
    emulator: EmulatorKind,
    /// Path to the rom to emulate
    rom_path: String,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum EmulatorKind {
    Chip8,
}

impl fmt::Display for EmulatorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EmulatorKind::Chip8 => "Chip 8",
        };
        write!(fmt, "{}", name)
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

pub trait Emulator: Debug {
    /// Load a rom into the memory of the emulator
    fn load_rom(&mut self, rom: Vec<u8>);

    /// Advance the emulator by the amount of cycles that should have occured in the elapsed time
    fn advance(&mut self, elapsed: std::time::Duration) -> bool;

    /// Amount of time that needs to pass for a single cycle
    fn cycle_dt(&self) -> std::time::Duration;

    /// Handle a down event
    fn key_down(&mut self, key: KeyCode);

    /// Handle a key_up event
    fn key_up(&mut self, key: KeyCode);

    /// Size of the drawn area
    fn draw_size(&self) -> (f32, f32);

    /// Draw the emulator state to the screen
    fn draw(&self, ctx: &mut Context) -> GameResult<()>;
}

// ----------------------------- //
// Emulator Initialisation Logic //
// ----------------------------- //

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

// ---------- //
// Game State //
// ---------- //

#[derive(Clone, Copy)]
enum ProgressMode {
    Normal,
    Cycle(bool),
    Frame(bool),
}

impl ProgressMode {
    fn next(self) -> ProgressMode {
        match self {
            ProgressMode::Normal => ProgressMode::Cycle(false),
            ProgressMode::Cycle(_) => ProgressMode::Frame(false),
            ProgressMode::Frame(_) => ProgressMode::Normal,
        }
    }
}

impl fmt::Display for ProgressMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressMode::Normal => f.pad("normal"),
            ProgressMode::Cycle(_) => f.pad("cycle"),
            ProgressMode::Frame(_) => f.pad("frame"),
        }
    }
}

struct State {
    emulator: Box<dyn Emulator>,
    debug_view: DebugView,

    // Emulation mode / speed
    progress_mode: ProgressMode,
    speed_factor: f32,

    // Drawing
    should_draw: bool,
}

impl State {
    fn new(conf: &Conf, emulator: Box<dyn Emulator>, debug_view: DebugView) -> State {
        let progress_mode = if conf.debug_view {
            ProgressMode::Cycle(false)
        } else {
            ProgressMode::Normal
        };

        State {
            emulator,
            debug_view,
            progress_mode,
            speed_factor: 1.0,
            should_draw: true,
        }
    }

    fn clear_draw(&mut self) {
        self.should_draw = false;
    }

    fn force_draw(&mut self) {
        self.should_draw = true;
    }

    fn maybe_draw(&mut self, should_draw: bool) {
        self.should_draw = self.should_draw || should_draw;
    }

    fn set_window_size(&mut self, ctx: &mut Context) {
        let (win_width, win_height) = graphics::drawable_size(ctx);
        let (emu_width, emu_height) = self.emulator.draw_size();
        let height_fct = win_height / emu_height;
        let width_fct = win_width / emu_width;
        let fct = height_fct.min(width_fct);

        let window_mode = WindowMode::default()
            .dimensions(emu_width * fct, emu_height * fct)
            .min_dimensions(emu_width, emu_height)
            .resizable(true);

        graphics::set_mode(ctx, window_mode).unwrap();
        graphics::set_screen_coordinates(ctx, [0.0, 0.0, emu_width, emu_height].into()).unwrap();

        self.force_draw();
    }

    fn change_progress_mode(&mut self, _ctx: &mut Context) {
        self.progress_mode = self.progress_mode.next();
        info!("Emulation changed to {:6} mode.", self.progress_mode);
    }

    fn inc_speed(&mut self, _ctx: &mut Context) {
        self.speed_factor += 0.1;
    }
    fn dec_speed(&mut self, _ctx: &mut Context) {
        self.speed_factor -= 0.1;
    }

    fn set_progress(&mut self) {
        self.progress_mode = match self.progress_mode {
            ProgressMode::Cycle(_) => ProgressMode::Cycle(true),
            ProgressMode::Frame(_) => ProgressMode::Frame(true),
            mode => mode,
        };
    }

    fn clear_progress(&mut self) {
        self.progress_mode = match self.progress_mode {
            ProgressMode::Cycle(_) => ProgressMode::Cycle(false),
            ProgressMode::Frame(_) => ProgressMode::Frame(false),
            mode => mode,
        };
    }

    fn frame_mode_cycle(&mut self) -> bool {
        const MAX_CYCLES: u32 = 10000;

        let dt = self.emulator.cycle_dt();
        let mut frame = false;
        let mut ctr = 0;

        while !frame && ctr < MAX_CYCLES {
            ctr += 1;
            frame = self.emulator.advance(dt);
        }

        if !frame {
            error!(
                "Possible infinite loop: more than {} cycles without a frame update",
                MAX_CYCLES
            );
        }

        frame
    }
}

// --------- //
// Game Loop //
// --------- //

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let emu_requires_draw = match self.progress_mode {
            ProgressMode::Normal => self
                .emulator
                .advance(timer::delta(ctx).mul_f32(self.speed_factor)),
            ProgressMode::Cycle(true) => self.emulator.advance(self.emulator.cycle_dt()),
            ProgressMode::Frame(true) => self.frame_mode_cycle(),
            _ => false,
        };

        self.clear_progress();
        self.maybe_draw(emu_requires_draw);
        self.debug_view.draw(&self.emulator).unwrap();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.should_draw {
            self.clear_draw();

            graphics::clear(ctx, graphics::BLACK);
            self.emulator.draw(ctx)?;
            graphics::present(ctx)?;
        }

        timer::yield_now();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        self.set_window_size(ctx);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, code: KeyCode, _mods: KeyMods) {
        self.emulator.key_up(code);
    }

    fn key_down_event(&mut self, ctx: &mut Context, code: KeyCode, _mods: KeyMods, _: bool) {
        match code {
            KeyCode::Escape => event::quit(ctx),
            // Speed / Cycle control
            KeyCode::Slash => self.change_progress_mode(ctx),
            KeyCode::Period => self.inc_speed(ctx),
            KeyCode::Comma => self.dec_speed(ctx),
            KeyCode::Space => match self.progress_mode {
                ProgressMode::Cycle(false) => self.set_progress(),
                ProgressMode::Frame(false) => self.set_progress(),
                _ => (),
            },
            key => self.emulator.key_down(key)
        }
    }
}

// ---------------------- //
// Program Initialisation //
// ---------------------- //

pub fn run(conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut debug_view = DebugView::new(&conf)?;
    logger::setup(&conf, &mut debug_view)?;

    let emulator = init_emulator(&conf)?;
    debug_view.draw(&emulator)?;

    let mut state = State::new(&conf, emulator, debug_view);

    let window_setup = conf::WindowSetup::default()
        .title(format!("memu ({}) - {}", conf.emulator, conf.rom_path).as_str())
        .vsync(true);

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("memu", "Mathijs Saey")
        .window_setup(window_setup)
        .build()?;

    state.set_window_size(ctx);

    info!("Starting emulation loop in {} mode", state.progress_mode);
    event::run(ctx, event_loop, &mut state)?;
    info!("Emulation loop finished, shutting down");
    Ok(())
}
