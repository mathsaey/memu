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
    pub fn create(code: OpCode, n: InsName, o: Operands, i: InsFn) -> Instruction {
        Instruction {
            code,
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

// Register Operations
// -------------------

pub fn ld_6xkk(e: &mut Chip8, o: Operands) {
    if let Operands::RegAndConst(r, cns) = o {
        e.regs[r as usize] = cns;
    }
}

pub fn ld_8xy0(e: &mut Chip8, o: Operands) {
    if let Operands::Regs(x, y) = o {
        e.regs[x as usize] = e.regs[y as usize];
    }
}

// Arithmetic
pub fn add_7xkk(e: &mut Chip8, o: Operands) {
    if let Operands::RegAndConst(r, k) = o {
        let res = (e.regs[r as usize] as u16) + (k as u16);
        // Flag is not set if overflow occurs
        e.regs[r as usize] = (res & 0x00FF) as u8;
    }
}

pub fn add_8xy4(e: &mut Chip8, o: Operands) {
    if let Operands::Regs(x, y) = o {
        let res = (e.regs[x as usize] as u16) + (e.regs[y as usize] as u16);
        if res > 255 {
            e.regs[0xF] = 1
        } else {
            e.regs[0xF] = 0
        }
        e.regs[x as usize] = (res & 0x00FF) as u8;
    }
}

pub fn jp_1nnn(e: &mut Chip8, o: Operands) {
    if let Operands::Address(a) = o {
        e.reg_pc = a;
    }
}

pub fn ld_annn(e: &mut Chip8, o: Operands) {
    if let Operands::Address(a) = o {
        e.reg_i = a;
    }
}

pub fn ld_fx55(e: &mut Chip8, o: Operands) {
    if let Operands::Reg(r) = o {
        for ctr in 0..(r + 1) {
            e.mem[e.reg_i as usize] = e.regs[ctr as usize];
            e.reg_i += 1;
        }
    }
}

pub fn ld_fx65(e: &mut Chip8, o: Operands) {
    if let Operands::Reg(r) = o {
        for ctr in 0..(r + 1) {
            e.regs[ctr as usize] = e.mem[e.reg_i as usize];
            e.reg_i += 1;
        }
    }
}

// ------------------ //
// Formatting Support //
// ------------------ //

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:04X}) {:<4}\t{:}",
            self.code, self.name, self.operands
        )
    }
}
