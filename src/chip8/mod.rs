mod instruction;
mod opcode;

use bitvec::vec::BitVec;
use ggez::{graphics::*, input::keyboard::KeyCode, *};
use log::*;

use std::ops::{Index, IndexMut};
use std::time::Duration;

use opcode::OpCode;

#[cfg(feature = "debug-view")]
mod debug_view;

#[cfg(not(feature = "debug-view"))]
impl crate::debug_view::Debug for Chip8 {}

// --------- //
// Constants //
// --------- //

const STACK_SIZE: usize = 16;
const GP_AMOUNT: usize = 16;
const MEM_SIZE: usize = 4 * 1024;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

// Chip8 cycles around 500Hz = 2 ms per cycle
const CYCLE_TIME: Duration = Duration::from_millis(2);
// Times cycle down at 60Hz = 16.6 ms per cycle
const TIMER_TIME: Duration = Duration::from_nanos(16666666);

// --------------- //
// Data Structures //
// --------------- //

pub struct Chip8 {
    // Main Memory
    mem: Mem,
    // Stack
    stack: Vec<u16>, // Stack to store program counter
    // Registers
    regs: Regs,              // General purpose, V0 to VF
    reg_i: u16,              // Address register
    reg_pc: u16,             // Program counter (pseudo)
    reg_dt: u8,              // Delay timer
    reg_st: u8,              // Sound timer
    // Graphics
    screen: BitVec,          // Screen
    // Keypad
    keypad: BitVec,          // Keypad state
    await_press: Option<u8>, // Some(reg) if the emulator is waiting for a keypress
    // Timing
    cycle_timer: Duration,   // Elapsed time since last cycle
    clock_timer: Duration    // Timer for sound / delay clocks
}

// Avoid constant typecasting in instructions
struct Regs([u8; GP_AMOUNT]);

impl Regs {
    fn new() -> Regs {
        Regs([0x00; GP_AMOUNT])
    }
}

impl Index<u8> for Regs {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for Regs {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

struct Mem([u8; MEM_SIZE]);

impl Mem {
    pub fn new() -> Mem {
        Mem([0x00; MEM_SIZE])
    }
}

impl Index<u16> for Mem {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for Mem {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

// -------------- //
// Emulator Logic //
// -------------- //

impl crate::Emulator for Chip8 {
    fn load_rom(&mut self, content: Vec<u8>) {
        for (ctr, el) in content.into_iter().enumerate() {
            self.mem[self.reg_pc + (ctr as u16)] = el;
        }
    }

    fn advance(&mut self, elapsed: std::time::Duration) -> bool {
        self.cycle_timer += elapsed;
        self.clock_timer += elapsed;

        let mut draw = false;

        while self.cycle_timer > CYCLE_TIME {
            self.cycle_timer -= CYCLE_TIME;
            draw = draw || self.cycle();
        }

        while self.clock_timer > TIMER_TIME {
            self.clock_timer -= TIMER_TIME;
            if self.reg_dt > 0 {
                self.reg_dt -= 1;
            }
            if self.reg_st > 0 {
                self.reg_st -= 1;
            }
        }

        draw
    }

    fn cycle_dt(&self) -> std::time::Duration {
        CYCLE_TIME
    }

    fn key_down(&mut self, key: KeyCode) {
      if let Some(idx) = self.translate_key(key) {
        if let Some(r) = self.await_press {
          self.regs[r] = idx as u8;
          self.await_press = None;
        }
        self.keypad.set(idx, true);
      }
    }

    fn key_up(&mut self, key: KeyCode) {
      if let Some(idx) = self.translate_key(key) {
        self.keypad.set(idx, false);
      }
    }

    fn draw_size(&self) -> (f32, f32) {
        (WIDTH as f32, HEIGHT as f32)
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for (idx, &pixel_set) in self.screen.iter().enumerate() {
            let y = idx / WIDTH;
            let x = idx % WIDTH;

            if pixel_set {
                crate::utils::draw_pixel(ctx, x, y, WHITE)?;
            }
        }
        Ok(())
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut res = Chip8 {
            mem: Mem::new(),
            stack: Vec::with_capacity(STACK_SIZE),
            regs: Regs::new(),
            reg_i: 0x000,
            reg_pc: 0x200, // Programs start at 0x200
            reg_dt: 0x00,
            reg_st: 0x00,
            screen: BitVec::repeat(false, WIDTH * HEIGHT),
            keypad: BitVec::repeat(false, 16),
            await_press: None,
            cycle_timer: Duration::from_millis(0),
            clock_timer: Duration::from_millis(0)
        };

        res.load_sprites();
        res
    }


