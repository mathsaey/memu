// Emulator-agnostic modules
mod debug_view;
mod logger;

// Emulators
#[cfg(feature = "chip8")]
mod chip8;

use ggez::{*, conf::*};
use log::*;

use structopt::clap::arg_enum;
use structopt::StructOpt;

use std::error::Error;
use std::fmt;
use std::fs;

use debug_view::{Debug, DebugView};

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

pub trait Emulator: Debug {
    /// Load a rom into the memory of the emulator
    fn load_rom(&mut self, rom: Vec<u8>);

    /// Advance the emulator by the amount of cycles that should have occured in the elapsed time
    fn advance(&mut self, elapsed: std::time::Duration) -> bool;

    /// Advance by one cpu cycle
    fn cycle(&mut self) -> bool;

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

// ------------------- //
// Game Loop and State //
// ------------------- //

struct State {
    emulator: Box<dyn Emulator>,
    debug_view: DebugView,

    step_mode: bool,

    // Drawing
    should_draw: bool,
}

impl State {
    fn new(emulator: Box<dyn Emulator>, debug_view: DebugView) -> State {
        State {
            emulator,
            debug_view,
            should_draw: true,
            step_mode: false,
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
        let fct  = height_fct.min(width_fct);

        let window_mode = WindowMode::default()
            .dimensions(emu_width * fct, emu_height * fct)
            .min_dimensions(emu_width, emu_height)
            .resizable(true);

        graphics::set_mode(ctx, window_mode).unwrap();
        graphics::set_screen_coordinates(ctx, [0.0, 0.0, emu_width, emu_height].into()).unwrap();

        self.force_draw();
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let should_draw = self.emulator.advance(timer::delta(ctx));
        self.maybe_draw(should_draw);

        self.debug_view.draw(&self.emulator).unwrap();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.should_draw {
            self.clear_draw();

            // Clear the screen and allow the emulator to draw
            graphics::clear(ctx, graphics::BLACK);
            self.emulator.draw(ctx)?;
            graphics::present(ctx)?;
        }

        // Find a better way to do this, currently vsync doesn't work on osx
        // At the very least, make it reason about frame time
        // timer::yield_now();
        timer::sleep(std::time::Duration::from_millis(20));

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, _width: f32, _height: f32) {
        self.set_window_size(ctx);
    }
}

pub fn run(conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut debug_view = DebugView::new(&conf)?;
    logger::setup(&conf, &mut debug_view)?;

    let emulator = init_emulator(&conf)?;
    debug_view.draw(&emulator)?;

    let mut state = State::new(emulator, debug_view);

    let window_setup = conf::WindowSetup::default()
        .title(format!("memu ({}) - {}", conf.emulator, conf.rom_path).as_str())
        .vsync(true);

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("memu", "Mathijs Saey")
        .window_setup(window_setup)
        .build()?;

    state.set_window_size(ctx);

    info!("Starting game loop");
    event::run(ctx, event_loop, &mut state).unwrap();
    Ok(())
}
