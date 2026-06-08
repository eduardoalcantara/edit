//! Seleção de texto no scrollback do terminal.

use crossterm::event::MouseEvent;
use ratatui::layout::Rect;

use super::scrollback::Scrollback;
use super::session::TerminalSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextCoord {
    pub line: usize,
    pub col: usize,
}

impl TextCoord {
    pub fn normalize(self, other: Self) -> (Self, Self) {
        if (self.line, self.col) <= (other.line, other.col) {
            (self, other)
        } else {
            (other, self)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSelection {
    pub anchor: TextCoord,
    pub cursor: TextCoord,
}

impl TerminalSelection {
    pub fn normalized(&self) -> (TextCoord, TextCoord) {
        self.anchor.normalize(self.cursor)
    }

    pub fn contains(&self, line: usize, col: usize) -> bool {
        let (start, end) = self.normalized();
        if line < start.line || line > end.line {
            return false;
        }
        if line == start.line && line == end.line {
            return col >= start.col && col < end.col;
        }
        if line == start.line {
            return col >= start.col;
        }
        if line == end.line {
            return col < end.col;
        }
        true
    }
}

pub fn mouse_to_coord(
    session: &TerminalSession,
    output: Rect,
    mouse: &MouseEvent,
) -> Option<TextCoord> {
    if output.width == 0 || output.height == 0 {
        return None;
    }
    if mouse.column < output.x
        || mouse.column >= output.x.saturating_add(output.width)
        || mouse.row < output.y
        || mouse.row >= output.y.saturating_add(output.height)
    {
        return None;
    }
    let rel_row = mouse.row.saturating_sub(output.y) as usize;
    let rel_col = mouse.column.saturating_sub(output.x) as usize;
    let h = output.height as usize;
    let indexed = session.scrollback.visible_tail_indexed(
        h,
        session.scroll_offset,
        session.follow_tail,
    );
    let (global_line, line_text) = indexed.get(rel_row)?;
    let col = rel_col.min(line_text.chars().count());
    Some(TextCoord {
        line: *global_line,
        col,
    })
}

pub fn extract_selection(scrollback: &Scrollback, selection: TerminalSelection) -> String {
    let (a, b) = selection.normalized();
    let logical = scrollback.logical_lines();
    let mut out = String::new();

    for line_idx in a.line..=b.line {
        let Some(text) = logical.get(line_idx) else {
            continue;
        };
        let char_len = text.chars().count();
        let from = if line_idx == a.line { a.col } else { 0 };
        let to = if line_idx == b.line {
            b.col.min(char_len)
        } else {
            char_len
        };
        if line_idx > a.line {
            out.push('\n');
        }
        out.extend(text.chars().skip(from).take(to.saturating_sub(from)));
    }
    out
}
