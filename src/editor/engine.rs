use ropey::Rope;

use crate::edit_mode::EditMode;
use crate::editor::cursor::{
    char_idx_to_line_col, line_col_to_char_idx, line_content_len, Cursor, SelectionMode,
};
use crate::editor::history::EditHistory;
use crate::editor::selection::{
    add_cursor_at, collect_block_delete_patches, delete_block, extract_block_text,
    extract_linear_text, merge_cursors, BlockSelectionState,
};
use crate::editor::tabs::{char_col_from_visual, tab_stop_width, visual_col_in_line};
use crate::editor::wrap;
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
    pub render_show_tabs: bool,
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
            render_show_tabs: false,
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
        self.viewport.top_visual_row = 0;
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
            count += self.line_logical_len(doc_line);
        }
        count
    }

    fn line_logical_len(&self, line_idx: usize) -> usize {
        let line = self.text.line(line_idx).to_string();
        line.trim_end_matches('\n').chars().count()
    }

    pub fn refresh_footer_size_stats(
        &mut self,
        top_visual: usize,
        visible_h: usize,
        text_width: usize,
        show_tabs: bool,
    ) {
        self.cached_total_chars = self.total_char_count();
        self.cached_visible_chars =
            self.count_visible_chars_in_viewport(top_visual, visible_h, text_width, show_tabs);
    }

    fn count_visible_chars_in_viewport(
        &self,
        top_visual: usize,
        visible_h: usize,
        text_width: usize,
        show_tabs: bool,
    ) -> usize {
        if visible_h == 0 {
            return 0;
        }
        if !self.word_wrap || text_width == 0 {
            let top_line = if self.word_wrap {
                top_visual
            } else {
                self.viewport.top_line
            };
            return self.count_visible_chars(top_line, visible_h);
        }
        let mut count = 0usize;
        for row in 0..visible_h {
            let Some(display_row) =
                wrap::build_display_row_at(self, top_visual + row, text_width, show_tabs)
            else {
                break;
            };
            let expanded = wrap::expanded_line(self, display_row.doc_line, show_tabs);
            let segs = wrap::segment_starts(&expanded, text_width, true);
            let seg_end = segs
                .get(display_row.seg_index + 1)
                .copied()
                .unwrap_or_else(|| expanded.chars().count());
            let line_str = self.text.line(display_row.doc_line).to_string();
            let line_body = line_str.trim_end_matches('\n');
            let tab_width = tab_stop_width(self.tabulation);
            let char_start =
                char_col_from_visual(line_body, display_row.seg_start, tab_width);
            let char_end = char_col_from_visual(line_body, seg_end, tab_width);
            count += char_end.saturating_sub(char_start);
        }
        count
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
        let line_count = self.text.len_lines().max(1);
        let (line, col) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let line_str = self.text.line(line).to_string();
        let vis_col = visual_col_in_line(&line_str, col, tab_stop_width(self.tabulation));
        let w = self.viewport.width as usize;
        if self.word_wrap && w > 0 {
            let total = wrap::total_visual_rows(self, w, self.render_show_tabs);
            let vrow = wrap::visual_row_for_cursor(self, line, vis_col, w, self.render_show_tabs);
            self.viewport.ensure_visual_row_visible(vrow, total);
            self.viewport.left_col = 0;
            if let Some(dr) =
                wrap::build_display_row_at(self, self.viewport.top_visual_row, w, self.render_show_tabs)
            {
                self.viewport.top_line = dr.doc_line;
            }
        } else {
            self.viewport
                .ensure_cursor_visible(line, vis_col, line_count);
            self.viewport.top_visual_row = self.viewport.top_line;
        }
    }

    fn record(&mut self, start: usize, removed: String, inserted: String, before: usize, after: usize) {
        self.history
            .record_change(start, removed, inserted, before, after);
    }

    fn record_block_delete(
        &mut self,
        patches: Vec<crate::editor::selection::BlockDeletePatch>,
        before: usize,
        after: usize,
    ) {
        self.history.record_block_delete(patches, before, after);
    }

    fn record_multi(
        &mut self,
        patches: Vec<(usize, String, String)>,
        before: usize,
        after: usize,
    ) {
        self.history.record_multi_linear(patches, before, after);
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
        let materialized = self.materialize_virtual_at_primary();
        let idx = self.primary().char_idx.min(self.text.len_chars());
        self.text.insert_char(idx, ch);
        self.primary_mut().char_idx = idx + 1;
        self.sync_primary_virtual();
        let (record_start, inserted) = if let Some((start, spaces)) = materialized {
            let mut ins = spaces;
            ins.push(ch);
            (start, ins)
        } else {
            (idx, ch.to_string())
        };
        self.record(
            record_start,
            String::new(),
            inserted,
            before,
            self.primary().char_idx,
        );
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
        let before = self.primary().char_idx;
        let mut indices: Vec<usize> = self.cursors.iter().map(|c| c.char_idx).collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        let before_positions: Vec<usize> = indices.clone();
        let mut patches = Vec::new();
        for idx in indices {
            let (insert_idx, materialized) = self.materialize_at(idx);
            let record_start = materialized
                .as_ref()
                .map(|(start, _)| *start)
                .unwrap_or(insert_idx);
            let mut inserted = materialized.map(|(_, spaces)| spaces).unwrap_or_default();
            inserted.push(ch);
            self.text.insert_char(insert_idx, ch);
            patches.push((record_start, String::new(), inserted));
        }
        for c in &mut self.cursors {
            if before_positions.contains(&c.char_idx) {
                c.char_idx += 1;
            }
        }
        merge_cursors(&mut self.cursors);
        self.record_multi(patches, before, self.primary().char_idx);
        self.ensure_visible();
    }

    pub fn insert_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.insert_char(ch);
        }
    }

    fn materialize_at(&mut self, char_idx: usize) -> (usize, Option<(usize, String)>) {
        let (line, col) = char_idx_to_line_col(&self.text, char_idx.min(self.text.len_chars()));
        let line_start = self.text.line_to_char(line);
        let line_len = self.text.line(line).len_chars();
        if col > line_len {
            let spaces: String = " ".repeat(col - line_len);
            let insert_at = line_start + line_len;
            self.text.insert(insert_at, &spaces);
            let insert_idx = insert_at + spaces.chars().count();
            return (insert_idx, Some((insert_at, spaces)));
        }
        (char_idx, None)
    }

    fn materialize_virtual_at_primary(&mut self) -> Option<(usize, String)> {
        let virtual_col = self.primary().virtual_col;
        let char_idx = self.primary().char_idx.min(self.text.len_chars());
        let (line, col) = char_idx_to_line_col(&self.text, char_idx);
        let line_start = self.text.line_to_char(line);
        let line_str = self.text.line(line).to_string();
        let tab_width = tab_stop_width(self.tabulation);
        let line_visual = visual_col_in_line(&line_str, col, tab_width);
        let line_char_len = self.text.line(line).len_chars();

        if virtual_col <= line_visual {
            return None;
        }
        if line_char_len == 0 {
            self.primary_mut().virtual_col = 0;
            return None;
        }
        // Caret clamped on a shorter line after ↑/↓ — não preencher com espaços.
        if col < line_char_len {
            self.primary_mut().virtual_col = line_visual;
            return None;
        }
        // Só materializa espaços quando o caret está no fim da linha e virtual_col avançou além.
        if virtual_col > line_visual {
            let spaces: String = " ".repeat(virtual_col - line_visual);
            let insert_at = line_start + line_char_len;
            self.text.insert(insert_at, &spaces);
            self.primary_mut().char_idx = insert_at + spaces.chars().count();
            return Some((insert_at, spaces));
        }
        None
    }

    pub fn backspace(&mut self) {
        if self.selection_mode == SelectionMode::Multi {
            let before = self.primary().char_idx;
            let mut indices: Vec<usize> = self.cursors.iter().map(|c| c.char_idx).collect();
            indices.sort_unstable_by(|a, b| b.cmp(a));
            let mut patches = Vec::new();
            for idx in indices {
                if idx > 0 {
                    let removed = self.text.slice(idx - 1..idx).to_string();
                    self.text.remove(idx - 1..idx);
                    patches.push((idx - 1, removed, String::new()));
                }
            }
            for c in &mut self.cursors {
                if c.char_idx > 0 {
                    c.char_idx -= 1;
                }
            }
            merge_cursors(&mut self.cursors);
            if !patches.is_empty() {
                self.record_multi(patches, before, self.primary().char_idx);
            }
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
            let before = self.primary().char_idx;
            let mut indices: Vec<usize> = self.cursors.iter().map(|c| c.char_idx).collect();
            indices.sort_unstable_by(|a, b| b.cmp(a));
            let mut patches = Vec::new();
            for idx in indices {
                if idx < self.text.len_chars() {
                    let removed = self.text.slice(idx..idx + 1).to_string();
                    self.text.remove(idx..idx + 1);
                    patches.push((idx, removed, String::new()));
                }
            }
            merge_cursors(&mut self.cursors);
            if !patches.is_empty() {
                self.record_multi(patches, before, self.primary().char_idx);
            }
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
        self.sync_primary_virtual();
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
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn scroll_page_up(&mut self) {
        let h = self.viewport.height as usize;
        if h <= 1 {
            return;
        }
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let step = h.saturating_sub(1).max(1);
        let target = line.saturating_sub(step);
        let line_str = self.text.line(target).to_string();
        let vcol = self.primary().virtual_col;
        let char_col = char_col_from_visual(&line_str, vcol, tab_stop_width(self.tabulation));
        let idx = line_col_to_char_idx(&self.text, target, char_col);
        self.primary_mut().char_idx = idx;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn scroll_page_down(&mut self) {
        let h = self.viewport.height as usize;
        if h <= 1 {
            return;
        }
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let step = h.saturating_sub(1).max(1);
        let last_line = self.text.len_lines().saturating_sub(1);
        let target = (line + step).min(last_line);
        let line_str = self.text.line(target).to_string();
        let vcol = self.primary().virtual_col;
        let char_col = char_col_from_visual(&line_str, vcol, tab_stop_width(self.tabulation));
        let idx = line_col_to_char_idx(&self.text, target, char_col);
        self.primary_mut().char_idx = idx;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    /// Roda do mouse: `delta` negativo sobe, positivo desce (número de linhas).
    pub fn scroll_wheel_lines(&mut self, delta: i32) {
        if delta == 0 {
            return;
        }
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let last_line = self.text.len_lines().saturating_sub(1);
        let step = delta.unsigned_abs() as usize;
        let target = if delta < 0 {
            line.saturating_sub(step)
        } else {
            (line + step).min(last_line)
        };
        if target == line {
            return;
        }
        let line_str = self.text.line(target).to_string();
        let vcol = self.primary().virtual_col;
        let char_col = char_col_from_visual(&line_str, vcol, tab_stop_width(self.tabulation));
        let idx = line_col_to_char_idx(&self.text, target, char_col);
        self.primary_mut().char_idx = idx;
        self.sync_primary_virtual();
        self.ensure_visible();
    }

    pub fn move_to_document_start(&mut self, extend: bool) {
        self.prepare_move(extend);
        self.primary_mut().char_idx = 0;
        self.sync_primary_virtual();
        self.viewport.top_line = 0;
        self.viewport.top_visual_row = 0;
        self.viewport.left_col = 0;
        self.ensure_visible();
    }

    pub fn move_to_document_end(&mut self, extend: bool) {
        self.prepare_move(extend);
        self.primary_mut().char_idx = self.text.len_chars();
        self.sync_primary_virtual();
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
        let line_len = line_content_len(&self.text, line);
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

    /// Texto da linha do cursor primário (sem `\n`).
    pub fn current_line_text(&self) -> String {
        if self.text.len_lines() == 0 {
            return String::new();
        }
        let (line, _) = char_idx_to_line_col(&self.text, self.primary().char_idx);
        let line = line.min(self.text.len_lines().saturating_sub(1));
        self.text.line(line).to_string()
    }

    /// Seleção ativa ou, se não houver, a linha do cursor.
    pub fn text_for_terminal_insert(&self) -> String {
        self.copy_text()
            .unwrap_or_else(|| self.current_line_text())
    }

    fn delete_selection(&mut self) -> bool {
        match self.selection_mode {
            SelectionMode::Block => {
                if let Some(block) = self.block_selection {
                    let before = self.primary().char_idx;
                    let tw = tab_stop_width(self.tabulation);
                    let patches = collect_block_delete_patches(&self.text, &block, tw);
                    delete_block(&mut self.text, &block, tw);
                    self.block_selection = None;
                    self.selection_mode = SelectionMode::Normal;
                    self.cursors.truncate(1);
                    self.primary_mut().char_idx =
                        self.primary().char_idx.min(self.text.len_chars());
                    let after = self.primary().char_idx;
                    self.record_block_delete(patches, before, after);
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
                        let before = self.primary().char_idx;
                        let removed = self.text.slice(a..b).to_string();
                        self.text.remove(a..b);
                        self.primary_mut().char_idx = a;
                        self.primary_mut().anchor = None;
                        self.sync_primary_virtual();
                        self.record(a, removed, String::new(), before, a);
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
        if pattern.is_empty() {
            return false;
        }
        let before = self.primary().char_idx;
        let Some((start_char, end_char)) =
            crate::editor::search::match_range_for_replace(&self.text, before, &pattern)
        else {
            return false;
        };
        let removed = self.text.slice(start_char..end_char).to_string();
        self.text.remove(start_char..end_char);
        self.text.insert(start_char, replacement);
        let after = start_char + replacement.chars().count();
        self.primary_mut().char_idx = start_char;
        self.sync_primary_virtual();
        self.record(
            start_char,
            removed,
            replacement.to_string(),
            before,
            after,
        );
        true
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
    fn text_for_terminal_insert_prefers_selection_over_current_line() {
        let mut e = EditorEngine::new();
        e.load_text("alpha beta");
        e.set_cursor_line_col(0, 0);
        for _ in 0..5 {
            e.move_right(true);
        }
        assert_eq!(e.text_for_terminal_insert(), "alpha");
    }

    #[test]
    fn text_for_terminal_insert_uses_current_line_without_selection() {
        let mut e = EditorEngine::new();
        e.load_text("one\ntwo");
        e.set_cursor_line_col(1, 1);
        assert_eq!(e.text_for_terminal_insert(), "two");
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
        assert_eq!(e.line_logical_len(0), 5);
        assert_eq!(e.line_logical_len(1), 5);
        assert_eq!(e.line_logical_len(2), 4);
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
    fn visible_char_count_treats_literal_tab_as_one_char() {
        use crate::encoding::Tabulation;

        let mut e = EditorEngine::new();
        e.tabulation = Tabulation::TabLiteral;
        e.load_text("\tworld");
        e.viewport.height = 1;
        assert_eq!(e.visible_char_count(), 6);
        assert_eq!(e.total_char_count(), 6);
    }

    #[test]
    fn undo_block_replace_restores_deleted_block() {
        let mut e = EditorEngine::new();
        e.load_text("abcd\nwxyz");
        e.start_block_select(0, 1);
        e.update_block_select(1, 3);
        e.finish_block_select();
        e.insert_char('x');
        e.insert_char('y');
        e.undo();
        e.undo();
        e.undo();
        assert_eq!(e.text.to_string(), "abcd\nwxyz");
    }

    #[test]
    fn undo_linear_selection_delete_restores_text() {
        let mut e = EditorEngine::new();
        e.load_text("hello world");
        e.primary_mut().anchor = Some(0);
        e.primary_mut().char_idx = 5;
        e.backspace();
        assert_eq!(e.text.to_string(), " world");
        e.undo();
        assert_eq!(e.text.to_string(), "hello world");
    }

    #[test]
    fn undo_replace_one_restores_match() {
        let mut e = EditorEngine::new();
        e.load_text("foo bar foo");
        e.search_pattern = "foo".into();
        e.set_cursor_line_col(0, 0);
        assert!(e.replace_one("baz"));
        assert_eq!(e.text.to_string(), "baz bar foo");
        e.undo();
        assert_eq!(e.text.to_string(), "foo bar foo");
    }

    #[test]
    fn undo_multi_cursor_insert() {
        let mut e = EditorEngine::new();
        e.load_text("abc");
        e.set_cursor_line_col(0, 1);
        e.add_cursor(0, 2);
        e.insert_char('X');
        assert_eq!(e.text.to_string(), "aXbXc");
        e.undo();
        assert_eq!(e.text.to_string(), "abc");
    }

    #[test]
    fn undo_materialize_spaces_with_typed_char() {
        let mut e = EditorEngine::new();
        e.load_text("hello world\nshort");
        e.set_cursor_line_col(0, 10);
        e.move_down(false);
        e.insert_char('x');
        e.undo();
        assert_eq!(e.text.to_string(), "hello world\nshort");
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

    #[test]
    fn move_up_from_readme_table_reaches_document_start() {
        let content = std::fs::read_to_string("README.md").expect("readme");
        let mut e = EditorEngine::new();
        e.load_text(&content);
        e.viewport.height = 15;
        e.set_cursor_line_col(72, 0);
        assert_eq!(e.cursor_line_col().0, 73);
        for _ in 0..72 {
            e.move_up(false);
        }
        let (line, _) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 0);
        assert_eq!(e.viewport.top_line, 0);
    }

    #[test]
    fn scroll_page_up_moves_viewport_on_long_document() {
        let mut e = EditorEngine::new();
        e.load_text(&"line\n".repeat(80));
        e.viewport.height = 10;
        e.set_cursor_line_col(70, 0);
        e.scroll_page_up();
        let (line, _) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 61);
    }

    #[test]
    fn scroll_wheel_lines_moves_cursor() {
        let mut e = EditorEngine::new();
        e.load_text("a\nb\nc\nd\ne\nf\ng\n");
        e.set_cursor_line_col(4, 0);
        e.scroll_wheel_lines(-3);
        let (line, _) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 1);
        e.scroll_wheel_lines(2);
        let (line, _) = char_idx_to_line_col(&e.text, e.primary().char_idx);
        assert_eq!(line, 3);
    }

    #[test]
    fn move_end_stays_on_same_line() {
        let mut e = EditorEngine::new();
        e.load_text("hello\nworld");
        e.set_cursor_line_col(0, 0);
        e.move_end(false);
        assert_eq!(e.cursor_raw(), (0, 5));
        e.set_cursor_line_col(1, 0);
        e.move_end(false);
        assert_eq!(e.cursor_raw(), (1, 5));
    }
}
