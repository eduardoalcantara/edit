mod commands;
mod cursor;
mod engine;
mod history;
pub mod line_numbers;
mod render;
mod search;
mod selection;
mod tabs;
mod viewport;
mod word_boundary;
mod wrap;

pub use commands::EditorCommand;
pub use cursor::SelectionMode;
pub use engine::{EditorEngine, EMPTY_DOCUMENT_TEXT};
pub use tabs::convert_tabulation_between;

/// Área de texto do editor (inner + margens) para viewport e hit-test.
pub fn editor_viewport_rect(
    shell: Rect,
    border: EditorBorder,
    terminal_block: Option<u16>,
    margin: EditorMargin,
) -> Rect {
    render::editor_viewport_rect(shell, border, terminal_block, margin)
}

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::clipboard::Clipboard;
use crate::edit_mode::EditMode;
use crate::editor::selection::selection_label;
use crate::encoding::Tabulation;
use crate::theme::ThemePalette;
use crate::view_state::{EditorBorder, EditorMargin};

pub struct Editor {
    engine: EditorEngine,
    /// Área clicável do texto (sem a coluna de números).
    text_area: Rect,
    /// Área do conteúdo (gutter + texto) para rolagem com mouse.
    content_area: Rect,
    select_drag_origin: Option<usize>,
}

impl Editor {
    pub fn new(_palette: &ThemePalette) -> Self {
        Self {
            engine: EditorEngine::new(),
            text_area: Rect::default(),
            content_area: Rect::default(),
            select_drag_origin: None,
        }
    }

    pub fn engine(&self) -> &EditorEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut EditorEngine {
        &mut self.engine
    }

    pub fn text_area(&self) -> Rect {
        self.text_area
    }

    pub fn content_area(&self) -> Rect {
        self.content_area
    }

    /// Compat: área de texto para hit-test de clique/seleção.
    pub fn inner_area(&self) -> Rect {
        self.text_area
    }

    pub fn apply_theme(&mut self, _palette: &ThemePalette) {}

    pub fn set_word_wrap(&mut self, on: bool) {
        self.engine.word_wrap = on;
    }

    pub fn set_tabulation(&mut self, tab: Tabulation) {
        self.engine.tabulation = tab;
    }

    pub fn tabulation(&self) -> Tabulation {
        self.engine.tabulation
    }

    pub fn mode(&self) -> EditMode {
        self.engine.input_mode
    }

    pub fn set_mode(&mut self, mode: EditMode, _palette: &ThemePalette) {
        self.engine.input_mode = mode;
    }

    pub fn toggle_mode(&mut self, palette: &ThemePalette) {
        self.set_mode(self.mode().toggle(), palette);
    }

