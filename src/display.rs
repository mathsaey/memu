use minifb::{Key, ScaleMode, Window, WindowOptions};

use std::error::Error;

use super::Conf;
use super::Emulator;

// TODO: Remove calls to unwrap once minifb is updated
// TODO: Find a better way to set scale

pub struct Display(Window);

pub type Scale = minifb::Scale;

impl Display {
    pub fn new(conf: &Conf, emulator: &Box<dyn Emulator>) -> Result<Display, Box<dyn Error>> {
        let (width, height) = emulator.screen_dimensions();
        let mut window = Window::new(
            format!("memu ({}) - {}", conf.emulator, conf.rom_path).as_str(),
            width,
            height,
            WindowOptions {
                scale: Scale::X16,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        window.limit_update_rate(None);
        Ok(Display(window))
    }

    pub fn update(&mut self, emulator: &Box<dyn Emulator>) -> Result<(), Box<dyn Error>> {
        let (width, height) = emulator.screen_dimensions();
        let _ = self
            .0
            .update_with_buffer(&emulator.draw_screen(), width, height)
            .unwrap();
        Ok(())
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }
}