    // Cycle
    // -----

    fn cycle(&mut self) -> bool {
      match self.await_press {
        None => self.fetch().decode().exec(self),
        Some(_) => false
      }
    }

    fn fetch(&mut self) -> OpCode {
        let code = self.get_opcode(self.reg_pc);
        trace!("Fetched `{:#06X}` from ${:#06X}", code, self.reg_pc);
        self.pc_inc();
        code
    }

    fn get_opcode(&self, idx: u16) -> OpCode {
        OpCode::from_cells(self.mem[idx], self.mem[idx + 1])
    }


    // Utilities
    // ---------

    #[inline]
    fn pc_inc(&mut self) {
        self.reg_pc += 2;
    }

    #[inline]
    fn clear_flag(&mut self) {
        self.regs[0xF] = 0;
    }

    #[inline]
    fn set_flag(&mut self) {
        self.regs[0xF] = 1;
    }

    // Key Translation
    // ---------------

    /// Map a keycode onto a hex value.
    ///
    /// We map the layout of the chip8 hex keypad onto the qwerty keyboard.
    ///
    /// | key | key | key | key |
    /// |---|---|---|---|
    /// | 1 | 2 | 3 | C |
    /// | 4 | 5 | 6 | D |
    /// | 7 | 8 | 9 | E |
    /// | A | 0 | B | F |
    ///
    /// is mapped onto
    ///
    /// | key | key | key | key |
    /// |---|---|---|---|
    /// | 1 | 2 | 3 | 4 |
    /// | q | w | e | r |
    /// | a | s | d | f |
    /// | z | x | c | v |
    ///
    #[rustfmt::skip]
    fn translate_key(&self, key: KeyCode) -> Option<usize> {
        match key {
          KeyCode::Key1 => Some(0x1), KeyCode::Key2 => Some(0x2), KeyCode::Key3 => Some(0x3), KeyCode::Key4 => Some(0xC),
          KeyCode::Q    => Some(0x4), KeyCode::W    => Some(0x5), KeyCode::E    => Some(0x6), KeyCode::R    => Some(0xD),
          KeyCode::A    => Some(0x7), KeyCode::S    => Some(0x8), KeyCode::D    => Some(0x9), KeyCode::F    => Some(0xE),
          KeyCode::Z    => Some(0xA), KeyCode::X    => Some(0x0), KeyCode::C    => Some(0xB), KeyCode::V    => Some(0xF),
          _ => None
        }
    }

    // Built-in Sprites
    // ----------------

    #[inline]
    fn sprite_addr(&self, digit: u8) -> u16 {
        digit as u16 * 5
    }

    fn load_sprite(&mut self, digit: u8, sprite: &[u8; 5]) {
        let addr = self.sprite_addr(digit) as usize;
        self.mem.0[addr..(addr + 5)].copy_from_slice(sprite);
    }

    fn load_sprites(&mut self) {
        self.load_sprite(0x0, &[0xF0, 0x90, 0x90, 0x90, 0xF0]);
        self.load_sprite(0x1, &[0x20, 0x60, 0x20, 0x20, 0x70]);
        self.load_sprite(0x2, &[0xF0, 0x10, 0xF0, 0x80, 0xF0]);
        self.load_sprite(0x3, &[0xF0, 0x10, 0xF0, 0x10, 0xF0]);
        self.load_sprite(0x4, &[0x90, 0x90, 0xF0, 0x10, 0x10]);
        self.load_sprite(0x5, &[0xF0, 0x80, 0xF0, 0x10, 0xF0]);
        self.load_sprite(0x6, &[0xF0, 0x80, 0xF0, 0x90, 0xF0]);
        self.load_sprite(0x7, &[0xF0, 0x10, 0x20, 0x40, 0x40]);
        self.load_sprite(0x8, &[0xF0, 0x90, 0xF0, 0x90, 0xF0]);
        self.load_sprite(0x9, &[0xF0, 0x90, 0xF0, 0x10, 0xF0]);
        self.load_sprite(0xA, &[0xF0, 0x90, 0xF0, 0x90, 0x90]);
        self.load_sprite(0xB, &[0xE0, 0x90, 0xE0, 0x90, 0xE0]);
        self.load_sprite(0xC, &[0xF0, 0x80, 0x80, 0x80, 0xF0]);
        self.load_sprite(0xD, &[0xE0, 0x90, 0x90, 0x90, 0xE0]);
        self.load_sprite(0xE, &[0xF0, 0x80, 0xF0, 0x80, 0xF0]);
        self.load_sprite(0xF, &[0xF0, 0x80, 0xF0, 0x80, 0x80]);
    }
}
