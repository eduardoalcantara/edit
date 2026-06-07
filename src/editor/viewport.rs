use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    pub top_line: usize,
    pub left_col: usize,
    pub width: u16,
    pub height: u16,
}

impl Viewport {
    pub fn update_size(&mut self, area: Rect) {
        self.width = area.width;
        self.height = area.height;
    }

    pub fn ensure_cursor_visible(&mut self, line: usize, col: usize) {
        if line < self.top_line {
            self.top_line = line;
        } else if self.height > 0 && line >= self.top_line + self.height as usize {
            self.top_line = line.saturating_sub(self.height as usize - 1);
        }

        let w = self.width as usize;
        if w == 0 {
            return;
        }
        if col < self.left_col {
            self.left_col = col;
        } else if col >= self.left_col + w {
            self.left_col = col.saturating_sub(w - 1);
        }
    }
}
