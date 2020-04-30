use log::*;
use std::fmt;

use super::instruction;
use super::instruction::Instruction;

#[derive(Clone, Copy, Debug)]
pub struct OpCode(u16);

#[derive(Debug)]
pub enum Operands {
    Empty,                    // No operands
    Address(u16),             // 12 bit address (nnn)
    Reg(u8),                  // Register name
    Regs(u8, u8),             // Register names
    RegAndConst(u8, u8),      // Register name + 8 bit constant (xkk)
    RegsAndConst(u8, u8, u8), // Register names + 4 bit constant
}

impl OpCode {
    pub fn from_cells(m1: u8, m2: u8) -> OpCode {
        let m1 = m1 as u16;
        let m2 = m2 as u16;
        OpCode((m1 << 8) | m2)
    }

    fn to_matchtup(self) -> (u8, u8, u8, u8) {
        (
            ((self.0 & 0xF000) >> 12) as u8,
            ((self.0 & 0x0F00) >> 8) as u8,
            ((self.0 & 0x00F0) >> 4) as u8,
            ((self.0 & 0x000F) >> 0) as u8,
        )
    }

    pub fn decode(self) -> Instruction {
        let ins = match self.to_matchtup() {
            // 00E0
            (0, 0, 0xE, 0x0) => {
                Instruction::create(self, "CLS", Operands::Empty, instruction::not_implemented)
            }
            // 00EE
            (0, 0, 0xE, 0xE) => {
                Instruction::create(self, "RET", Operands::Empty, instruction::not_implemented)
            }
            //0nnn
            (0, _, _, _) => {
                Instruction::create(self, "SYS", decode_addr(self), instruction::not_implemented)
            }
            // 1nnn
            (1, _, _, _) => {
                Instruction::create(self, "JP", decode_addr(self), instruction::jp_1nnn)
            }
            // 2nnn
            (2, _, _, _) => Instruction::create(
                self,
                "CALL",
                decode_addr(self),
                instruction::not_implemented,
            ),
            // 3xkk
            (3, _, _, _) => Instruction::create(
                self,
                "SE",
                decode_reg_const(self),
                instruction::not_implemented,
            ),
            // 4xkk
            (4, _, _, _) => Instruction::create(
                self,
                "SNE",
                decode_reg_const(self),
                instruction::not_implemented,
            ),
            // 5xy0
            (5, _, _, 0) => {
                Instruction::create(self, "SE", decode_regs(self), instruction::not_implemented)
            }
            // 6xkk
            (6, _, _, _) => {
                Instruction::create(self, "LD", decode_reg_const(self), instruction::ld_6xkk)
            }
            // 7xkk
            (7, _, _, _) => Instruction::create(
                self,
                "ADD",
                decode_reg_const(self),
                instruction::add_7xkk,
            ),
            // 8xy0
            (8, _, _, 0) => {
                Instruction::create(self, "LD", decode_regs(self), instruction::ld_8xy0)
            }
            // 8xy1
            (8, _, _, 1) => {
                Instruction::create(self, "OR", decode_regs(self), instruction::or_8xy1)
            }
            // 8xy2
            (8, _, _, 2) => {
                Instruction::create(self, "AND", decode_regs(self), instruction::and_8xy2)
            }
            // 8xy3
            (8, _, _, 3) => {
                Instruction::create(self, "XOR", decode_regs(self), instruction::xor_8xy3)
            }
            // 8xy4
            (8, _, _, 4) => {
                Instruction::create(self, "ADD", decode_regs(self), instruction::add_8xy4)
            }
            // 8xy5
            (8, _, _, 5) => {
                Instruction::create(self, "SUB", decode_regs(self), instruction::not_implemented)
            }
            // 8xy6
            (8, _, _, 6) => {
                Instruction::create(self, "SHR", decode_regs(self), instruction::not_implemented)
            }
            // 8xy7
            (8, _, _, 7) => Instruction::create(
                self,
                "SUBN",
                decode_regs(self),
                instruction::not_implemented,
            ),
            // 8xyE
            (8, _, _, 0xE) => {
                Instruction::create(self, "SHL", decode_regs(self), instruction::not_implemented)
            }
            // 9xy0
            (9, _, _, 0) => {
                Instruction::create(self, "SNE", decode_regs(self), instruction::not_implemented)
            }
            // Annn
            (0xA, _, _, _) => {
                Instruction::create(self, "LD", decode_addr(self), instruction::ld_annn)
            }
            // Bnnn
            (0xB, _, _, _) => {
                Instruction::create(self, "JP", decode_addr(self), instruction::not_implemented)
            }
            // Cxkk
            (0xC, _, _, _) => Instruction::create(
                self,
                "RND",
                decode_reg_const(self),
                instruction::not_implemented,
            ),
            // Dxyn
            (0xD, _, _, _) => Instruction::create(
                self,
                "DRW",
                decode_regs_const(self),
                instruction::not_implemented,
            ),
            // Ex9E
            (0xE, _, 9, 0xE) => {
                Instruction::create(self, "SKP", decode_reg(self), instruction::not_implemented)
            }
            // ExA1
            (0xE, _, 0xA, 1) => {
                Instruction::create(self, "SKNP", decode_reg(self), instruction::not_implemented)
            }
            // Fx07
            (0xF, _, 0, 7) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::not_implemented)
            }
            // Fx0A
            (0xF, _, 0, 0xA) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::not_implemented)
            }
            // Fx15
            (0xF, _, 1, 5) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::not_implemented)
            }
            // Fx18
            (0xF, _, 1, 8) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::not_implemented)
            }
            // Fx1E
            (0xF, _, 1, 0xE) => {
                Instruction::create(self, "ADD", decode_reg(self), instruction::not_implemented)
            }
            // Fx29
            (0xF, _, 2, 9) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::ld_fx29)
            }
            // Fx33
            (0xF, _, 3, 3) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::not_implemented)
            }
            // Fx55
            (0xF, _, 5, 5) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::ld_fx55)
            }
            // Fx65
            (0xF, _, 6, 5) => {
                Instruction::create(self, "LD", decode_reg(self), instruction::ld_fx65)
            }
            _ => {
                warn!("Failed to decode: `{:#06X}`", self);
                Instruction::create(self, "???", Operands::Empty, instruction::not_implemented)
            }
        };
        trace!("Decoded `{:#06X}` into `{}`", self, ins);
        ins
    }
}