    pub fn execute(&mut self, cmd: EditorCommand) {
        match cmd {
            EditorCommand::InsertChar(ch) => self.engine.insert_char(ch),
            EditorCommand::Backspace => self.engine.backspace(),
            EditorCommand::Delete => self.engine.delete(),
            EditorCommand::MoveLeft { extend } => self.engine.move_left(extend),
            EditorCommand::MoveRight { extend } => self.engine.move_right(extend),
            EditorCommand::MoveWordLeft { extend } => self.engine.move_word_left(extend),
            EditorCommand::MoveWordRight { extend } => self.engine.move_word_right(extend),
            EditorCommand::MoveUp { extend } => self.engine.move_up(extend),
            EditorCommand::MoveDown { extend } => self.engine.move_down(extend),
            EditorCommand::Home { extend } => self.engine.move_home(extend),
            EditorCommand::End { extend } => self.engine.move_end(extend),
            EditorCommand::DocumentStart { extend } => self.engine.move_to_document_start(extend),
            EditorCommand::DocumentEnd { extend } => self.engine.move_to_document_end(extend),
            EditorCommand::PageUp => self.engine.scroll_page_up(),
            EditorCommand::PageDown => self.engine.scroll_page_down(),
            EditorCommand::ScrollWheel { delta } => self.engine.scroll_wheel_lines(delta),
            EditorCommand::SelectAll => self.engine.select_all(),
            EditorCommand::CancelSelection => self.engine.cancel_selection(),
            EditorCommand::Undo => self.engine.undo(),
            EditorCommand::Redo => self.engine.redo(),
            EditorCommand::Tab => self.engine.insert_tab(),
            EditorCommand::Paste(text) => self.engine.insert_str(&text),
            EditorCommand::StartBlockSelect { line, col } => {
                self.engine.start_block_select(line, col);
            }
            EditorCommand::UpdateBlockSelect { line, col } => {
                self.engine.update_block_select(line, col);
            }
            EditorCommand::EndBlockSelect => self.engine.finish_block_select(),
            EditorCommand::AddCursor { line, col } => self.engine.add_cursor(line, col),
            EditorCommand::SetCursor { line, col } => self.engine.set_cursor_line_col(line, col),
            EditorCommand::Click { line, col } => {
                self.select_drag_origin = None;
                self.engine.set_cursor_line_col(line, col);
            }
            EditorCommand::ExtendSelection { line, col } => {
                self.engine.extend_selection_to(line, col);
            }
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        palette: ThemePalette,
        margin: EditorMargin,
        border: EditorBorder,
        terminal_block: Option<u16>,
        text_viewport: Option<Rect>,
        show_cursor: bool,
        show_tabs: bool,
        show_line_numbers: bool,
        pane_border: crate::widgets::panel::PanelBorder,
    ) {
        self.engine.render_show_tabs = show_tabs;
        let (text_area, content_area) = render::draw(
            &mut self.engine,
            frame,
            area,
            title,
            palette,
            margin,
            border,
            terminal_block,
            text_viewport,
            show_cursor,
            show_tabs,
            show_line_numbers,
            pane_border,
        );
        self.text_area = text_area;
        self.content_area = content_area;
    }

    pub fn viewport_to_doc(&self, vp_line: usize, vp_col: usize) -> (usize, usize) {
        let (line, vis_col) = crate::input::mouse::viewport_to_doc(
            vp_line,
            vp_col,
            self.engine.viewport.top_line,
            self.engine.viewport.left_col,
        );
        let line = line.min(self.engine.text.len_lines().saturating_sub(1));
        let line_str = self.engine.text.line(line).to_string();
        let tab_width = tabs::tab_stop_width(self.engine.tabulation);
        let char_col = tabs::char_col_from_visual(&line_str, vis_col, tab_width);
        (line, char_col)
    }

    pub fn is_linear_dragging(&self) -> bool {
        self.select_drag_origin.is_some()
    }

    pub fn begin_mouse_select(&mut self, line: usize, col: usize) {
        self.engine.set_caret_line_col(line, col, true);
        self.select_drag_origin = Some(self.engine.primary().char_idx);
    }

    pub fn drag_mouse_select(&mut self, line: usize, col: usize) {
        if let Some(origin) = self.select_drag_origin {
            self.engine.primary_mut().anchor = Some(origin);
            self.engine.extend_selection_to(line, col);
        }
    }

    pub fn finish_mouse_select(&mut self) {
        if let Some(origin) = self.select_drag_origin {
            if self.engine.primary().anchor == Some(origin)
                && self.engine.primary().char_idx == origin
            {
                self.engine.primary_mut().anchor = None;
            }
        }
        self.select_drag_origin = None;
    }

    pub fn is_block_dragging(&self) -> bool {
        self.engine.is_block_dragging()
    }

    pub fn selection_mode_label(&self) -> String {
        match self.engine.selection_mode {
            SelectionMode::Normal => "normal".to_string(),
            SelectionMode::Block => "bloco".to_string(),
            SelectionMode::Multi => format!("multi:{}", self.engine.cursors.len()),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.engine.to_lines()
    }

    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.engine.load_lines(&lines);
    }

    pub fn content_string(&self) -> String {
        self.engine.content_string()
    }

    pub fn clear(&mut self) {
        self.engine.load_text("");
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        self.engine.cursor_line_col()
    }

    pub fn cursor_raw(&self) -> (usize, usize) {
        self.engine.cursor_raw()
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.engine.set_cursor_line_col(row, col);
    }

    pub fn selection_label(&self) -> String {
        selection_label(
            self.engine.selection_mode,
            self.engine.block_selection.as_ref(),
            &self.engine.cursors,
            self.engine.primary(),
            &self.engine.text,
        )
    }

    pub fn byte_size(&self) -> usize {
        self.engine.byte_size()
    }

    pub fn visible_char_count(&self) -> usize {
        self.engine.footer_visible_chars()
    }

    pub fn total_char_count(&self) -> usize {
        self.engine.footer_total_chars()
    }

    pub fn line_count(&self) -> usize {
        self.engine.text.len_lines().max(1)
    }

    pub fn undo(&mut self) {
        self.engine.undo();
    }

    pub fn redo(&mut self) {
        self.engine.redo();
    }

    pub fn select_all(&mut self) {
        self.engine.select_all();
    }

    pub fn cancel_selection(&mut self) {
        self.select_drag_origin = None;
        self.engine.cancel_selection();
    }

    pub fn copy_selection(&mut self, clipboard: &mut Clipboard) -> bool {
        if let Some(text) = self.engine.copy_text() {
            if !text.is_empty() {
                clipboard.push(text);
                return true;
            }
        }
        false
    }

    pub fn text_for_terminal_insert(&self) -> String {
        self.engine.text_for_terminal_insert()
    }

    pub fn cut_selection(&mut self, clipboard: &mut Clipboard) -> bool {
        if let Some(text) = self.engine.cut_selection() {
            if !text.is_empty() {
                clipboard.push(text);
                return true;
            }
        }
        false
    }

    pub fn paste(&mut self, text: &str) {
        self.engine.insert_str(text);
    }

    pub fn set_search_pattern(&mut self, pattern: &str) {
        self.engine.search_pattern = pattern.to_string();
        self.engine.search_match_start = None;
        self.engine.search_match_positions.clear();
    }

    pub fn search_pattern(&self) -> &str {
        &self.engine.search_pattern
    }

    pub fn find_next(&mut self) -> bool {
        self.engine.find_next()
    }

    pub fn find_prev(&mut self) -> bool {
        self.engine.find_prev()
    }

    pub fn find_first(&mut self) -> bool {
        self.engine.find_first()
    }

    pub fn find_last(&mut self) -> bool {
        self.engine.find_last()
    }

    pub fn replace_one(&mut self, replacement: &str) -> bool {
        self.engine.replace_one(replacement)
    }

    pub fn replace_all(&mut self, replacement: &str) -> usize {
        self.engine.replace_all(replacement)
    }

    pub fn replace_content(&mut self, content: &str) {
        let (line, col) = self.engine.cursor_raw();
        let top_line = self.engine.viewport.top_line;
        let left_col = self.engine.viewport.left_col;
        self.engine.load_text(content);
        self.engine.viewport.top_line = top_line;
        self.engine.viewport.left_col = left_col;
        self.engine.set_caret_line_col(line, col, true);
    }
}
