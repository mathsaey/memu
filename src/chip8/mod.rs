mod opcode;
mod instruction;

// TODO:
//  - move stack to separate module, push & pop

use crate::generic::emulator::Emulator;
use opcode::OpCode;

const STACK_SIZE: usize = 16;
const GP_AMOUNT:  usize = 16;
const MEM_SIZE:   usize = 4 * 1024;

pub struct Chip8 {
    // Main Memory ------------------------------------------------------------
    mem:   [u8; MEM_SIZE],
    // Stack ------------------------------------------------------------------
    stack: [u16; STACK_SIZE], // Stack to store program counter
    // Registers --------------------------------------------------------------
    regs:  [u8; GP_AMOUNT],   // General purpose, V0 to VF
    reg_i:  u16,              // Address register
    reg_pc: u16,              // Program counter (pseudo)
    // ------------------------------------------------------------------------
}

impl Emulator for Chip8 {
    type Cell = u8;

    fn new() -> Chip8 {
        Chip8 {
            mem:    [0x00; MEM_SIZE],
            stack:  [0x00; STACK_SIZE],
            regs:   [0x00; GP_AMOUNT],
            reg_i:  0x000,
            reg_pc: 0x200, // Programs start at 0x200
        }
    }

    fn load_rom(&mut self, content: Vec<Self::Cell>) {
        for (ctr, el) in content.into_iter().enumerate() {
            self.mem[self.reg_pc as usize + ctr] = el;
        }
    }

    fn cycle(&mut self) {
        let instruction = self.fetch().decode();
        println!("${:X} {}", self.reg_pc, instruction);

        // let mut input = String::new();
        // std::io::stdin().read_line(&mut input).unwrap();
        instruction.exec(self);
    }

}

impl Chip8 {
    fn fetch(&mut self) -> OpCode {
        let c1 = self.mem[self.reg_pc as usize];
        let c2 = self.mem[(self.reg_pc + 1) as usize];
        self.reg_pc += 2;

        OpCode::from_cells(c1, c2)
    }

    pub fn print_mem(&self) {
        let base = self.reg_pc as usize;
        println!("Memory contents: {:?}", &self.mem[base..base + 32]);
    }
}
