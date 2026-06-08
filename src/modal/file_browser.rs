mod listing;
mod path_resolve;

pub use listing::{FileEntry, FileEntryKind, format_metadata_line, list_directory};
pub use path_resolve::{
    infer_filter_from_path, initial_directory, resolve_open_target, resolve_save_target,
    suggest_file_name,
};

use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::modal::buttons::{FILE_BROWSER_OPEN, FILE_BROWSER_SAVE};
use crate::modal::dialog::{
    centered_dialog_rect, dialog_button_row_y, dialog_content_rect, hit_dialog_button,
    paint_dialog_buttons, paint_titled_dialog_content, Dialog, DialogKeyResult,
};
use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 18;
const LIST_MIN_ROWS: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserMode {
    Open,
    Save,
    SaveAs,
}

impl FileBrowserMode {
    pub fn title(self) -> &'static str {
        match self {
            FileBrowserMode::Open => "Open File",
            FileBrowserMode::Save | FileBrowserMode::SaveAs => "Save File As",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBrowserFocus {
    Name,
    List,
    Filter,
    HiddenToggle,
    Buttons,
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
    files_label: Rect,
    list_box: Rect,
    filter_label: Rect,
    filter_field: Rect,
    hidden_row: Rect,
    status_bar: Rect,
    button_y: u16,
}

#[derive(Debug, Clone)]
pub struct FileBrowserModal {
    pub dialog: Dialog,
    pub mode: FileBrowserMode,
    pub current_dir: PathBuf,
    pub name_input: String,
    pub filter_input: String,
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
            name_input,
            filter_input,
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
        match list_directory(&self.current_dir, &self.filter_input, self.show_hidden) {
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
        let selected = self.entries.get(self.list_cursor).map(|e| e.path.as_path());
        match self.mode {
            FileBrowserMode::Open => {
                resolve_open_target(&self.current_dir, &self.name_input, selected)
            }
            FileBrowserMode::Save | FileBrowserMode::SaveAs => {
                resolve_save_target(&self.current_dir, &self.name_input)
            }
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = area.width.saturating_mul(78).saturating_div(100).max(MIN_WIDTH);
        let height = area.height.saturating_mul(72).saturating_div(100).max(MIN_HEIGHT);
        centered_dialog_rect(area, width.min(area.width), height.min(area.height))
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = paint_titled_dialog_content(frame, area, self.mode.title(), palette);
        let layout = self.layout(content);

        self.paint_label(frame, layout.name_label, "Name", palette);
        self.paint_field(
            frame,
            layout.name_field,
            &self.name_input,
            self.focus == FileBrowserFocus::Name,
            palette,
        );

        self.paint_label(frame, layout.files_label, "Files", palette);
        self.paint_list(frame, layout.list_box, palette);

        self.paint_label(frame, layout.filter_label, "Filter", palette);
        self.paint_field(
            frame,
            layout.filter_field,
            &self.filter_input,
            self.focus == FileBrowserFocus::Filter,
            palette,
        );

        self.paint_hidden_toggle(frame, layout.hidden_row, palette);
        self.paint_status(frame, layout.status_bar, palette);

        paint_dialog_buttons(
            frame,
            content,
            layout.button_y,
            if self.focus == FileBrowserFocus::Buttons {
                self.dialog.selected
            } else {
                self.dialog.selected
            },
            self.dialog.buttons,
            palette,
        );
    }

    fn paint_label(&self, frame: &mut Frame<'_>, area: Rect, text: &str, palette: ThemePalette) {
        frame.render_widget(
            Paragraph::new(text).style(palette.status_style()),
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
        let inner = panel::render_titled_frame(
            frame,
            area,
            "",
            palette.editor_text_style(),
            if focused {
                palette.convert_field_border_style(true)
            } else {
                palette.menu_border_style()
            },
            palette.menu_panel_style(),
            false,
            PanelBorder::Plain,
        );
        let display = if focused {
            format!("{text}▌")
        } else {
            text.to_string()
        };
        frame.render_widget(
            Paragraph::new(display).style(palette.editor_text_style()),
            inner,
        );
    }

    fn paint_list(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        let inner = panel::render_titled_frame(
            frame,
            area,
            "",
            Style::default().bg(ratatui::style::Color::Cyan).fg(ratatui::style::Color::Black),
            if self.focus == FileBrowserFocus::List {
                palette.convert_field_border_style(true)
            } else {
                palette.menu_border_style()
            },
            Style::default().bg(ratatui::style::Color::Cyan),
            false,
            PanelBorder::Plain,
        );
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

    fn paint_status(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        let line1 = if self.status_msg.is_empty() {
            format!(
                "{} / {}",
                self.current_dir.display(),
                self.filter_input
            )
        } else {
            self.status_msg.clone()
        };
        let line2 = self
            .entries
            .get(self.list_cursor)
            .map(format_metadata_line)
            .unwrap_or_default();
        let text = vec![
            Line::from(Span::styled(line1, palette.editor_text_style())),
            Line::from(Span::styled(line2, palette.editor_text_style())),
        ];
        let inner = panel::render_titled_frame(
            frame,
            area,
            "",
            palette.editor_text_style(),
            palette.menu_border_style(),
            palette.menu_panel_style(),
            false,
            PanelBorder::Plain,
        );
        frame.render_widget(Paragraph::new(text), inner);
    }

    pub fn focused_help(&self) -> Option<&'static str> {
        match self.focus {
            FileBrowserFocus::Name => Some("Nome ou caminho do arquivo"),
            FileBrowserFocus::List => Some("↑/↓ navega; Enter abre pasta ou confirma arquivo"),
            FileBrowserFocus::Filter => Some("Máscara de arquivos, ex.: *.rs ou *.*"),
            FileBrowserFocus::HiddenToggle => Some("Mostra arquivos/pastas ocultos"),
            FileBrowserFocus::Buttons => self.dialog.focused_help(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
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
            FileBrowserFocus::Name => self.handle_name_key(key),
            FileBrowserFocus::List => self.handle_list_key(key),
            FileBrowserFocus::Filter => self.handle_filter_key(key),
            FileBrowserFocus::HiddenToggle => self.handle_hidden_key(key),
            FileBrowserFocus::Buttons => match self.dialog.handle_button_keys(key) {
                DialogKeyResult::Activate(_) => FileBrowserKeyResult::Submit,
                DialogKeyResult::Cancel => FileBrowserKeyResult::Cancel,
                DialogKeyResult::Consumed => FileBrowserKeyResult::Consumed,
                DialogKeyResult::Ignored => FileBrowserKeyResult::Consumed,
            },
        }
    }

    fn handle_name_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Enter => FileBrowserKeyResult::Submit,
            KeyCode::Backspace => {
                self.name_input.pop();
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.name_input.push(ch);
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Enter => {
                self.refresh_listing();
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Backspace => {
                self.filter_input.pop();
                FileBrowserKeyResult::Consumed
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter_input.push(ch);
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_hidden_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        match key.code {
            KeyCode::Tab | KeyCode::Enter | KeyCode::Char(' ') => {
                self.show_hidden = !self.show_hidden;
                self.refresh_listing();
                if matches!(key.code, KeyCode::Tab) {
                    self.cycle_focus(1);
                }
                FileBrowserKeyResult::Consumed
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                FileBrowserKeyResult::Consumed
            }
            _ => FileBrowserKeyResult::Consumed,
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> FileBrowserKeyResult {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                FileBrowserKeyResult::Consumed
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                FileBrowserKeyResult::Consumed
            }
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
                self.name_input = entry.name;
                if self.mode == FileBrowserMode::Open {
                    FileBrowserKeyResult::Submit
                } else {
                    FileBrowserKeyResult::Consumed
                }
            }
        }
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
                if rect_contains(layout.filter_field, mouse) {
                    self.focus = FileBrowserFocus::Filter;
                    return true;
                }
                if rect_contains(layout.hidden_row, mouse) {
                    self.focus = FileBrowserFocus::HiddenToggle;
                    if matches!(mouse.kind, MouseEventKind::Down(_)) {
                        self.show_hidden = !self.show_hidden;
                        self.refresh_listing();
                    }
                    return true;
                }
                if let Some(idx) = self.hit_list(mouse, layout.list_box) {
                    self.focus = FileBrowserFocus::List;
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
                    return true;
                }
                if let Some(idx) = hit_dialog_button(mouse, outer, self.dialog.buttons) {
                    self.focus = FileBrowserFocus::Buttons;
                    self.dialog.set_selected(idx);
                    return true;
                }
            }
            MouseEventKind::Moved => {
                if let Some(idx) = hit_dialog_button(mouse, outer, self.dialog.buttons) {
                    self.dialog.set_selected(idx);
                }
            }
            MouseEventKind::ScrollUp => {
                if rect_contains(layout.list_box, mouse) {
                    self.move_list_cursor(-3);
                    return true;
                }
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(layout.list_box, mouse) {
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
        self.focus == FileBrowserFocus::Buttons
            && self.dialog.selected_action()
                == Some(crate::modal::DialogButtonAction::Primary)
    }

    fn hit_list(&self, mouse: &MouseEvent, list_box: Rect) -> Option<usize> {
        let inner = panel::inner_rect(list_box);
        let inner = panel::inset_rect(inner, 1, 1, 1, 1);
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
        let mut y = content.y;
        let w = content.width;
        let name_label = Rect { x: content.x, y, width: w, height: 1 };
        y += 1;
        let name_field = Rect { x: content.x, y, width: w.saturating_sub(12), height: 3 };
        y += 3;
        y += 1;
        let files_label = Rect { x: content.x, y, width: w, height: 1 };
        y += 1;
        let list_height = content
            .height
            .saturating_sub(16)
            .max(LIST_MIN_ROWS);
        let list_box = Rect {
            x: content.x,
            y,
            width: w.saturating_sub(12),
            height: list_height,
        };
        y += list_height;
        y += 1;
        let filter_label = Rect { x: content.x, y, width: w, height: 1 };
        y += 1;
        let filter_field = Rect { x: content.x, y, width: w.saturating_sub(12), height: 3 };
        y += 3;
        let hidden_row = Rect { x: content.x, y, width: w, height: 1 };
        y += 1;
        let status_bar = Rect {
            x: content.x,
            y,
            width: w,
            height: 2,
        };
        Layout {
            name_label,
            name_field,
            files_label,
            list_box,
            filter_label,
            filter_field,
            hidden_row,
            status_bar,
            button_y: dialog_button_row_y(content),
        }
    }

    fn cycle_focus(&mut self, delta: i32) {
        let order = [
            FileBrowserFocus::Name,
            FileBrowserFocus::List,
            FileBrowserFocus::Filter,
            FileBrowserFocus::HiddenToggle,
            FileBrowserFocus::Buttons,
        ];
        let current = order
            .iter()
            .position(|&f| f == self.focus)
            .unwrap_or(0) as i32;
        let len = order.len() as i32;
        let next = (current + delta).rem_euclid(len) as usize;
        self.focus = order[next];
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
        let visible = LIST_MIN_ROWS as usize;
        if self.list_cursor < self.list_scroll {
            self.list_scroll = self.list_cursor;
        } else if self.list_cursor >= self.list_scroll.saturating_add(visible) {
            self.list_scroll = self.list_cursor + 1 - visible;
        }
    }

    fn sync_name_from_selection(&mut self) {
        if let Some(entry) = self.entries.get(self.list_cursor) {
            if entry.kind == FileEntryKind::File {
                self.name_input = entry.name.clone();
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
    fn enter_directory_updates_listing() {
        let base = std::env::temp_dir().join(format!("edit-fb-enter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
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
}
