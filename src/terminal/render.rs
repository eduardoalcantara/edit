//! Desenho do painel terminal (output + coluna de sessões).

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::selection::TerminalSelection;

use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

use super::workspace::{
    effective_sidebar_width, layout_terminal_panel, terminal_split_col, SidebarClick,
    TerminalPanelLayout, TerminalWorkspace, SIDEBAR_COLS,
};

pub fn paint_terminal_panel(
    frame: &mut Frame,
    shell: Rect,
    term_outer: Rect,
    panel: TerminalPanelLayout,
    workspace: &TerminalWorkspace,
    palette: ThemePalette,
    active_index: usize,
) {
    let border_style = Style::default()
        .fg(palette.border)
        .bg(palette.editor_bg);
    let text_style = Style::default()
        .fg(palette.editor_fg)
        .bg(palette.editor_bg);

    panel::fill_rect(frame, panel.outer, text_style);

    if let Some(session) = workspace.sessions.get(active_index) {
        let h = panel.output.height as usize;
        let indexed = session.scrollback.visible_tail_indexed(
            h,
            session.scroll_offset,
            session.follow_tail,
        );
        let sel_style = Style::default()
            .fg(palette.editor_bg)
            .bg(palette.editor_fg)
            .add_modifier(Modifier::BOLD);
        let width = panel.output.width as usize;
        for (row, (global_line, line)) in indexed.iter().enumerate() {
            paint_terminal_output_row(
                frame,
                panel.output,
                row as u16,
                line,
                width,
                text_style,
                sel_style,
                workspace.selection,
                *global_line,
            );
        }
    } else {
        panel::render_plain_row(
            frame,
            panel.output,
            0,
            " Nenhuma sessão — clique [n] ",
            text_style,
        );
    }

    paint_sidebar(
        frame,
        panel.sidebar,
        workspace,
        active_index,
        palette,
        text_style,
        workspace.sidebar_hover,
    );

    if panel.output.width > 0 && panel.sidebar.width > 0 {
        let c = PanelBorder::Plain.chars();
        for row in 0..panel.outer.height {
            panel::render_cell(
                frame,
                panel.output.x.saturating_add(panel.output.width),
                panel.outer.y.saturating_add(row),
                c.v,
                border_style,
            );
        }
    }

    paint_terminal_side_borders(frame, shell, term_outer, border_style);
}

/// Reforça `│` nas laterais da shell na faixa do terminal (z-order acima do fill).
pub fn paint_terminal_side_borders(
    frame: &mut Frame,
    shell: Rect,
    term_outer: Rect,
    border_style: Style,
) {
    if term_outer.height < 3 {
        return;
    }
    let c = PanelBorder::Plain.chars();
    let left = shell.x;
    let right = shell.x.saturating_add(shell.width.saturating_sub(1));
    let first = term_outer.y.saturating_add(1);
    let last = term_outer.y.saturating_add(term_outer.height.saturating_sub(2));
    for row in first..=last {
        panel::render_cell(frame, left, row, c.v, border_style);
        panel::render_cell(frame, right, row, c.v, border_style);
    }
}

fn paint_sidebar(
    frame: &mut Frame,
    area: Rect,
    workspace: &TerminalWorkspace,
    active: usize,
    palette: ThemePalette,
    text_style: Style,
    hover: Option<SidebarClick>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let w = area.width as usize;
    let mut row = 0u16;

    paint_sidebar_top_row(frame, area, row, w, palette, text_style, hover);
    row += 1;

    let last_data_row = area.height.saturating_sub(1);
    for (i, session) in workspace.sessions.iter().enumerate() {
        if row > last_data_row {
            break;
        }
        let mark = if i == active { '*' } else { ' ' };
        let close_label = "[q]";
        let close_w = 3usize;
        let prefix_w = 4usize;
        let name_w = w.saturating_sub(prefix_w + close_w).max(1);
        let name = truncate_line(&session.label, name_w);
        let prefix = format!(
            "{mark}{idx:02} {name:<name_w$}",
            mark = mark,
            idx = i + 1,
            name = name,
            name_w = name_w
        );
        let row_style = if i == active {
            text_style.add_modifier(Modifier::BOLD)
        } else {
            text_style
        };
        panel::render_plain_row(frame, area, row, &prefix, row_style);

        let close_action = SidebarClick::CloseSession(i);
        let focus_action = SidebarClick::FocusSession(i);
        let q_hovered = hover == Some(close_action);
        let row_hovered = hover == Some(focus_action);
        let q_style = if q_hovered {
            palette.button_style(true)
        } else if row_hovered {
            row_style.add_modifier(Modifier::UNDERLINED)
        } else {
            palette.button_style(false)
        };
        let q_x = area.x.saturating_add((w.saturating_sub(close_w)) as u16);
        frame.render_widget(
            Paragraph::new(close_label).style(q_style),
            Rect {
                x: q_x,
                y: area.y.saturating_add(row),
                width: close_w as u16,
                height: 1,
            },
        );
        row += 1;
    }
}

