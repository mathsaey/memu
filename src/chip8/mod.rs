mod instruction;
mod opcode;

use bitvec::vec::BitVec;
use log::*;

use std::ops::{Index, IndexMut};

use super::debug_view::{Frame, Rect};
use opcode::{OpCode, Operands};

// --------- //
// Constants //
// --------- //

const STACK_SIZE: usize = 16;
const GP_AMOUNT: usize = 16;
const MEM_SIZE: usize = 4 * 1024;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const PX_SET: u32 = super::display::Display::rgb(0xFF, 0xFF, 0xFF);
const PX_UNS: u32 = super::display::Display::rgb(0x00, 0x00, 0x00);

// --------------- //
// Data Structures //
// --------------- //

pub struct Chip8 {
    // Main Memory
    mem: Mem,
    // Stack
    stack: Vec<u16>, // Stack to store program counter
    // Registers
    regs: Regs,       // General purpose, V0 to VF
    reg_i: u16,       // Address register
    reg_pc: u16,      // Program counter (pseudo)
    reg_dt: u8,       // Delay timer
    reg_st: u8,       // Sound timer
    // Graphics
    screen: BitVec,   // Screen
}

// Avoid constant typecasting in instructions
struct Regs([u8; GP_AMOUNT]);

impl Regs {
    fn new() -> Regs {
        Regs([0x00 ; GP_AMOUNT])
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
    fn clock_rate(&self) -> usize {
        1500
    }

    fn screen_dimensions(&self) -> (usize, usize) {
        (WIDTH, HEIGHT)
    }

    fn load_rom(&mut self, content: Vec<u8>) {
        for (ctr, el) in content.into_iter().enumerate() {
            self.mem[self.reg_pc + (ctr as u16)] = el;
        }
    }

    fn cycle(&mut self) -> bool {
        self.fetch().decode().exec(self)
    }

    fn draw_screen(&self) -> Vec<u32> {
        self.screen.iter().map(|&b| {
            if b { PX_SET } else { PX_UNS }
        }).collect()
    }

    fn draw_debug(&self, frame: &mut Frame, rect: Rect) {
        draw_debug(&self, frame, rect)
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
            screen: BitVec::repeat(false, WIDTH * HEIGHT)
        };

        res.load_sprites();
        res
    }

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

    fn fetch(&mut self) -> OpCode {
        let code = self.get_opcode(self.reg_pc);
        trace!("Fetched `{:#06X}` from ${:#06X}", code, self.reg_pc);
        self.pc_inc();
        code
    }

    fn get_opcode(&self, idx: u16) -> OpCode {
        OpCode::from_cells(self.mem[idx], self.mem[idx + 1])
    }

    #[inline]
    fn sprite_addr(&self, digit: u8) -> u16 {
        (digit * 5) as u16
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

// ---------- //
// Debug View //
// ---------- //

#[cfg(feature = "debug-view")]
use tui::{
    style::*,
    widgets::*,
    layout::{Constraint, Direction, Layout}
};

#[inline]
#[cfg(not(feature = "debug-view"))]
fn draw_debug(_state: &Chip8, _frame: &mut Frame, _rect: Rect) {
}

#[inline]
#[cfg(feature = "debug-view")]
fn draw_debug(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(rect);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(8),
                Constraint::Length(4),
                Constraint::Percentage(45),
            ]
            .as_ref(),
        )
        .split(top[1]);

    draw_memory(state, frame, top[0]);
    draw_registers(state, frame, right[0]);
    draw_stack(state, frame, right[1]);
    draw_instructions(state, frame, right[2]);
}

