use ropey::Rope;

use crate::edit_mode::EditMode;
use crate::editor::cursor::{
    char_idx_to_line_col, line_col_to_char_idx, Cursor, SelectionMode,
};
use crate::editor::history::EditHistory;
use crate::editor::selection::{
    add_cursor_at, delete_block, extract_block_text, extract_linear_text, merge_cursors,
    BlockSelectionState,
};
use crate::editor::tabs::{char_col_from_visual, line_visual_len, tab_stop_width, visual_col_in_line};
use crate::editor::word_boundary::{get_next_word_boundary, WordDirection};
use crate::editor::viewport::Viewport;
use crate::encoding::Tabulation;

/// Texto interno do editor quando o documento está vazio (`load_text("")`).
pub const EMPTY_DOCUMENT_TEXT: &str = "";

pub struct EditorEngine {
    pub text: Rope,
    pub cursors: Vec<Cursor>,
    pub block_selection: Option<BlockSelectionState>,
    pub selection_mode: SelectionMode,
    pub input_mode: EditMode,
    pub viewport: Viewport,
    pub word_wrap: bool,
    pub tabulation: Tabulation,
    pub search_pattern: String,
    history: EditHistory,
    /// Atualizado a cada frame em `render::draw` para o rodapé.
    cached_visible_chars: usize,
    cached_total_chars: usize,
}

impl Default for EditorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorEngine {
    pub fn new() -> Self {
        Self {
            text: Rope::from_str(EMPTY_DOCUMENT_TEXT),
            cursors: vec![Cursor::new(0)],
            block_selection: None,
            selection_mode: SelectionMode::Normal,
            input_mode: EditMode::Insert,
            viewport: Viewport::default(),
            word_wrap: false,
            tabulation: Tabulation::default(),
            search_pattern: String::new(),
            history: EditHistory::new(),
            cached_visible_chars: 0,
            cached_total_chars: 0,
        }
    }

    pub fn primary(&self) -> &Cursor {
        &self.cursors[0]
    }

