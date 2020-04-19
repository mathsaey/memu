use memu::{Conf, EmulatorKind};

fn main() {
    let rom_path = "roms/test.ch8".to_string();
    let conf = Conf::new(EmulatorKind::Chip8, rom_path, true);
    memu::run(conf).unwrap_or_else(|e| panic!("Emulator failed with error: `{}`", e));
}
