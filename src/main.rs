use memu::chip8::Chip8;
use memu::generic::emulator::Emulator;

use std::fs;

fn main() {
    let rom = fs::read("roms/test.ch8").unwrap();
    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);

    for _ in 1..30 {
        chip8.cycle();
    }
}