    pub fn primary_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[0]
    }

    pub fn load_text(&mut self, content: &str) {
        self.text = Rope::from_str(content);
        self.cursors = vec![Cursor::new(0)];
        self.block_selection = None;
        self.selection_mode = SelectionMode::Normal;
        self.history.clear();
        self.viewport.top_line = 0;
        self.viewport.left_col = 0;
        self.sync_primary_virtual();
    }

    pub fn load_lines(&mut self, lines: &[String]) {
        self.load_text(&lines.join("\n"));
    }

    pub fn to_lines(&self) -> Vec<String> {
        if self.text.len_lines() == 0 {
            return vec![String::new()];
        }
        (0..self.text.len_lines())
            .map(|i| self.text.line(i).to_string())
            .collect()
    }

    pub fn content_string(&self) -> String {
        self.text.to_string()
    }

    pub fn byte_size(&self) -> usize {
        self.text.len_bytes()
    }

    /// Tamanho total do arquivo em caracteres (inclui `\n` e demais codepoints).
    pub fn total_char_count(&self) -> usize {
        self.text.len_chars()
    }

    /// Caracteres das linhas presentes no viewport vertical (conteúdo completo de cada linha, sem `\n`).
    pub fn visible_char_count(&self) -> usize {
        self.count_visible_chars(
            self.viewport.top_line,
            self.viewport.height as usize,
        )
    }

    pub fn footer_visible_chars(&self) -> usize {
        self.cached_visible_chars
    }

    pub fn footer_total_chars(&self) -> usize {
        self.cached_total_chars
    }

    /// Soma o comprimento visual das linhas visíveis verticalmente (ignora recorte horizontal).
    fn count_visible_chars(&self, top_line: usize, visible_h: usize) -> usize {
        if visible_h == 0 {
            return 0;
        }

        let line_count = self.text.len_lines();
        let mut count = 0usize;
        for row in 0..visible_h {
            let doc_line = top_line + row;
            if doc_line >= line_count {
                break;
            }
            count += self.line_display_len(doc_line);
        }
        count
    }

    fn line_display_len(&self, line_idx: usize) -> usize {
        let line = self.text.line(line_idx).to_string();
        let line = line.trim_end_matches('\n');
        line_visual_len(line, tab_stop_width(self.tabulation))
    }

    pub fn refresh_footer_size_stats(
        &mut self,
        top_line: usize,
        visible_h: usize,
    ) {
        self.cached_total_chars = self.total_char_count();
        self.cached_visible_chars = self.count_visible_chars(top_line, visible_h);
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let (line, col) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        (line + 1, col + 1)
    }

    pub fn cursor_raw(&self) -> (usize, usize) {
        char_idx_to_line_col(&self.text, self.primary().char_idx)
    }

    pub fn set_cursor_line_col(&mut self, line: usize, col: usize) {
        self.set_caret_line_col(line, col, true);
    }

    pub fn set_caret_line_col(&mut self, line: usize, col: usize, clear_selection: bool) {
        let line = line.min(self.text.len_lines().saturating_sub(1));
        let idx = line_col_to_char_idx(&self.text, line, col);
        self.primary_mut().char_idx = idx;
        self.sync_primary_virtual();
        if clear_selection {
            self.primary_mut().anchor = None;
            self.block_selection = None;
            self.selection_mode = SelectionMode::Normal;
            self.cursors.truncate(1);
        }
        self.ensure_visible();
    }

    pub fn extend_selection_to(&mut self, line: usize, col: usize) {
        if self.primary().anchor.is_none() {
            self.primary_mut().anchor = Some(self.primary().char_idx);
        }
        self.selection_mode = SelectionMode::Normal;
        self.block_selection = None;
        self.cursors.truncate(1);
        let idx = line_col_to_char_idx(&self.text, line, col);
        self.primary_mut().char_idx = idx;
        self.primary_mut().virtual_col = col;
        self.ensure_visible();
    }

    fn prepare_move(&mut self, extend: bool) {
        if extend {
            if self.primary().anchor.is_none() {
                self.primary_mut().anchor = Some(self.primary().char_idx);
            }
            self.selection_mode = SelectionMode::Normal;
            self.block_selection = None;
            self.cursors.truncate(1);
        } else if self.selection_mode == SelectionMode::Normal {
            self.primary_mut().anchor = None;
        }
    }

    fn sync_primary_virtual(&mut self) {
        let idx = self.cursors[0].char_idx;
        let (line, col) = char_idx_to_line_col(&self.text, idx);
        let line_str = self.text.line(line).to_string();
        self.cursors[0].virtual_col =
            visual_col_in_line(&line_str, col, tab_stop_width(self.tabulation));
    }

    pub fn ensure_visible(&mut self) {
        let (line, col) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let line_str = self.text.line(line).to_string();
        let vis_col = visual_col_in_line(&line_str, col, tab_stop_width(self.tabulation));
        self.viewport.ensure_cursor_visible(line, vis_col);
    }

    fn record(&mut self, start: usize, removed: String, inserted: String, before: usize, after: usize) {
        self.history
            .record_change(start, removed, inserted, before, after);
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.selection_mode == SelectionMode::Multi {
            self.insert_char_multi(ch);
            return;
        }
        self.delete_selection();
        let before = self.primary().char_idx;
        if self.input_mode == EditMode::Replace && ch != '\n' {
            let idx = before;
            if idx < self.text.len_chars() && self.text.get_char(idx) == Some('\n') {
                self.text.insert_char(idx, ch);
                self.primary_mut().char_idx = idx + 1;
                self.sync_primary_virtual();
                self.record(idx, String::new(), ch.to_string(), before, idx + 1);
                self.ensure_visible();
                return;
            }
            self.replace_char_at(before, ch);
            return;
        }
        self.materialize_virtual_at_primary();
        let idx = self.primary().char_idx.min(self.text.len_chars());
        self.text.insert_char(idx, ch);
        self.primary_mut().char_idx = idx + 1;
        self.sync_primary_virtual();
        self.record(idx, String::new(), ch.to_string(), before, self.primary().char_idx);
        self.ensure_visible();
    }

    fn replace_char_at(&mut self, idx: usize, ch: char) {
        let before = idx;
        if idx < self.text.len_chars() {
            let removed = self.text.slice(idx..idx + 1).to_string();
            self.text.remove(idx..idx + 1);
            self.text.insert_char(idx, ch);
            self.primary_mut().char_idx = idx + 1;
            self.sync_primary_virtual();
            self.record(idx, removed, ch.to_string(), before, idx + 1);
        } else {
            self.text.insert_char(idx, ch);
            self.primary_mut().char_idx = idx + 1;
            self.sync_primary_virtual();
            self.record(idx, String::new(), ch.to_string(), before, idx + 1);
        }
        self.ensure_visible();
    }

    fn insert_char_multi(&mut self, ch: char) {
        let mut indices: Vec<usize> = self.cursors.iter().map(|c| c.char_idx).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        let before_positions: Vec<usize> = indices.clone();
        for idx in indices {
            self.materialize_at(idx);
            self.text.insert_char(idx, ch);
        }
        for c in &mut self.cursors {
            if before_positions.contains(&c.char_idx) {
                c.char_idx += 1;
            }
        }
        merge_cursors(&mut self.cursors);
        if let Some(first) = before_positions.first() {
            self.record(*first, String::new(), ch.to_string(), *first, first + 1);
        }
        self.ensure_visible();
    }

    pub fn insert_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.insert_char(ch);
        }
    }

    fn materialize_at(&mut self, char_idx: usize) {
        let (line, col) = char_idx_to_line_col(&self.text, char_idx.min(self.text.len_chars()));
        let line_start = self.text.line_to_char(line);
        let line_len = self.text.line(line).len_chars();
        if col > line_len {
            let spaces: String = " ".repeat(col - line_len);
            self.text.insert(line_start + line_len, &spaces);
        }
    }

    fn materialize_virtual_at_primary(&mut self) {
        let virtual_col = self.primary().virtual_col;
        let char_idx = self.primary().char_idx.min(self.text.len_chars());
        let (line, col) = char_idx_to_line_col(&self.text, char_idx);
        let line_start = self.text.line_to_char(line);
        let line_str = self.text.line(line).to_string();
        let tab_width = tab_stop_width(self.tabulation);
        let line_visual = visual_col_in_line(&line_str, col, tab_width);
        let line_char_len = self.text.line(line).len_chars();

        if virtual_col <= line_visual {
            return;
        }
        if line_char_len == 0 {
            self.primary_mut().virtual_col = 0;
            return;
        }
        // Caret clamped on a shorter line after ↑/↓ — não preencher com espaços.
        if col < line_char_len {
            self.primary_mut().virtual_col = line_visual;
            return;
        }
        // Só materializa espaços quando o caret está no fim da linha e virtual_col avançou além.
        if virtual_col > line_visual {
            let spaces: String = " ".repeat(virtual_col - line_visual);
            self.text.insert(line_start + line_char_len, &spaces);
            self.primary_mut().char_idx = line_start + line_char_len + spaces.chars().count();
        }
    }

    pub fn backspace(&mut self) {
        if self.selection_mode == SelectionMode::Multi {
            for idx in self.cursors.iter().map(|c| c.char_idx).collect::<Vec<_>>() {
                if idx > 0 {
                    self.text.remove(idx - 1..idx);
                }
            }
            for c in &mut self.cursors {
                if c.char_idx > 0 {
                    c.char_idx -= 1;
                }
            }
            merge_cursors(&mut self.cursors);
            self.ensure_visible();
            return;
        }
        if self.delete_selection() {
            return;
        }
        let idx = self.primary().char_idx;
        if idx == 0 {
            return;
        }
        let before = idx;
        let removed = self.text.slice(idx - 1..idx).to_string();
        self.text.remove(idx - 1..idx);
        self.primary_mut().char_idx = idx - 1;
        self.sync_primary_virtual();
        self.record(idx - 1, removed, String::new(), before, idx - 1);
        self.ensure_visible();
    }

    pub fn delete(&mut self) {
        if self.selection_mode == SelectionMode::Multi {
            let indices: Vec<usize> = self.cursors.iter().map(|c| c.char_idx).collect();
            for idx in indices.into_iter().rev() {
                if idx < self.text.len_chars() {
                    self.text.remove(idx..idx + 1);
                }
            }
            merge_cursors(&mut self.cursors);
            self.ensure_visible();
            return;
        }
        if self.delete_selection() {
            return;
        }
        let idx = self.primary().char_idx;
        if idx >= self.text.len_chars() {
            return;
        }
        let removed = self.text.slice(idx..idx + 1).to_string();
        self.text.remove(idx..idx + 1);
        self.sync_primary_virtual();
        self.record(idx, removed, String::new(), idx, idx);
        self.ensure_visible();
    }

    pub fn move_left(&mut self, extend: bool) {
        self.prepare_move(extend);
        if self.primary().char_idx > 0 {
            self.primary_mut().char_idx -= 1;
            self.sync_primary_virtual();
            self.ensure_visible();
        }
    }

    pub fn move_right(&mut self, extend: bool) {
        self.prepare_move(extend);
        if self.primary().char_idx < self.text.len_chars() {
            self.primary_mut().char_idx += 1;
            self.sync_primary_virtual();
            self.ensure_visible();
        }
    }

    pub fn move_word_left(&mut self, extend: bool) {
        self.prepare_move(extend);
        let pos = self.primary().char_idx;
        let next = get_next_word_boundary(&self.text, pos, WordDirection::Left);
        self.primary_mut().char_idx = next;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn move_word_right(&mut self, extend: bool) {
        self.prepare_move(extend);
        let pos = self.primary().char_idx;
        let next = get_next_word_boundary(&self.text, pos, WordDirection::Right);
        self.primary_mut().char_idx = next;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn move_up(&mut self, extend: bool) {
        self.prepare_move(extend);
        let vcol = self.primary().virtual_col;
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        if line == 0 {
            return;
        }
        let target_line = line - 1;
        let line_str = self.text.line(target_line).to_string();
        let char_col = char_col_from_visual(&line_str, vcol, tab_stop_width(self.tabulation));
        let idx = line_col_to_char_idx(&self.text, target_line, char_col);
        self.primary_mut().char_idx = idx;
        self.primary_mut().virtual_col = vcol;
        self.ensure_visible();
    }

    pub fn move_down(&mut self, extend: bool) {
        self.prepare_move(extend);
        let vcol = self.primary().virtual_col;
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        if line + 1 >= self.text.len_lines() {
            return;
        }
        let target_line = line + 1;
        let line_str = self.text.line(target_line).to_string();
        let char_col = char_col_from_visual(&line_str, vcol, tab_stop_width(self.tabulation));
        let idx = line_col_to_char_idx(&self.text, target_line, char_col);
        self.primary_mut().char_idx = idx;
        self.primary_mut().virtual_col = vcol;
        self.ensure_visible();
    }

    pub fn move_home(&mut self, extend: bool) {
        self.prepare_move(extend);
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        self.primary_mut().char_idx = self.text.line_to_char(line);
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn move_end(&mut self, extend: bool) {
        self.prepare_move(extend);
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let line_start = self.text.line_to_char(line);
        let line_len = self.text.line(line).len_chars();
        self.primary_mut().char_idx = line_start + line_len;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn select_all(&mut self) {
        self.primary_mut().anchor = Some(0);
        self.primary_mut().char_idx = self.text.len_chars();
        self.selection_mode = SelectionMode::Normal;
        self.sync_primary_virtual();
    }

    pub fn cancel_selection(&mut self) {
        self.primary_mut().anchor = None;
        self.block_selection = None;
        self.selection_mode = SelectionMode::Normal;
        self.cursors.truncate(1);
    }

    pub fn start_block_select(&mut self, line: usize, col: usize) {
        self.selection_mode = SelectionMode::Block;
        self.block_selection = Some(BlockSelectionState {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
            dragging: true,
        });
        self.primary_mut().anchor = None;
    }

    pub fn update_block_select(&mut self, line: usize, col: usize) {
        if let Some(block) = &mut self.block_selection {
            block.end_line = line;
            block.end_col = col;
        }
    }

    pub fn finish_block_select(&mut self) {
        if let Some(block) = &mut self.block_selection {
            block.dragging = false;
            let (r0, vc0, r1, vc1) = block.normalized();
            if r0 == r1 && vc0 == vc1 {
                self.block_selection = None;
                self.selection_mode = SelectionMode::Normal;
                return;
            }
            self.cursors.truncate(1);
            let line_str = self.text.line(r0).to_string();
            let char_col = char_col_from_visual(
                line_str.trim_end_matches('\n'),
                vc0,
                tab_stop_width(self.tabulation),
            );
            self.primary_mut().char_idx = line_col_to_char_idx(&self.text, r0, char_col);
            self.primary_mut().anchor = None;
            self.sync_primary_virtual();
        }
    }

    pub fn is_block_dragging(&self) -> bool {
        self.block_selection.is_some_and(|b| b.dragging)
    }

    pub fn add_cursor(&mut self, line: usize, col: usize) {
        add_cursor_at(&self.text, &mut self.cursors, line, col);
        self.selection_mode = SelectionMode::Multi;
    }

    pub fn copy_text(&self) -> Option<String> {
        if let Some(block) = self.block_selection {
            let (r0, vc0, r1, vc1) = block.normalized();
            if r0 != r1 || vc0 != vc1 {
                let tw = tab_stop_width(self.tabulation);
                return Some(extract_block_text(&self.text, r0, vc0, r1, vc1, tw));
            }
        }
        if let Some(anchor) = self.primary().anchor {
            let caret = self.primary().char_idx;
            if anchor != caret {
                return Some(extract_linear_text(&self.text, anchor, caret));
            }
        }
        None
    }

    fn delete_selection(&mut self) -> bool {
        match self.selection_mode {
            SelectionMode::Block => {
                if let Some(block) = self.block_selection {
                    delete_block(&mut self.text, &block, tab_stop_width(self.tabulation));
                    self.block_selection = None;
                    self.selection_mode = SelectionMode::Normal;
                    self.cursors.truncate(1);
                    self.primary_mut().char_idx = self.primary().char_idx.min(self.text.len_chars());
                    return true;
                }
            }
            SelectionMode::Normal => {
                if let Some(anchor) = self.primary().anchor {
                    let (a, b) = if anchor <= self.primary().char_idx {
                        (anchor, self.primary().char_idx)
                    } else {
                        (self.primary().char_idx, anchor)
                    };
                    if a != b {
                        self.text.remove(a..b);
                        self.primary_mut().char_idx = a;
                        self.primary_mut().anchor = None;
                        self.sync_primary_virtual();
                        return true;
                    }
                }
            }
            SelectionMode::Multi => {}
        }
        false
    }

    pub fn cut_selection(&mut self) -> Option<String> {
        let text = self.copy_text()?;
        if !text.is_empty() {
            let _ = self.delete_selection();
        }
        Some(text)
    }

    pub fn undo(&mut self) {
        let idx = self.primary().char_idx;
        if self.history.undo(&mut self.text, &mut self.cursors[0].char_idx) {
            self.sync_primary_virtual();
            self.ensure_visible();
        } else {
            self.primary_mut().char_idx = idx;
        }
    }

    pub fn redo(&mut self) {
        if self.history.redo(&mut self.text, &mut self.cursors[0].char_idx) {
            self.sync_primary_virtual();
            self.ensure_visible();
        }
    }

    pub fn find_next(&mut self) -> bool {
        if self.search_pattern.is_empty() {
            return false;
        }
        if let Some(idx) =
            crate::editor::search::find_next(&self.text, &self.search_pattern, self.primary().char_idx + 1)
        {
            self.primary_mut().char_idx = idx;
            self.sync_primary_virtual();
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    pub fn find_prev(&mut self) -> bool {
        if self.search_pattern.is_empty() {
            return false;
        }
        if let Some(idx) =
            crate::editor::search::find_prev(&self.text, &self.search_pattern, self.primary().char_idx)
        {
            self.primary_mut().char_idx = idx;
            self.sync_primary_virtual();
            self.ensure_visible();
            true
        } else {
            false
        }
    }

    pub fn replace_one(&mut self, replacement: &str) -> bool {
        let pattern = self.search_pattern.clone();
        let idx = self.primary().char_idx;
        if crate::editor::search::replace_at(&mut self.text, idx, &pattern, replacement) {
            self.primary_mut().char_idx = idx;
            self.sync_primary_virtual();
            true
        } else {
            false
        }
    }

    pub fn insert_tab(&mut self) {
        let text = self.tabulation.insert_text();
        self.insert_str(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::Tabulation;

    #[test]
    fn insert_and_overtype_advances() {
        let mut e = EditorEngine::new();
        e.load_text("abc");
        e.set_cursor_line_col(0, 1);
        e.input_mode = EditMode::Replace;
        e.insert_char('X');
        assert_eq!(e.text.to_string(), "aXc");
        assert_eq!(e.primary().char_idx, 2);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut e = EditorEngine::new();
        e.load_text("hello");
        e.set_cursor_line_col(0, 5);
        e.insert_char('\n');
        assert_eq!(e.text.to_string(), "hello\n");
        assert_eq!(e.text.len_lines(), 2);
        let (line, col) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn replace_on_newline_preserves_line_break() {
        let mut e = EditorEngine::new();
        e.load_text("hello\nworld");
        e.input_mode = EditMode::Replace;
        e.set_cursor_line_col(0, 5);
        assert_eq!(e.text.get_char(5), Some('\n'));
        e.insert_char('X');
        assert_eq!(e.text.to_string(), "helloX\nworld");
        assert_eq!(e.text.len_lines(), 2);
    }

    #[test]
    fn copy_text_linear_selection() {
        let mut e = EditorEngine::new();
        e.load_text("hello world");
        e.set_cursor_line_col(0, 0);
        e.move_right(true);
        e.move_right(true);
        e.move_right(true);
        e.move_right(true);
        e.move_right(true);
        let text = e.copy_text().unwrap();
        assert_eq!(text, "hello");
    }

    #[test]
    fn empty_document_typing_stays_on_first_line() {
        let mut e = EditorEngine::new();
        e.insert_char('a');
        e.insert_char('b');
        e.insert_char('c');
        assert_eq!(e.text.to_string(), "abc");
        let (line, col) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 0);
        assert_eq!(col, 3);
    }

    #[test]
    fn vertical_move_to_empty_line_does_not_pad_on_type() {
        let mut e = EditorEngine::new();
        e.load_text("abcdefghijklmnopqrstuvwxyz\n");
        e.set_cursor_line_col(0, 20);
        e.move_down(false);
        e.insert_char('x');
        assert_eq!(e.text.to_string(), "abcdefghijklmnopqrstuvwxyz\nx");
    }

    #[test]
    fn load_text_resets_viewport() {
        let mut e = EditorEngine::new();
        e.load_text("line0\nline1\nline2\nline3");
        e.set_cursor_line_col(3, 0);
        e.load_text("");
        assert_eq!(e.viewport.top_line, 0);
        assert_eq!(e.viewport.left_col, 0);
        assert_eq!(e.primary().char_idx, 0);
    }

    #[test]
    fn total_includes_newlines_visible_does_not() {
        let mut e = EditorEngine::new();
        e.load_text("hello\nworld\ntest");
        e.viewport.width = 80;
        e.viewport.height = 24;
        assert_eq!(e.text.len_lines(), 3);
        assert_eq!(e.line_display_len(0), 5);
        assert_eq!(e.line_display_len(1), 5);
        assert_eq!(e.line_display_len(2), 4);
        assert_eq!(e.total_char_count(), 16);
        assert_eq!(e.visible_char_count(), 14);
        assert_ne!(e.total_char_count(), e.visible_char_count());
    }

    #[test]
    fn visible_char_count_grows_on_long_single_line() {
        let mut e = EditorEngine::new();
        let line = "x".repeat(553);
        e.load_text(&line);
        e.viewport.width = 153;
        e.viewport.height = 24;
        e.viewport.left_col = 400;
        assert_eq!(e.total_char_count(), 553);
        assert_eq!(e.visible_char_count(), 553);
    }

    #[test]
    fn visible_char_count_respects_vertical_viewport() {
        let mut e = EditorEngine::new();
        e.load_text("a\nb\nc\nd\ne");
        e.viewport.height = 2;
        e.viewport.top_line = 0;
        assert_eq!(e.visible_char_count(), 2);
        e.viewport.top_line = 2;
        assert_eq!(e.visible_char_count(), 2);
    }

    #[test]
    fn visible_char_count_respects_viewport() {
        let mut e = EditorEngine::new();
        e.load_text("abcdefghij");
        e.viewport.width = 4;
        e.viewport.height = 1;
        e.viewport.top_line = 0;
        e.viewport.left_col = 0;
        assert_eq!(e.total_char_count(), 10);
        assert_eq!(e.visible_char_count(), 10);
        e.viewport.left_col = 6;
        assert_eq!(e.visible_char_count(), 10);
    }

    #[test]
    fn copy_text_block_after_release() {
        let mut e = EditorEngine::new();
        e.load_text("ab\nxy");
        e.start_block_select(0, 0);
        e.update_block_select(1, 1);
        e.finish_block_select();
        assert_eq!(e.selection_mode, SelectionMode::Block);
        assert!(!e.block_selection.unwrap().dragging);
        let text = e.copy_text().unwrap();
        assert_eq!(text, "a\nx");
    }

    #[test]
    fn literal_tab_advances_visual_column() {
        let mut e = EditorEngine::new();
        e.tabulation = Tabulation::TabLiteral;
        e.load_text("testesdd");
        e.set_cursor_line_col(0, 8);
        e.insert_tab();
        assert_eq!(e.text.to_string(), "testesdd\t");
        let (_, col) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(col, 9);
        let line = e.text.line(0).to_string();
        assert_eq!(
            visual_col_in_line(&line, col, tab_stop_width(Tabulation::TabLiteral)),
            16
        );
    }
}
