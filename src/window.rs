use minifb::{Key, WindowOptions, ScaleMode};

use std::error::Error;

use super::Emulator;
use super::Conf;

// TODO: Remove calls to unwrap once minifb is updated
// TODO: Find a better way to set scale

pub struct Window{
    window: minifb::Window,
}

pub type Scale = minifb::Scale;

impl Window {
    pub fn new(_conf: &Conf, emulator: &Box<dyn Emulator>) -> Result<Window, Box<dyn Error>> {
        let (width, height) = emulator.screen_dimensions();
        let window = minifb::Window::new(
            "memu",
            width,
            height,
            WindowOptions{
                scale: Scale::X16,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            }
        ).unwrap();
        Ok(Window{window})
    }

    pub fn update(&mut self, emulator: &Box<dyn Emulator>) -> Result<(), Box<dyn Error>> {
        let (width, height) = emulator.screen_dimensions();
        let _ = self.window.update_with_buffer(emulator.screen_buffer(), width, height).unwrap();
        Ok(())
    }
}
