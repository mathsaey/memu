use tui::style::*;
use tui::widgets::*;
use tui::layout::{Constraint, Direction, Layout};

use super::{Chip8, opcode::Operands};
use crate::debug_view::{Frame, Rect};


impl crate::debug_view::Debug for Chip8 {
    fn debug_view(&self, frame: &mut Frame, rect: Rect) {
        draw_debug(&self, frame, rect)
    }
}

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

fn draw_registers(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut regs: Vec<Text> = Vec::with_capacity(50);

    let name_style = Style::default().fg(Color::Blue);
    let wait_style = Style::default().fg(Color::Red);

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

    if let Some(_) = state.await_press {
        regs.push(Text::styled("     Await Key", wait_style));
    };

    regs.push(Text::raw("\n"));

    regs.push(Text::styled("ST ", name_style));
    regs.push(Text::raw(format!("{:#04X} ", state.reg_st)));
    regs.push(Text::styled("PC ", name_style));
    regs.push(Text::raw(format!("{:#06X}", state.reg_pc)));

    if let Some(r) = state.await_press {
        regs.push(Text::styled(format!("          (v{:X})", r), wait_style));
    };

    let par = Paragraph::new(regs.iter())
        .block(Block::default().title("Registers").borders(Borders::ALL))
        .alignment(tui::layout::Alignment::Left);

    frame.render_widget(par, rect);
}

fn draw_stack(state: &Chip8, frame: &mut Frame, rect: Rect) {
    let mut slots: Vec<Text> = Vec::with_capacity(33);

    for (idx, addr) in state.stack.iter().enumerate() {
        slots.push(Text::raw(format!("{:#05X}", addr)));
        slots.push(Text::styled(" |", Style::default().fg(Color::Gray)));
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
