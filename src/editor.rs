use ratatui::widgets::Block;
use tui_textarea::{CursorMove, Input, TextArea, WrapMode};

use crate::block_select::extract_block_text;
use crate::clipboard::Clipboard;
use crate::cursors::{CursorManager, CursorMode, CursorPos};
use crate::edit_mode::EditMode;
use crate::encoding::Tabulation;
use crate::theme::ThemePalette;

pub struct Editor {
    textarea: TextArea<'static>,
    mode: EditMode,
    cursors: CursorManager,
    tabulation: Tabulation,
}

impl Editor {
    pub fn new(palette: &ThemePalette) -> Self {
        let mut editor = Self {
            textarea: TextArea::default(),
            mode: EditMode::Insert,
            cursors: CursorManager::default(),
            tabulation: Tabulation::default(),
        };
        editor
            .textarea
            .set_placeholder_text("Digite aqui para começar...");
        editor.apply_theme(palette);
        editor
    }

    pub fn apply_theme(&mut self, palette: &ThemePalette) {
        self.textarea.set_style(palette.editor_text_style());
        self.textarea.set_cursor_style(palette.cursor_style_for_mode(self.mode));
        self.textarea.set_selection_style(palette.selection_style());
        self.textarea
            .set_placeholder_style(palette.placeholder_style());
        self.set_window_title("Sem título", palette);
    }

