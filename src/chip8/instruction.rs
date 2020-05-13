use log::*;
use std::fmt;

use super::opcode::{OpCode, Operands};
use super::Chip8;

pub type InsFn = fn(&mut Chip8, Operands) -> bool;
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

    pub fn exec(self, chip8: &mut Chip8) -> bool {
        trace!("Execute `{}`", self);
        let inst = self.instruction;
        inst(chip8, self.operands)
    }
}

// ------------ //
// Instructions //
// ------------ //

pub fn not_implemented(e: &mut Chip8, _: Operands) -> bool {
    let instruction = e.get_opcode(e.reg_pc - 2).decode();
    warn!("Ignoring unimplemented instruction: {}", instruction);
    false
}

pub fn cls_00e0(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Empty = o {
        for mut px in e.screen.iter_mut() {
            *px = false;
        }
    }
    true
}

pub fn ret_00ee(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Empty = o {
        e.reg_pc = e.stack.pop().unwrap();
    }
    false
}

pub fn jp_1nnn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Address(a) = o {
        e.reg_pc = a;
    }
    false
}

pub fn call_2nnn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Address(a) = o {
        e.stack.push(e.reg_pc);
        e.reg_pc = a;
    }
    false
}

pub fn se_3xnn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegAndConst(r, c) = o {
        if e.regs[r] == c {
            e.pc_inc();
        }
    }
    false
}

pub fn sne_4xnn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegAndConst(r, c) = o {
        if e.regs[r] != c {
            e.pc_inc();
        }
    }
    false
}

pub fn se_5xy0(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        if e.regs[x] == e.regs[y] {
            e.pc_inc();
        }
    }
    false
}

pub fn ld_6xkk(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegAndConst(r, c) = o {
        e.regs[r] = c;
    }
    false
}

pub fn add_7xkk(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegAndConst(r, k) = o {
        let res = (e.regs[r] as u16) + (k as u16);
        // Flag is not set if overflow occurs
        e.regs[r] = (res & 0x00FF) as u8;
    }
    false
}

pub fn ld_8xy0(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        e.regs[x] = e.regs[y];
    }
    false
}

pub fn or_8xy1(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        e.regs[x] |= e.regs[y];
    }
    false
}

pub fn and_8xy2(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        e.regs[x] &= e.regs[y];
    }
    false
}

pub fn xor_8xy3(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        e.regs[x] ^= e.regs[y];
    }
    false
}

pub fn add_8xy4(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        let res = (e.regs[x] as u16) + (e.regs[y] as u16);
        if res > 255 {
            e.set_flag()
        } else {
            e.clear_flag()
        }
        e.regs[x] = (res & 0x00FF) as u8;
    }
    false
}

pub fn sub_8xy5(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        let lhs = e.regs[x];
        let rhs = e.regs[y];
        if  lhs > rhs {
            e.set_flag();
            e.regs[x] = lhs - rhs;
        } else {
            e.clear_flag();
            e.regs[x] = 0xFF - (rhs - lhs);
        }
    }
    false
}


pub fn shr_8xy6(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, _) = o {
        // Super Chip8 behaviour
        let val = e.regs[x];
        e.regs[0xF] = val & 0x01;
        e.regs[x] = val >> 1;
    }
    false
}

pub fn sub_8xy7(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, y) = o {
        let lhs = e.regs[x];
        let rhs = e.regs[y];
        if  rhs > lhs {
            e.set_flag();
            e.regs[x] = rhs - lhs;
        } else {
            e.clear_flag();
            e.regs[x] = 0xFF - (lhs - rhs);
        }
    }
    false
}

pub fn shl_8xye(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Regs(x, _) = o {
        // Super Chip8 behaviour
        let val = e.regs[x];
        e.regs[0xF] = (val & 0b10000000) >> 7;
        e.regs[x] = val << 1;
    }
    false
}

pub fn ld_annn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Address(a) = o {
        e.reg_i = a;
    }
    false
}

pub fn rnd_cxkk(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegAndConst(r, c) = o {
        let rand: u8 = rand::random();
        e.regs[r] = rand & c;
    }
    false
}

pub fn drw_dxyn(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::RegsAndConst(x, y, c) = o {
        // Reset flag register
        e.clear_flag();

        let mut collision = false;

        // Fetch the sprite
        let i = e.reg_i as usize;
        let sprite = &e.mem.0[i..i + c as usize];

        // Feth the location to draw
        // Coordinates should wrap around
        let base_x = e.regs[x] as usize % super::WIDTH;
        let base_y = e.regs[y] as usize % super::HEIGHT;
        let c = c as usize;

        // Iterate over every bit in the sprite, to chec if it is set
        for (sprite_y, y) in (base_y..(base_y + c)).enumerate() {
            for (sprite_x, x) in (base_x..(base_x + 8)).enumerate() {
                if (sprite[sprite_y] & (0b10000000 >> sprite_x)) != 0 {

                    let addr = (y * super::WIDTH + x) as usize;

                    // Collision check
                    if e.screen[addr] {
                        collision = true;
                    }

                    // Update the display
                    let cur = e.screen[addr];
                    e.screen.set(addr, cur ^ true);
                }
            }
        }

        // Need to do this out of the loop to make the borrow checker happy
        if collision {
            e.set_flag();
        }
    }
    true
}

pub fn skp_ex9e(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        if e.keypad[r as usize] {
            e.pc_inc();
        }
    }
    false
}

pub fn sknp_exa1(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        if !e.keypad[r as usize] {
            e.pc_inc();
        }
    }
    false
}

pub fn ld_fx07(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        e.regs[r] = e.reg_dt;
    }
    false
}

pub fn ld_fx0a(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        e.await_press = Some(r);
    }
    false
}

pub fn ld_fx15(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        e.reg_dt = e.regs[r];
    }
    false
}

pub fn add_fx1e(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        e.reg_i += e.regs[r] as u16;
    }
    false
}

pub fn ld_fx33(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        let num = e.regs[r];

        e.mem[e.reg_i + 0] = num / 100;
        e.mem[e.reg_i + 1] = (num % 100) / 10;
        e.mem[e.reg_i + 2] = num % 10;

    }
    false
}

pub fn ld_fx55(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        // Super Chip8 behaviour
        for ctr in 0..(r + 1) {
            e.mem[e.reg_i + ctr as u16] = e.regs[ctr];
        }
    }
    false
}

pub fn ld_fx65(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        // Super Chip8 behaviour
        for ctr in 0..(r + 1) {
            e.regs[ctr] = e.mem[e.reg_i + ctr as u16];
        }
    }
    false
}

pub fn ld_fx29(e: &mut Chip8, o: Operands) -> bool {
    if let Operands::Reg(r) = o {
        let addr = e.regs[r] & 0x0F;
        e.reg_i = e.sprite_addr(addr);
    }
    false
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
