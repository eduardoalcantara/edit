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

    /// Garante que `line`/`col` ficam visíveis e limita `top_line` ao documento.
    pub fn ensure_cursor_visible(&mut self, line: usize, col: usize, line_count: usize) {
        let h = self.height as usize;
        if h > 0 {
            if line < self.top_line {
                self.top_line = line;
            } else if line >= self.top_line.saturating_add(h) {
                self.top_line = line.saturating_add(1).saturating_sub(h);
            }
            self.clamp_top_line(line_count);
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

    /// Impede `top_line` de ultrapassar o fim do documento.
    pub fn clamp_top_line(&mut self, line_count: usize) {
        if line_count == 0 {
            self.top_line = 0;
            return;
        }
        let h = self.height as usize;
        if h == 0 {
            return;
        }
        let max_top = line_count.saturating_sub(h);
        if self.top_line > max_top {
            self.top_line = max_top;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolls_up_when_cursor_above_viewport() {
        let mut vp = Viewport {
            top_line: 10,
            height: 5,
            ..Viewport::default()
        };
        vp.ensure_cursor_visible(5, 0, 100);
        assert_eq!(vp.top_line, 5);
    }

    #[test]
    fn scrolls_down_when_cursor_below_viewport() {
        let mut vp = Viewport {
            top_line: 0,
            height: 5,
            ..Viewport::default()
        };
        vp.ensure_cursor_visible(16, 0, 100);
        assert_eq!(vp.top_line, 12);
    }

    #[test]
    fn clamp_top_line_at_document_end() {
        let mut vp = Viewport {
            top_line: 50,
            height: 10,
            ..Viewport::default()
        };
        vp.clamp_top_line(55);
        assert_eq!(vp.top_line, 45);
    }

    #[test]
    fn scrolls_up_through_readme_with_terminal_height() {
        let mut vp = Viewport {
            top_line: 0,
            height: 15,
            ..Viewport::default()
        };
        let line_count = 146;
        vp.ensure_cursor_visible(72, 0, line_count);
        assert_eq!(vp.top_line, 58);
        for line in (0..72).rev() {
            vp.ensure_cursor_visible(line, 0, line_count);
        }
        assert_eq!(vp.top_line, 0);
    }
}