fn paint_terminal_output_row(
    frame: &mut Frame,
    area: Rect,
    row_index: u16,
    line: &str,
    width: usize,
    text_style: Style,
    sel_style: Style,
    selection: Option<TerminalSelection>,
    global_line: usize,
) {
    if area.width == 0 || row_index >= area.height {
        return;
    }
    let display: String = line.chars().take(width).collect();
    if let Some(sel) = selection {
        let mut spans = Vec::new();
        let mut col = 0usize;
        for ch in display.chars() {
            let style = if sel.contains(global_line, col) {
                sel_style
            } else {
                text_style
            };
            spans.push(Span::styled(ch.to_string(), style));
            col += 1;
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(text_style),
            Rect {
                x: area.x,
                y: area.y.saturating_add(row_index),
                width: area.width,
                height: 1,
            },
        );
    } else {
        panel::render_plain_row(frame, area, row_index, &display, text_style);
    }
}

fn paint_sidebar_top_row(
    frame: &mut Frame,
    area: Rect,
    row: u16,
    width: usize,
    palette: ThemePalette,
    text_style: Style,
    hover: Option<SidebarClick>,
) {
    let y = area.y.saturating_add(row);
    let mut x = area.x;

    let buttons: [(&str, SidebarClick); 3] = [("[n]", SidebarClick::NewSession), ("[+]", SidebarClick::GrowPanel), ("[-]", SidebarClick::ShrinkPanel)];
    for (label, action) in buttons {
        if width < label.len() {
            break;
        }
        let btn_w = label.chars().count() as u16;
        let style = if hover == Some(action) {
            palette.button_style(true)
        } else {
            text_style
        };
        frame.render_widget(
            Paragraph::new(label).style(style),
            Rect {
                x,
                y,
                width: btn_w,
                height: 1,
            },
        );
        x = x.saturating_add(btn_w);
    }

    if width >= 12 {
        let close_x = area.x.saturating_add((width.saturating_sub(3)) as u16);
        let style = if hover == Some(SidebarClick::ClosePanel) {
            palette.button_style(true)
        } else {
            text_style
        };
        frame.render_widget(
            Paragraph::new("[f]").style(style),
            Rect {
                x: close_x,
                y,
                width: 3,
                height: 1,
            },
        );
    }
}

fn truncate_line(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}

/// Linha divisória `├─[ bash ]───────┬───────┤` com largura exata da shell.
pub fn render_terminal_divider(
    frame: &mut Frame,
    shell: Rect,
    y: u16,
    title: &str,
    border_visible: bool,
    border_style: Style,
    title_style: Style,
) {
    let c = PanelBorder::Plain.chars();
    let w = shell.width as usize;
    if w < 4 {
        return;
    }
    let sb_start = (terminal_split_col(shell, border_visible).saturating_sub(shell.x)) as usize;
    let mut line: Vec<char> = vec![c.h; w];
    line[0] = c.sep_l;
    line[w - 1] = c.sep_r;
    if sb_start > 0 && sb_start < w {
        line[sb_start] = '┬';
    }
    for i in 1..sb_start {
        line[i] = c.h;
    }
    for i in sb_start.saturating_add(1)..w.saturating_sub(1) {
        line[i] = c.h;
    }

    let title_block = format!("─{}", panel::framed_title(title));
    let mut col = 1usize;
    for ch in title_block.chars() {
        if col >= sb_start {
            break;
        }
        line[col] = ch;
        col += 1;
    }

    let mut out_spans = Vec::new();
    let mut i = 0usize;
    while i < w {
        let ch = line[i];
        let start = i;
        while i < w && line[i] == ch {
            i += 1;
        }
        let style = if start >= 1 && start < col && ch != c.h {
            title_style
        } else {
            border_style
        };
        let run: String = line[start..i].iter().collect();
        out_spans.push(Span::styled(run, style));
    }

    frame.render_widget(
        Paragraph::new(Line::from(out_spans)),
        Rect {
            x: shell.x,
            y,
            width: shell.width,
            height: 1,
        },
    );
}

