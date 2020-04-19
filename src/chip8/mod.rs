mod instruction;
mod opcode;

use log::*;

use super::debug_view::{Frame, Rect};
use opcode::OpCode;

// TODO:
//  - move stack to separate module, push & pop
//  - Or just use dequeue?

const STACK_SIZE: usize = 16;
const GP_AMOUNT: usize = 16;
const MEM_SIZE: usize = 4 * 1024;
const SCREEN: (usize, usize) = (64, 32);

pub struct Chip8 {
    // Main Memory
    mem: [u8; MEM_SIZE],
    // Stack
    stack: [u16; STACK_SIZE], // Stack to store program counter
    // Registers
    regs: [u8; GP_AMOUNT], // General purpose, V0 to VF
    reg_i: u16,            // Address register
    reg_pc: u16,           // Program counter (pseudo)
    reg_dt: u8,            // Delay timer
    reg_st: u8,            // Sound timer
    // Graphics
    screen: Vec<u32>       // Screen
}

impl crate::Emulator for Chip8 {
    fn clock_rate(&self) -> usize {
        1500
    }

    fn screen_dimensions(&self) -> (usize, usize) {
        SCREEN
    }

    fn load_rom(&mut self, content: Vec<u8>) {
        for (ctr, el) in content.into_iter().enumerate() {
            self.mem[self.reg_pc as usize + ctr] = el;
        }
    }

    fn cycle(&mut self) -> bool {
        let opcode = self.fetch();
        let instruction = opcode.decode();
        trace!("Decoded `{:#06X}` into `{}`", opcode, instruction);
        instruction.exec(self);
        false
    }

    fn screen_buffer(&self) -> &[u32] {
        &self.screen
    }

    fn draw_debug(&self, frame: &mut Frame, rect: Rect) {
        draw_debug(&self, frame, rect)
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let (width, height) = SCREEN;

        Chip8 {
            mem: [0x00; MEM_SIZE],
            stack: [0x00; STACK_SIZE],
            regs: [0x00; GP_AMOUNT],
            reg_i: 0x000,
            reg_pc: 0x200, // Programs start at 0x200
            reg_dt: 0x00,
            reg_st: 0x00,
            screen: vec![0x0; width * height]
        }
    }

    fn fetch(&mut self) -> OpCode {
        let code = self.get_opcode(self.reg_pc);
        trace!("Fetched `{:#06X}` from ${:#06X}", code, self.reg_pc);
        self.reg_pc += 2;
        code
    }

    fn get_opcode(&self, idx: u16) -> OpCode {
        OpCode::from_cells(self.mem[idx as usize], self.mem[(idx + 1) as usize])
    }
}

// ---------- //
// Debug View //
// ---------- //

use tui::layout::{Constraint, Direction, Layout};
use tui::style::*;
use tui::widgets::*;

#[inline]
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

fn draw_instructions(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let rows = rect.height - 3;

    let instructions = (0..rows).map(|i| {
        let addr = state.reg_pc + (i * 2);
        let instruction = state.get_opcode(addr).decode();

        Row::Data(
            vec![
                format!("${:#05X}", addr),
                String::from(instruction.name),
                format!("{}", instruction.operands),
            ]
            .into_iter(),
        )
    });

    let tab = Table::new(["Addr", "Op", "Args"].iter(), instructions)
        .block(Block::default().title("Instructions").borders(Borders::ALL))
        .widths(&[
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(10),
        ])
        .header_style(Style::default().fg(Color::Gray))
        .header_gap(0)
        .style(Style::default().fg(Color::White));

    frame.render_widget(tab, rect);
}

fn draw_registers(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut regs: Vec<Text> = Vec::with_capacity(50);

    let name_style = Style::default().fg(Color::Blue);

    for (idx, reg) in state.regs.iter().enumerate() {
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

// TODO: Make this better later when the stack pointer is implemented
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

fn draw_memory(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut constraints = [Constraint::Length(3); 17];
    constraints[0] = Constraint::Length(8);

    let mut headers: Vec<String> = Vec::with_capacity(17);
    headers.push(String::from("Address"));
    for i in 0..(0xF + 1) {
        headers.push(format!("{:X}", i));
    }

    let rows = (0..(rect.height - 3)).map(move |row_idx| {
        let start_addr = state.reg_i + (row_idx * 16);
        let mut vec: Vec<String> = Vec::with_capacity(17);
        vec.push(format!("${:#06X}", start_addr));

        for addr in start_addr..(start_addr + 16) {
            vec.push(format!("{:02X}", state.mem[(addr) as usize]));
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
