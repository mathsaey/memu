pub trait Emulator {
    type Cell;

    fn new() -> Self;
    fn load_rom(&mut self, rom: Vec<Self::Cell>);
    fn cycle(&mut self);
}