/// Fecha a moldura inferior `└──────┴──────┘`.
pub fn render_terminal_bottom_row(
    frame: &mut Frame,
    shell: Rect,
    y: u16,
    border_visible: bool,
    border_style: Style,
) {
    let c = PanelBorder::Plain.chars();
    let w = shell.width as usize;
    if w < 2 {
        return;
    }
    let sb_start = (terminal_split_col(shell, border_visible).saturating_sub(shell.x)) as usize;
    let mut line: Vec<char> = vec![c.h; w];
    line[0] = c.bl;
    line[w - 1] = c.br;
    if sb_start > 0 && sb_start < w {
        line[sb_start] = '┴';
    }
    for i in 1..sb_start {
        line[i] = c.h;
    }
    for i in sb_start.saturating_add(1)..w.saturating_sub(1) {
        line[i] = c.h;
    }
    let s: String = line.iter().collect();
    frame.render_widget(
        Paragraph::new(Span::styled(s, border_style)).style(border_style),
        Rect {
            x: shell.x,
            y,
            width: shell.width,
            height: 1,
        },
    );
}

pub const TERMINAL_PANEL_ROWS_MIN: u16 = 7;
pub const TERMINAL_PANEL_ROWS_MAX: u16 = 11;
pub const TERMINAL_PANEL_ROWS_DEFAULT: u16 = 9;

pub fn clamp_terminal_panel_rows(rows: u16) -> u16 {
    rows.clamp(TERMINAL_PANEL_ROWS_MIN, TERMINAL_PANEL_ROWS_MAX)
}

/// Linhas reservadas na base da shell quando o terminal está visível.
pub fn terminal_reserved_rows(shell: Rect, panel_rows: u16) -> u16 {
    terminal_panel_outer(shell, panel_rows).height
}

pub fn terminal_panel_outer(shell: Rect, panel_rows: u16) -> Rect {
    let h = clamp_terminal_panel_rows(panel_rows).min(shell.height.saturating_sub(4));
    Rect {
        x: shell.x,
        y: shell.y.saturating_add(shell.height.saturating_sub(h)),
        width: shell.width,
        height: h,
    }
}

pub fn editor_content_in_shell(shell: Rect, terminal_rows: u16, border_visible: bool) -> Rect {
    let top = 1u16;
    let bottom_reserve = terminal_rows;
    Rect {
        x: shell.x.saturating_add(if border_visible { 1 } else { 0 }),
        y: shell.y.saturating_add(top),
        width: shell.width.saturating_sub(if border_visible { 2 } else { 0 }),
        height: shell
            .height
            .saturating_sub(top)
            .saturating_sub(bottom_reserve),
    }
}

pub fn sidebar_cols_for_shell(shell: Rect) -> u16 {
    effective_sidebar_width(shell.width)
}

/// Linhas de texto no output (exclui divisor superior e base da moldura).
pub fn terminal_output_rows(term_outer: Rect) -> u16 {
    term_outer.height.saturating_sub(2).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::sidebar_click;
    use ratatui::layout::Rect;

    #[test]
    fn sidebar_width_constant() {
        assert_eq!(SIDEBAR_COLS, 16);
    }

    #[test]
    fn sidebar_starts_at_split_col() {
        let shell = Rect::new(0, 0, 80, 24);
        let term_outer = terminal_panel_outer(shell, TERMINAL_PANEL_ROWS_DEFAULT);
        let panel = layout_terminal_panel(shell, term_outer, true);
        let split = terminal_split_col(shell, true);
        assert_eq!(panel.output.x + panel.output.width, split);
        assert_eq!(panel.sidebar.x, split.saturating_add(1));
        assert_eq!(panel.sidebar.width, SIDEBAR_COLS);
    }

    #[test]
    fn sidebar_top_row_fits_buttons() {
        let area = Rect::new(0, 0, 16, 6);
        assert_eq!(sidebar_click(area, 0, 0, 0), Some(SidebarClick::NewSession));
        assert_eq!(sidebar_click(area, 3, 0, 0), Some(SidebarClick::GrowPanel));
        assert_eq!(sidebar_click(area, 6, 0, 0), Some(SidebarClick::ShrinkPanel));
        assert_eq!(sidebar_click(area, 13, 0, 0), Some(SidebarClick::ClosePanel));
    }

    #[test]
    fn output_rows_exclude_frame_lines() {
        let shell = Rect::new(0, 0, 80, 24);
        let term_outer = terminal_panel_outer(shell, 9);
        assert_eq!(terminal_output_rows(term_outer), 7);
    }
}
