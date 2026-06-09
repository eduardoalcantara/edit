mod listing;
mod path_resolve;

pub use listing::{FileEntry, FileEntryKind, list_directory};
pub use path_resolve::{
    infer_filter_from_path, initial_directory, resolve_open_target, resolve_save_target,
    suggest_file_name,
};

use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::modal::buttons::{FILE_BROWSER_OPEN, FILE_BROWSER_SAVE};
use crate::modal::form_controls::paint_text_input;
use crate::modal::text_input::{CharAccept, TextInput};
use crate::clipboard::Clipboard;
use crate::modal::dialog::{
    centered_dialog_rect, dialog_content_rect, paint_titled_dialog_content, Dialog,
};
use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

const DIALOG_WIDTH: u16 = 50;
/// Altura máxima da moldura (sem contar a sombra de 1 linha abaixo).
const MAX_OUTER_HEIGHT: u16 = 24;
const MIN_OUTER_HEIGHT: u16 = 16;
const FIELD_HEIGHT: u16 = 1;
const LIST_MIN_ROWS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserMode {
    Open,
    Save,
    SaveAs,
}

impl FileBrowserMode {
    pub fn title(self) -> &'static str {
        match self {
            FileBrowserMode::Open => "Abrir",
            FileBrowserMode::Save => "Salvar",
            FileBrowserMode::SaveAs => "Salvar Como",
        }
    }

    fn primary_label(self) -> &'static str {
        match self {
            FileBrowserMode::Open => "Abrir",
            FileBrowserMode::Save | FileBrowserMode::SaveAs => "Salvar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserFocus {
    Name,
    List,
    Filter,
    HiddenToggle,
    PrimaryButton,
    CancelButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserKeyResult {
    Consumed,
    Submit,
    Cancel,
}

#[derive(Debug, Clone)]
struct Layout {
    name_label: Rect,
    name_field: Rect,
    primary_btn: Rect,
    files_label: Rect,
    hidden_toggle: Rect,
    list_box: Rect,
    filter_label: Rect,
    filter_field: Rect,
    cancel_btn: Rect,
}

#[derive(Debug, Clone)]
pub struct FileBrowserModal {
    pub dialog: Dialog,
    pub mode: FileBrowserMode,
    pub current_dir: PathBuf,
    pub name_input: TextInput,
    pub filter_input: TextInput,
    pub show_hidden: bool,
    pub entries: Vec<FileEntry>,
    pub list_cursor: usize,
    pub list_scroll: usize,
    pub focus: FileBrowserFocus,
    pub status_msg: String,
    pub pending_submit: bool,
    last_click: Option<(Instant, usize)>,
}

impl FileBrowserModal {
    pub fn new(
        mode: FileBrowserMode,
        current_dir: PathBuf,
        name_input: String,
        filter_input: String,
        show_hidden: bool,
    ) -> Self {
        let buttons = match mode {
            FileBrowserMode::Open => &FILE_BROWSER_OPEN,
            FileBrowserMode::Save | FileBrowserMode::SaveAs => &FILE_BROWSER_SAVE,
        };
        let mut modal = Self {
            dialog: Dialog::form(mode.title(), String::new(), buttons),
            mode,
            current_dir,
            name_input: TextInput::new(name_input),
            filter_input: TextInput::new(filter_input),
            show_hidden,
            entries: Vec::new(),
            list_cursor: 0,
            list_scroll: 0,
            focus: FileBrowserFocus::Name,
            status_msg: String::new(),
            pending_submit: false,
            last_click: None,
        };
        modal.refresh_listing();
        modal
    }

    pub fn refresh_listing(&mut self) {
        match list_directory(&self.current_dir, self.filter_input.text(), self.show_hidden) {
            Ok(entries) => {
                self.entries = entries;
                self.status_msg.clear();
            }
            Err(msg) => {
                self.entries.clear();
                self.status_msg = msg;
            }
        }
        if self.list_cursor >= self.entries.len() {
            self.list_cursor = self.entries.len().saturating_sub(1);
        }
        self.sync_name_from_selection();
        self.ensure_list_visible();
    }

    pub fn resolved_path(&self) -> PathBuf {
        match self.mode {
            FileBrowserMode::Open => {
                if let Some(entry) = self.entries.get(self.list_cursor) {
                    if entry.kind == FileEntryKind::File {
                        return entry.path.clone();
                    }
                }
                resolve_open_target(&self.current_dir, self.name_input.text())
            }
            FileBrowserMode::Save | FileBrowserMode::SaveAs => {
                resolve_save_target(&self.current_dir, self.name_input.text())
            }
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = DIALOG_WIDTH.min(area.width);
        let proportional = area.height.saturating_mul(65).saturating_div(100);
        let height = proportional
            .max(MIN_OUTER_HEIGHT)
            .min(MAX_OUTER_HEIGHT)
            .min(area.height);
        centered_dialog_rect(area, width, height)
    }

    /// Linhas da moldura + 1 linha de sombra horizontal (quando cabe no terminal).
    pub fn total_footprint_rows(&self, area: Rect) -> u16 {
        let outer = self.outer_rect(area);
        let shadow = u16::from(outer.y.saturating_add(outer.height) < area.height);
        outer.height.saturating_add(shadow)
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = paint_titled_dialog_content(frame, area, self.mode.title(), palette);
        let layout = self.layout(content);

        self.paint_label(frame, layout.name_label, "Nome", palette);
        paint_text_input(
            frame,
            layout.name_field,
            &self.name_input,
            self.focus == FileBrowserFocus::Name,
            palette,
        );
        self.paint_button(
            frame,
            layout.primary_btn,
            self.mode.primary_label(),
            self.focus == FileBrowserFocus::PrimaryButton,
            palette,
        );

        self.paint_label(frame, layout.files_label, "Arquivos", palette);
        self.paint_hidden_toggle(frame, layout.hidden_toggle, palette);
        self.paint_list(frame, layout.list_box, palette);

        self.paint_label(frame, layout.filter_label, "Filtro", palette);
        paint_text_input(
            frame,
            layout.filter_field,
            &self.filter_input,
            self.focus == FileBrowserFocus::Filter,
            palette,
        );
        self.paint_button(
            frame,
            layout.cancel_btn,
            "Cancelar",
            self.focus == FileBrowserFocus::CancelButton,
            palette,
        );
    }

    fn paint_label(&self, frame: &mut Frame<'_>, area: Rect, text: &str, palette: ThemePalette) {
        frame.render_widget(
            Paragraph::new(text).style(palette.status_style()),
            area,
        );
    }

    fn paint_button(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        label: &str,
        focused: bool,
        palette: ThemePalette,
    ) {
        let text = format!("[{label}]");
        frame.render_widget(
            Paragraph::new(text).style(palette.button_style(focused)),
            area,
        );
    }

    fn paint_field(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        text: &str,
        focused: bool,
        palette: ThemePalette,
    ) {
        let mut field_style = palette.editor_text_style();
        if focused {
            field_style = field_style.add_modifier(Modifier::BOLD);
        }
        panel::fill_rect(frame, area, field_style);
        let display = if focused {
            format!(" {text}▌")
        } else {
            format!(" {text}")
        };
        frame.render_widget(Paragraph::new(display).style(field_style), area);
    }

    fn paint_list(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        let border_style = if self.focus == FileBrowserFocus::List {
            palette.convert_field_border_style(true)
        } else {
            palette.menu_border_style()
        };
        panel::render_frame(
            frame,
            area,
            Style::default().bg(ratatui::style::Color::Cyan),
            border_style,
            PanelBorder::Plain,
        );
        let inner = panel::inner_rect(area);
        let visible = inner.height as usize;
        for row in 0..visible {
            let idx = self.list_scroll + row;
            let y = inner.y.saturating_add(row as u16);
            if let Some(entry) = self.entries.get(idx) {
                let label = match entry.kind {
                    FileEntryKind::Dir | FileEntryKind::Parent => format!("{}/", entry.name),
                    FileEntryKind::File => entry.name.clone(),
                };
                let selected = idx == self.list_cursor;
                let style = if selected {
                    Style::default()
                        .fg(palette.status)
                        .add_modifier(Modifier::BOLD)
                        .bg(ratatui::style::Color::Cyan)
                } else {
                    Style::default().fg(ratatui::style::Color::Black).bg(ratatui::style::Color::Cyan)
                };
                frame.render_widget(
                    Paragraph::new(label).style(style),
                    Rect {
                        x: inner.x,
                        y,
                        width: inner.width,
                        height: 1,
                    },
                );
            } else {
                frame.render_widget(
                    Paragraph::new("").style(Style::default().bg(ratatui::style::Color::Cyan)),
                    Rect {
                        x: inner.x,
                        y,
                        width: inner.width,
                        height: 1,
                    },
                );
            }
        }
    }

    fn paint_hidden_toggle(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        let mark = if self.show_hidden { '√' } else { ' ' };
        let focused = self.focus == FileBrowserFocus::HiddenToggle;
        let style = if focused {
            palette.menu_item_focus_style()
        } else {
            palette.menu_panel_style()
        };
        frame.render_widget(
            Paragraph::new(format!("[{mark}] Mostrar arquivos ocultos")).style(style),
            area,
        );
    }

    pub fn focused_help(&self) -> Option<String> {
        if !self.status_msg.is_empty() {
            return Some(self.status_msg.clone());
        }
        let help = match self.focus {
            FileBrowserFocus::Name => "Nome ou caminho do arquivo",
            FileBrowserFocus::List => "↑/↓ navega; Enter abre pasta ou confirma arquivo",
            FileBrowserFocus::Filter => "Máscara de arquivos, ex.: *.rs ou *.*",
            FileBrowserFocus::HiddenToggle => "Mostra arquivos/pastas ocultos",
            FileBrowserFocus::PrimaryButton => self.dialog.buttons[0].help,
            FileBrowserFocus::CancelButton => self.dialog.buttons[1].help,
        };
        Some(help.to_string())
    }

    pub fn handle_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> FileBrowserKeyResult {
        if key.code == KeyCode::Esc {
            return FileBrowserKeyResult::Cancel;
        }
        if key.code == KeyCode::F(5) {
            self.refresh_listing();
            return FileBrowserKeyResult::Consumed;
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('o' | 'O') => return FileBrowserKeyResult::Submit,
                KeyCode::Char('c' | 'C') => return FileBrowserKeyResult::Cancel,
                _ => {}
            }
        }

        match self.focus {
            FileBrowserFocus::Name => self.handle_name_key(key, clipboard),
            FileBrowserFocus::List => self.handle_list_key(key),
            FileBrowserFocus::Filter => self.handle_filter_key(key, clipboard),
            FileBrowserFocus::HiddenToggle => self.handle_hidden_key(key),
            FileBrowserFocus::PrimaryButton => self.handle_primary_button_key(key),
            FileBrowserFocus::CancelButton => self.handle_cancel_button_key(key),
        }
    }

    fn handle_tab_navigation(&mut self, key: KeyEvent) -> Option<FileBrowserKeyResult> {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                Some(FileBrowserKeyResult::Consumed)
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                Some(FileBrowserKeyResult::Consumed)
            }
            _ => None,
        }
    }

    fn handle_name_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            return FileBrowserKeyResult::Submit;
        }
        if self.name_input.handle_key(key, clipboard, CharAccept::Any) {
            return FileBrowserKeyResult::Consumed;
        }
        FileBrowserKeyResult::Consumed
    }

    fn handle_filter_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            self.refresh_listing();
            return FileBrowserKeyResult::Consumed;
        }
        let refresh = matches!(
            key.code,
            KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Char(_)
        ) || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('v' | 'V' | 'x' | 'X')));
        if self.filter_input.handle_key(key, clipboard, CharAccept::Any) {
            if refresh {
                self.refresh_listing();
            }
            return FileBrowserKeyResult::Consumed;
        }
        FileBrowserKeyResult::Consumed
    }

    fn handle_hidden_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.show_hidden = !self.show_hidden;
                self.refresh_listing();
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Up => {
                self.move_list_cursor(-1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Down => {
                self.move_list_cursor(1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::PageUp => {
                let step = LIST_MIN_ROWS as i32;
                self.move_list_cursor(-step);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::PageDown => {
                let step = LIST_MIN_ROWS as i32;
                self.move_list_cursor(step);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Enter => self.activate_list_entry(),
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_primary_button_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => FileBrowserKeyResult::Submit,
            KeyCode::Left => {
                self.focus = FileBrowserFocus::CancelButton;
                self.dialog.set_selected(1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Right => {
                self.focus = FileBrowserFocus::CancelButton;
                self.dialog.set_selected(1);
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_cancel_button_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => FileBrowserKeyResult::Cancel,
            KeyCode::Left => {
                self.focus = FileBrowserFocus::PrimaryButton;
                self.dialog.set_selected(0);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Right => {
                self.focus = FileBrowserFocus::PrimaryButton;
                self.dialog.set_selected(0);
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn activate_list_entry(&mut self) -> FileBrowserKeyResult {
        let Some(entry) = self.entries.get(self.list_cursor).cloned() else {
            return FileBrowserKeyResult::Consumed;
        };
        match entry.kind {
            FileEntryKind::Parent | FileEntryKind::Dir => {
                self.current_dir = entry.path;
                self.list_cursor = 0;
                self.list_scroll = 0;
                self.refresh_listing();
                FileBrowserKeyResult::Consumed
            }
            FileEntryKind::File => {
                self.name_input.set_text(&entry.name);
                if self.mode == FileBrowserMode::Open {
                    FileBrowserKeyResult::Submit
                } else {
                    FileBrowserKeyResult::Consumed
                }
            }
        }
    }

    pub fn hit_inline_button(&self, mouse: &MouseEvent, outer: Rect) -> Option<usize> {
        let layout = self.layout(dialog_content_rect(outer));
        if rect_contains(layout.primary_btn, mouse) {
            return Some(0);
        }
        if rect_contains(layout.cancel_btn, mouse) {
            return Some(1);
        }
        None
    }

    pub fn handle_mouse(&mut self, mouse: &MouseEvent, outer: Rect) -> bool {
        let content = dialog_content_rect(outer);
        let layout = self.layout(content);
        match mouse.kind {
            MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                if rect_contains(layout.name_field, mouse) {
                    self.focus = FileBrowserFocus::Name;
                    return true;
                }
                if rect_contains(layout.primary_btn, mouse) {
                    self.focus = FileBrowserFocus::PrimaryButton;
                    self.dialog.set_selected(0);
                    if matches!(mouse.kind, MouseEventKind::Down(_)) {
                        self.pending_submit = true;
                    }
                    return true;
                }
                if rect_contains(layout.filter_field, mouse) {
                    self.focus = FileBrowserFocus::Filter;
                    return true;
                }
                if rect_contains(layout.cancel_btn, mouse) {
                    self.focus = FileBrowserFocus::CancelButton;
                    self.dialog.set_selected(1);
                    return true;
                }
                if rect_contains(layout.hidden_toggle, mouse) {
                    self.focus = FileBrowserFocus::HiddenToggle;
                    if matches!(mouse.kind, MouseEventKind::Down(_)) {
                        self.show_hidden = !self.show_hidden;
                        self.refresh_listing();
                    }
                    return true;
                }
                if rect_contains(layout.list_box, mouse) {
                    self.focus = FileBrowserFocus::List;
                    if let Some(idx) = self.hit_list_item(mouse, layout.list_box) {
                        if idx != self.list_cursor {
                            self.list_cursor = idx;
                            self.sync_name_from_selection();
                            self.last_click = Some((Instant::now(), idx));
                        } else if matches!(mouse.kind, MouseEventKind::Down(_)) {
                            if let Some((t, i)) = self.last_click {
                                if i == idx && t.elapsed().as_millis() < 400 {
                                    self.pending_submit = matches!(
                                        self.activate_list_entry(),
                                        FileBrowserKeyResult::Submit
                                    );
                                    self.last_click = None;
                                    return true;
                                }
                            }
                            self.last_click = Some((Instant::now(), idx));
                        }
                    }
                    return true;
                }
            }
            MouseEventKind::Moved => {
                if rect_contains(layout.primary_btn, mouse) {
                    self.focus = FileBrowserFocus::PrimaryButton;
                    self.dialog.set_selected(0);
                } else if rect_contains(layout.cancel_btn, mouse) {
                    self.focus = FileBrowserFocus::CancelButton;
                    self.dialog.set_selected(1);
                }
            }
            MouseEventKind::ScrollUp => {
                if rect_contains(layout.list_box, mouse) {
                    self.focus = FileBrowserFocus::List;
                    self.move_list_cursor(-3);
                    return true;
                }
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(layout.list_box, mouse) {
                    self.focus = FileBrowserFocus::List;
                    self.move_list_cursor(3);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    pub fn mouse_submit(&mut self) -> bool {
        if self.pending_submit {
            self.pending_submit = false;
            return true;
        }
        false
    }

    fn hit_list_item(&self, mouse: &MouseEvent, list_box: Rect) -> Option<usize> {
        let inner = panel::inner_rect(list_box);
        if !rect_contains(inner, mouse) {
            return None;
        }
        let row = mouse.row.saturating_sub(inner.y) as usize;
        let idx = self.list_scroll + row;
        if idx < self.entries.len() {
            Some(idx)
        } else {
            None
        }
    }

    fn layout(&self, content: Rect) -> Layout {
        let w = content.width;
        let primary_w = button_width(self.mode.primary_label());
        let cancel_w = button_width("Cancelar");
        let hidden_w = hidden_toggle_width();

        let mut y = content.y;

        let name_label = Rect { x: content.x, y, width: w, height: 1 };
        y += 1;

        let name_field = Rect {
            x: content.x,
            y,
            width: w.saturating_sub(primary_w.saturating_add(1)),
            height: FIELD_HEIGHT,
        };
        let primary_btn = Rect {
            x: name_field.x.saturating_add(name_field.width.saturating_add(1)),
            y,
            width: primary_w,
            height: 1,
        };
        y += FIELD_HEIGHT;

        let files_label = Rect {
            x: content.x,
            y,
            width: w.saturating_sub(hidden_w),
            height: 1,
        };
        let hidden_toggle = Rect {
            x: content.x.saturating_add(w.saturating_sub(hidden_w)),
            y,
            width: hidden_w,
            height: 1,
        };
        y += 1;

        let filter_label = Rect {
            x: content.x,
            y: content.y.saturating_add(content.height.saturating_sub(FIELD_HEIGHT + 1)),
            width: w,
            height: 1,
        };
        let filter_field = Rect {
            x: content.x,
            y: filter_label.y.saturating_add(1),
            width: w.saturating_sub(cancel_w.saturating_add(1)),
            height: FIELD_HEIGHT,
        };
        let cancel_btn = Rect {
            x: filter_field.x.saturating_add(filter_field.width.saturating_add(1)),
            y: filter_field.y,
            width: cancel_w,
            height: 1,
        };

        let available_list = filter_label.y.saturating_sub(y);
        let list_height = available_list.max(LIST_MIN_ROWS);
        let list_box = Rect {
            x: content.x,
            y,
            width: w,
            height: list_height,
        };

        Layout {
            name_label,
            name_field,
            primary_btn,
            files_label,
            hidden_toggle,
            list_box,
            filter_label,
            filter_field,
            cancel_btn,
        }
    }

    fn cycle_focus(&mut self, delta: i32) {
        let order = [
            FileBrowserFocus::Name,
            FileBrowserFocus::List,
            FileBrowserFocus::Filter,
            FileBrowserFocus::HiddenToggle,
            FileBrowserFocus::PrimaryButton,
            FileBrowserFocus::CancelButton,
        ];
        let current = order
            .iter()
            .position(|&f| f == self.focus)
            .unwrap_or(0) as i32;
        let len = order.len() as i32;
        let next = (current + delta).rem_euclid(len) as usize;
        self.focus = order[next];
        match self.focus {
            FileBrowserFocus::PrimaryButton => self.dialog.set_selected(0),
            FileBrowserFocus::CancelButton => self.dialog.set_selected(1),
            _ => {}
        }
    }

    fn move_list_cursor(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let len = self.entries.len() as i32;
        let next = (self.list_cursor as i32 + delta).clamp(0, len - 1) as usize;
        self.list_cursor = next;
        self.sync_name_from_selection();
        self.ensure_list_visible();
    }

    fn ensure_list_visible(&mut self) {
        let visible = estimated_list_viewport();
        if self.list_cursor < self.list_scroll {
            self.list_scroll = self.list_cursor;
        } else if self.list_cursor >= self.list_scroll.saturating_add(visible) {
            self.list_scroll = self.list_cursor + 1 - visible;
        }
    }

    fn sync_name_from_selection(&mut self) {
        if let Some(entry) = self.entries.get(self.list_cursor) {
            match entry.kind {
                FileEntryKind::File => self.name_input.set_text(&entry.name),
                FileEntryKind::Dir | FileEntryKind::Parent => self.name_input.clear(),
            }
        }
    }

    pub fn enter_directory(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.list_cursor = 0;
        self.list_scroll = 0;
        self.refresh_listing();
    }
}

fn button_width(label: &str) -> u16 {
    label.chars().count() as u16 + 2
}

fn hidden_toggle_width() -> u16 {
    "[ ] Mostrar arquivos ocultos".chars().count() as u16
}

/// Linhas visíveis dentro da lista (estimativa conservadora para scroll).
fn estimated_list_viewport() -> usize {
    let content_h = MAX_OUTER_HEIGHT.saturating_sub(4);
    let fixed = 1 + FIELD_HEIGHT + 1 + 1 + FIELD_HEIGHT;
    panel::inner_rect(Rect {
        x: 0,
        y: 0,
        width: 1,
        height: content_h.saturating_sub(fixed).max(LIST_MIN_ROWS),
    })
    .height
    .max(1) as usize
}

fn rect_contains(r: Rect, mouse: &MouseEvent) -> bool {
    mouse.column >= r.x
        && mouse.column < r.x.saturating_add(r.width)
        && mouse.row >= r.y
        && mouse.row < r.y.saturating_add(r.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parent_entry_navigates_on_enter() {
        let base = std::env::temp_dir().join(format!("edit-fb-parent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(base.join("sub")).unwrap();

        let mut modal = FileBrowserModal::new(
            FileBrowserMode::Open,
            base.join("sub"),
            String::new(),
            "*.*".to_string(),
            true,
        );
        assert_eq!(modal.entries.first().map(|e| e.name.as_str()), Some(".."));
        modal.list_cursor = 0;
        assert!(matches!(
            modal.activate_list_entry(),
            FileBrowserKeyResult::Consumed
        ));
        assert_eq!(modal.current_dir, base);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn resolved_path_ignores_directory_selection() {
        let base = std::env::temp_dir().join(format!("edit-fb-resolve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(base.join("sub")).unwrap();

        let mut modal = FileBrowserModal::new(
            FileBrowserMode::Open,
            base.clone(),
            String::new(),
            "*.*".to_string(),
            true,
        );
        let sub_idx = modal
            .entries
            .iter()
            .position(|e| e.name == "sub")
            .expect("subdir");
        modal.list_cursor = sub_idx;
        assert_eq!(modal.resolved_path(), base);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn enter_directory_updates_listing() {
        let base = std::env::temp_dir().join(format!("edit-fb-enter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(base.join("sub")).unwrap();
        fs::write(base.join("sub/a.txt"), b"hi").unwrap();

        let mut modal = FileBrowserModal::new(
            FileBrowserMode::Open,
            base.clone(),
            String::new(),
            "*.*".to_string(),
            true,
        );
        assert!(modal.entries.iter().any(|e| e.name == "sub"));
        let sub = base.join("sub");
        modal.enter_directory(sub);
        assert!(modal.entries.iter().any(|e| e.name == "a.txt"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn outer_rect_never_exceeds_max_height() {
        let modal = FileBrowserModal::new(
            FileBrowserMode::Open,
            std::env::temp_dir(),
            String::new(),
            "*.*".to_string(),
            false,
        );
        for term_h in [25u16, 30, 40, 80] {
            let area = Rect {
                x: 0,
                y: 0,
                width: 80,
                height: term_h,
            };
            assert!(
                modal.outer_rect(area).height <= MAX_OUTER_HEIGHT,
                "outer height on {term_h}-row terminal"
            );
            assert!(
                modal.total_footprint_rows(area) <= 25,
                "footprint on {term_h}-row terminal"
            );
        }
    }

    #[test]
    fn tab_cycles_all_focus_targets() {
        let modal = FileBrowserModal::new(
            FileBrowserMode::Open,
            std::env::temp_dir(),
            String::new(),
            "*.*".to_string(),
            false,
        );
        let mut modal = modal;
        modal.focus = FileBrowserFocus::Name;
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::List);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::Filter);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::HiddenToggle);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::PrimaryButton);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::CancelButton);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FileBrowserFocus::Name);
    }
}