    pub fn set_window_title(&mut self, title: &str, palette: &ThemePalette) {
        let (line, col) = self.cursor_line_col();
        let frame_title = format!(
            " {title} │ Ln {line} Col {col} │ {} ",
            self.mode.label()
        );
        self.textarea.set_block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(
                    ratatui::style::Style::default()
                        .fg(palette.border)
                        .bg(palette.editor_bg),
                )
                .title(frame_title)
                .style(palette.editor_text_style()),
        );
    }

    pub fn set_word_wrap(&mut self, on: bool) {
        if on {
            self.textarea.set_wrap_mode(WrapMode::WordOrGlyph);
        } else {
            self.textarea.set_wrap_mode(WrapMode::None);
        }
    }

    pub fn set_tabulation(&mut self, tab: Tabulation) {
        self.tabulation = tab;
    }

    pub fn tabulation(&self) -> Tabulation {
        self.tabulation
    }

    pub fn cursors(&self) -> &CursorManager {
        &self.cursors
    }

    pub fn cursors_mut(&mut self) -> &mut CursorManager {
        &mut self.cursors
    }

    pub fn mode(&self) -> EditMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: EditMode, palette: &ThemePalette) {
        self.mode = mode;
        self.apply_theme(palette);
    }

    pub fn toggle_mode(&mut self, palette: &ThemePalette) {
        self.set_mode(self.mode.toggle(), palette);
    }

    pub fn handle_input(&mut self, input: Input) {
        if self.cursors.mode == CursorMode::Multi {
            self.handle_multi_input(input);
            return;
        }

        if matches!(input.key, tui_textarea::Key::Tab) && !input.ctrl && !input.alt {
            let text = self.tabulation.insert_text();
            for ch in text.chars() {
                self.textarea.input(Input {
                    key: tui_textarea::Key::Char(ch),
                    ctrl: false,
                    alt: false,
                    shift: false,
                });
            }
            self.sync_cursors_from_textarea();
            return;
        }

        if self.mode == EditMode::Replace {
            let advance = matches!(
                input.key,
                tui_textarea::Key::Char(ch) if ch != '\n' && ch != '\t'
            );
            if let Input {
                key: tui_textarea::Key::Char(ch),
                ctrl: false,
                alt: false,
                shift: false,
            } = &input
            {
                if *ch != '\n' && *ch != '\t' {
                    self.textarea.delete_char();
                }
            }
            self.textarea.input(input);
            if advance {
                self.textarea.move_cursor(CursorMove::Forward);
            }
            self.sync_cursors_from_textarea();
            return;
        }
    }

    fn handle_multi_input(&mut self, input: Input) {
        match input.key {
            tui_textarea::Key::Esc => {
                self.cursors.cancel_to_normal();
                return;
            }
            tui_textarea::Key::Char(ch) if !input.ctrl && !input.alt => {
                self.insert_at_all_cursors(&ch.to_string());
            }
            tui_textarea::Key::Backspace => {
                for pos in self.cursors.cursors.clone() {
                    self.delete_at(pos, true);
                }
                self.cursors.merge_colliding();
            }
            tui_textarea::Key::Delete => {
                for pos in self.cursors.cursors.clone() {
                    self.delete_at(pos, false);
                }
                self.cursors.merge_colliding();
            }
            _ => {}
        }
        self.sync_cursors_from_textarea();
    }

    fn insert_at_all_cursors(&mut self, text: &str) {
        let mut positions = self.cursors.cursors.clone();
        positions.sort_by(|a, b| b.row.cmp(&a.row).then(b.col.cmp(&a.col)));
        let mut lines = self.lines();
        for pos in positions {
            materialize_virtual_space(&mut lines, pos.row, pos.col);
            if let Some(line) = lines.get_mut(pos.row) {
                if pos.col <= line.len() {
                    line.insert_str(pos.col, text);
                } else {
                    line.push_str(&" ".repeat(pos.col - line.len()));
                    line.push_str(text);
                }
            }
        }
        self.set_lines(lines);
        for c in &mut self.cursors.cursors {
            c.col += text.chars().count();
        }
        self.cursors.primary.col += text.chars().count();
    }

    fn delete_at(&mut self, pos: CursorPos, backspace: bool) {
        let mut lines = self.lines();
        let Some(line) = lines.get_mut(pos.row) else {
            return;
        };
        if backspace && pos.col > 0 && pos.col <= line.len() {
            line.remove(pos.col - 1);
        } else if !backspace && pos.col < line.len() {
            line.remove(pos.col);
        }
        self.set_lines(lines);
    }

    fn sync_cursors_from_textarea(&mut self) {
        let (r, c) = self.textarea.cursor();
        self.cursors.sync_primary(r, c);
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.textarea
            .move_cursor(CursorMove::Jump(row as u16, col as u16));
        self.cursors.sync_primary(row, col);
    }

    pub fn lines(&self) -> Vec<String> {
        self.textarea.lines().to_vec()
    }

    pub fn set_lines(&mut self, lines: Vec<String>) {
        let (r, c) = self.textarea.cursor();
        self.textarea.set_lines(lines, (r, c));
    }

    pub fn clear(&mut self) {
        self.textarea.clear();
        self.cursors = CursorManager::default();
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let cursor = self.textarea.cursor();
        (cursor.0 + 1, cursor.1 + 1)
    }

    pub fn cursor_raw(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    pub fn selection_label(&self) -> String {
        if self.cursors.mode == CursorMode::Block {
            if let Some((r0, c0, r1, c1)) = self.cursors.block_range() {
                return format!("bloco ({r0},{c0})-({r1},{c1})");
            }
        }
        if self.cursors.mode == CursorMode::Multi {
            return format!("multi:{}", self.cursors.cursors.len());
        }
        if !self.textarea.is_selecting() {
            return "0".to_string();
        }
        match self.textarea.selection_range() {
            Some(((r1, c1), (r2, c2))) => format!("({r1},{c1})-({r2},{c2})"),
            None => "0".to_string(),
        }
    }

    pub fn byte_size(&self) -> usize {
        self.lines().join("\n").len()
    }

    pub fn undo(&mut self) {
        self.textarea.undo();
        self.sync_cursors_from_textarea();
    }

    pub fn redo(&mut self) {
        self.textarea.redo();
        self.sync_cursors_from_textarea();
    }

    pub fn select_all(&mut self) {
        self.textarea.select_all();
        self.cursors.cancel_to_normal();
    }

    pub fn cancel_selection(&mut self) {
        self.textarea.cancel_selection();
        self.cursors.cancel_to_normal();
    }

    pub fn copy_selection(&mut self, clipboard: &mut Clipboard) -> bool {
        if self.cursors.mode == CursorMode::Block {
            if let Some((r0, c0, r1, c1)) = self.cursors.block_range() {
                let text = extract_block_text(&self.lines(), r0, c0, r1, c1);
                clipboard.push(text);
                return true;
            }
        }
        if let Some(((r1, c1), (r2, c2))) = self.textarea.selection_range() {
            let lines = self.lines();
            let text = extract_range(&lines, r1, c1, r2, c2);
            clipboard.push(text);
            return true;
        }
        false
    }

    pub fn cut_selection(&mut self, clipboard: &mut Clipboard) -> bool {
        if !self.copy_selection(clipboard) {
            return false;
        }
        if self.cursors.mode == CursorMode::Block {
            self.delete_block_selection();
        } else if self.textarea.is_selecting() {
            self.textarea.delete_char();
            self.textarea.delete_word();
            self.textarea.cancel_selection();
        }
        true
    }

    pub fn paste(&mut self, text: &str) {
        for ch in text.chars() {
            self.textarea.input(Input {
                key: tui_textarea::Key::Char(ch),
                ctrl: false,
                alt: false,
                shift: false,
            });
        }
        self.sync_cursors_from_textarea();
    }

    fn delete_block_selection(&mut self) {
        if let Some((r0, c0, r1, c1)) = self.cursors.block_range() {
            let mut lines = self.lines();
            for row in r0..=r1.min(lines.len().saturating_sub(1)) {
                if let Some(line) = lines.get_mut(row) {
                    let end = c1.min(line.len());
                    let start = c0.min(end);
                    line.replace_range(start..end, "");
                }
            }
            self.set_lines(lines);
            self.cursors.cancel_to_normal();
        }
    }

    pub fn textarea(&self) -> &TextArea<'static> {
        &self.textarea
    }
}

fn materialize_virtual_space(lines: &mut [String], row: usize, col: usize) {
    if let Some(line) = lines.get_mut(row) {
        if col > line.len() {
            line.push_str(&" ".repeat(col - line.len()));
        }
    }
}

fn extract_range(lines: &[String], r1: usize, c1: usize, r2: usize, c2: usize) -> String {
    if r1 == r2 {
        return lines
            .get(r1)
            .map(|l| l[c1.min(l.len())..c2.min(l.len())].to_string())
            .unwrap_or_default();
    }
    let mut out = Vec::new();
    for row in r1..=r2 {
        if let Some(line) = lines.get(row) {
            if row == r1 {
                out.push(line[c1.min(line.len())..].to_string());
            } else if row == r2 {
                out.push(line[..c2.min(line.len())].to_string());
            } else {
                out.push(line.clone());
            }
        }
    }
    out.join("\n")
}
