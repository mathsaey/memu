use log::*;
use std::fmt;

use super::opcode::{OpCode, Operands};
use super::Chip8;

pub type InsFn = fn(&mut Chip8, Operands);
pub type InsName = &'static str;

pub struct Instruction {
    pub code: OpCode,
    pub name: InsName,
    pub operands: Operands,
    pub instruction: InsFn,
}

impl Instruction {
    pub fn create(c: OpCode, n: InsName, o: Operands, i: InsFn) -> Instruction {
        Instruction {
            code: c,
            name: n,
            operands: o,
            instruction: i,
        }
    }

    pub fn exec(self, chip8: &mut Chip8) {
        trace!("Execute `{}`", self);
        let inst = self.instruction;
        inst(chip8, self.operands)
    }
}

// ------------ //
// Instructions //
// ------------ //

pub fn not_implemented(_: &mut Chip8, _: Operands) {
    warn!("Ignoring unimplemented opcode");
}

pub fn jp_1nnn(e: &mut Chip8, o: Operands) {
    if let Operands::Address(a) = o {
        e.reg_pc = a;
    }
}

pub fn ld_6xkk(e: &mut Chip8, o: Operands) {
    if let Operands::RegAndConst(r, cns) = o {
        e.regs[r as usize] = cns;
    }
}

// ------------------ //
// Formatting Support //
// ------------------ //

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:<4}\t{:}", self.name, self.operands)
    }
}