// -------- //
// Decoders //
// -------- //

fn decode_addr(op: OpCode) -> Operands {
    Operands::Address(op.0 & 0x0FFF)
}

fn decode_reg(op: OpCode) -> Operands {
    Operands::Reg(((op.0 & 0x0F00) >> 8) as u8)
}

fn decode_regs(op: OpCode) -> Operands {
    Operands::Regs(((op.0 & 0x0F00) >> 8) as u8, ((op.0 & 0x00F0) >> 4) as u8)
}

fn decode_reg_const(op: OpCode) -> Operands {
    Operands::RegAndConst(((op.0 & 0x0F00) >> 8) as u8, (op.0 & 0x00FF) as u8)
}

fn decode_regs_const(op: OpCode) -> Operands {
    Operands::RegsAndConst(
        ((op.0 & 0x0F00) >> 8) as u8,
        ((op.0 & 0x00F0) >> 4) as u8,
        ((op.0 & 0x000F) >> 0) as u8,
    )
}

// ------------------ //
// Formatting Support //
// ------------------ //

impl fmt::UpperHex for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;
        fmt::UpperHex::fmt(&val, f)
    }
}

impl fmt::Display for Operands {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operands::Empty => write!(f, ""),
            Operands::Address(addr) => write!(f, "{:#03X}", addr),
            Operands::Reg(reg) => write!(f, "v{:X}", reg),
            Operands::Regs(regx, regy) => write!(f, "v{:X} v{:X}", regx, regy),
            Operands::RegAndConst(reg, cnst) => write!(f, "v{:X} {:#04X}", reg, cnst),
            Operands::RegsAndConst(regx, regy, cnst) => {
                write!(f, "v{:X} v{:X} {:#03X}", regx, regy, cnst)
            }
        }
    }
}