#[cfg(feature = "debug-view")]
fn draw_instructions(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let rows = rect.height - 3;

    let instructions = (0..rows).map(|i| {
        let addr = state.reg_pc + (i * 2);
        let instruction = state.get_opcode(addr).decode();

        let a = format!("${:#05X}", addr);
        let c = format!("({:04X})", instruction.code);
        let n = String::from(instruction.name);

        let v = match instruction.operands {
            Operands::Empty => vec![a, c, n],
            Operands::Address(addr) => vec![a, c, n, format!("${:#03X}", addr)],
            Operands::Reg(reg) => vec![a, c, n, format!("v{:X}", reg)],
            Operands::Regs(regx, regy) => {
                vec![a, c, n, format!("v{:X}", regx), format!("v{:X}", regy)]
            }
            Operands::RegAndConst(reg, cnst) => {
                vec![a, c, n, format!("v{:X}", reg), format!("{:#04X}", cnst)]
            }
            Operands::RegsAndConst(regx, regy, cnst) => vec![
                a,
                c,
                n,
                format!("v{:X}", regx),
                format!("v{:X}", regy),
                format!("{:#03X}", cnst),
            ],
        };

        Row::Data(v.into_iter())
    });

    let tab = Table::new(
        ["Addr", "Code", "Op", "a1", "a2", "a3"].iter(),
        instructions,
    )
    .block(Block::default().title("Instructions").borders(Borders::ALL))
    .widths(&[
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
    ])
    .header_style(Style::default().fg(Color::Gray))
    .header_gap(0)
    .style(Style::default().fg(Color::White));

    frame.render_widget(tab, rect);
}

#[cfg(feature = "debug-view")]
fn draw_registers(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut regs: Vec<Text> = Vec::with_capacity(50);

    let name_style = Style::default().fg(Color::Blue);

    for (idx, reg) in state.regs.0.iter().enumerate() {
        regs.push(Text::styled(format!("v{:X} ", idx), name_style));
        regs.push(Text::raw(format!("{:#04X} ", reg)));
        if (idx + 1) % 4 == 0 {
            regs.push(Text::raw("\n"))
        }
    }

    regs.push(Text::styled("DT ", name_style));
    regs.push(Text::raw(format!("{:#04X} ", state.reg_dt)));

    regs.push(Text::styled("I  ", name_style));
    regs.push(Text::raw(format!("{:#06X}", state.reg_i)));

    regs.push(Text::raw("\n"));

    regs.push(Text::styled("ST ", name_style));
    regs.push(Text::raw(format!("{:#04X} ", state.reg_st)));
    regs.push(Text::styled("PC ", name_style));
    regs.push(Text::raw(format!("{:#06X}", state.reg_pc)));

    let par = Paragraph::new(regs.iter())
        .block(Block::default().title("Registers").borders(Borders::ALL))
        .alignment(tui::layout::Alignment::Left);

    frame.render_widget(par, rect);
}

#[cfg(feature = "debug-view")]
fn draw_stack(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut slots: Vec<Text> = Vec::with_capacity(33);

    for (idx, addr) in state.stack.iter().enumerate() {
        slots.push(Text::raw(format!("{:#03}", addr)));
        slots.push(Text::styled("|", Style::default().fg(Color::Gray)));
        if (idx + 1) % 8 == 0 {
            slots.push(Text::raw("\n"))
        }
    }

    let par = Paragraph::new(slots.iter())
        .block(Block::default().title("Stack").borders(Borders::ALL))
        .alignment(tui::layout::Alignment::Center);

    frame.render_widget(par, rect);
}

#[cfg(feature = "debug-view")]
fn draw_memory(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut constraints = [Constraint::Length(3); 17];
    constraints[0] = Constraint::Length(8);

    let mut headers: Vec<String> = Vec::with_capacity(17);
    headers.push(String::from("Address"));
    for i in 0..(0xF + 1) {
        headers.push(format!("{:X}", i));
    }

    let rows = (0..(rect.height - 3)).map(move |row_idx| {
        let start_addr = state.reg_i - (state.reg_i % 0x10) + (row_idx * 16);
        let mut vec: Vec<String> = Vec::with_capacity(17);
        vec.push(format!("${:#06X}", start_addr));

        for addr in start_addr..(start_addr + 16) {
            vec.push(format!("{:02X}", state.mem[addr]));
        }

        Row::Data(vec.into_iter())
    });

    let tab = Table::new(headers.iter(), rows)
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .header_gap(0)
        .header_style(Style::default().fg(Color::Gray))
        .widths(&constraints)
        .style(Style::default().fg(Color::White));

    frame.render_widget(tab, rect);
}
