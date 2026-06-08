//! Seleção de texto na tela do terminal (coordenadas VT100).

use crossterm::event::MouseEvent;
use ratatui::layout::Rect;

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
    _session: &TerminalSession,
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
    if rel_row >= output.height as usize {
        return None;
    }
    Some(TextCoord {
        line: rel_row,
        col: rel_col,
    })
}
