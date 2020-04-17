mod opcode;
mod instruction;

use log::*;
use opcode::OpCode;

// TODO:
//  - move stack to separate module, push & pop

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

impl crate::Emulator for Chip8 {
    fn load_rom(&mut self, content: Vec<u8>) {
        for (ctr, el) in content.into_iter().enumerate() {
            self.mem[self.reg_pc as usize + ctr] = el;
        }
    }

    fn cycle(&mut self) {
        let instruction = self.fetch().decode();
        debug!("PC: {:#X} Instruction: {}", self.reg_pc, instruction);

        // Temporary
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        instruction.exec(self);
    }

}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            mem:    [0x00; MEM_SIZE],
            stack:  [0x00; STACK_SIZE],
            regs:   [0x00; GP_AMOUNT],
            reg_i:  0x000,
            reg_pc: 0x200, // Programs start at 0x200
        }
    }

    fn fetch(&mut self) -> OpCode {
        let code = self.get_opcode(self.reg_pc);
        self.reg_pc += 2;
        code
    }

    fn get_opcode(&self, idx: u16) -> OpCode {
        OpCode::from_cells(
            self.mem[idx as usize],
            self.mem[(idx + 1) as usize]
        )
    }
}
