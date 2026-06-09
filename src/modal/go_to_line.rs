use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::modal::dialog::{
    centered_dialog_rect, dialog_content_rect, dialog_outer_height, paint_titled_dialog_content,
    Dialog,
};
use crate::modal::form_controls::{
    layout_button_row, paint_button, paint_label, paint_text_input, rect_contains,
};
use crate::modal::text_input::{CharAccept, TextInput};
use crate::clipboard::Clipboard;
use crate::theme::ThemePalette;
use crate::widgets::panel;

const DIALOG_WIDTH: u16 = 44;
const FIELD_HEIGHT: u16 = 1;

const CONTENT_ROWS: u16 = 6;

const COMMANDS: &[&str] = &["Ir", "Fechar"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoToLineCommand {
    Go,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoToLineFocus {
    Line,
    Column,
    Command(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoToLineKeyResult {
    Consumed,
    Command(GoToLineCommand),
    Cancel,
}

#[derive(Debug, Clone)]
struct Layout {
    line_label: Rect,
    line_field: Rect,
    col_label: Rect,
    col_field: Rect,
    commands: Vec<(usize, &'static str, Rect)>,
}

#[derive(Debug, Clone)]
pub struct GoToLineModal {
    pub dialog: Dialog,
    pub line: TextInput,
    pub col: TextInput,
    pub focus: GoToLineFocus,
    pub pending_command: Option<GoToLineCommand>,
}

impl GoToLineModal {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            dialog: Dialog::form("Ir para linha", String::new(), &[]),
            line: TextInput::new(line.to_string()),
            col: TextInput::new(col.to_string()),
            focus: GoToLineFocus::Line,
            pending_command: None,
        }
    }

    fn command_at(index: usize) -> Option<GoToLineCommand> {
        match index {
            0 => Some(GoToLineCommand::Go),
            1 => Some(GoToLineCommand::Close),
            _ => None,
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = DIALOG_WIDTH.min(area.width);
        let height = dialog_outer_height(CONTENT_ROWS).min(area.height);
        centered_dialog_rect(area, width, height)
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = paint_titled_dialog_content(frame, area, "Ir para linha", palette);
        let layout = self.layout(content);

        paint_label(frame, layout.line_label, "Linha", palette);
        paint_text_input(
            frame,
            layout.line_field,
            &self.line,
            self.focus == GoToLineFocus::Line,
            palette,
        );
        paint_label(frame, layout.col_label, "Coluna (opcional)", palette);
        paint_text_input(
            frame,
            layout.col_field,
            &self.col,
            self.focus == GoToLineFocus::Column,
            palette,
        );

        for &(idx, label, btn) in &layout.commands {
            paint_button(
                frame,
                btn,
                label,
                self.focus == GoToLineFocus::Command(idx),
                palette,
            );
        }
    }

    pub fn focused_help(&self) -> Option<String> {
        let help = match self.focus {
            GoToLineFocus::Line => "Número da linha de destino (1 = primeira)",
            GoToLineFocus::Column => "Coluna opcional; vazio mantém a coluna atual",
            GoToLineFocus::Command(0) => "Move o cursor para linha e coluna informadas",
            GoToLineFocus::Command(1) => "Fecha o diálogo",
            GoToLineFocus::Command(_) => "Comando",
        };
        Some(help.to_string())
    }

    pub fn handle_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> GoToLineKeyResult {
        if key.code == KeyCode::Esc {
            return GoToLineKeyResult::Cancel;
        }
        match self.focus {
            GoToLineFocus::Line => self.handle_line_key(key, clipboard),
            GoToLineFocus::Column => self.handle_col_key(key, clipboard),
            GoToLineFocus::Command(i) => self.handle_command_key(key, i),
        }
    }

    fn handle_tab_navigation(&mut self, key: KeyEvent) -> Option<GoToLineKeyResult> {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                Some(GoToLineKeyResult::Consumed)
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                Some(GoToLineKeyResult::Consumed)
            }
            _ => None,
        }
    }

    fn handle_line_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> GoToLineKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            return GoToLineKeyResult::Command(GoToLineCommand::Go);
        }
        if self.line.handle_key(key, clipboard, CharAccept::AsciiDigit) {
            return GoToLineKeyResult::Consumed;
        }
        GoToLineKeyResult::Consumed
    }

    fn handle_col_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> GoToLineKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            return GoToLineKeyResult::Command(GoToLineCommand::Go);
        }
        if self.col.handle_key(key, clipboard, CharAccept::AsciiDigit) {
            return GoToLineKeyResult::Consumed;
        }
        GoToLineKeyResult::Consumed
    }

    fn handle_command_key(&mut self, key: KeyEvent, index: usize) -> GoToLineKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(cmd) = Self::command_at(index) {
                    GoToLineKeyResult::Command(cmd)
                } else {
                    GoToLineKeyResult::Consumed
                }
            }
            KeyCode::Left => {
                self.cycle_focus(-1);
                GoToLineKeyResult::Consumed
            }
            KeyCode::Right => {
                self.cycle_focus(1);
                GoToLineKeyResult::Consumed
            }
            _ => GoToLineKeyResult::Consumed,
        }
    }

    pub fn handle_mouse(&mut self, mouse: &MouseEvent, outer: Rect) -> bool {
        let layout = self.layout(dialog_content_rect(outer));
        match mouse.kind {
            MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                if rect_contains(layout.line_field, mouse) {
                    self.focus = GoToLineFocus::Line;
                    return true;
                }
                if rect_contains(layout.col_field, mouse) {
                    self.focus = GoToLineFocus::Column;
                    return true;
                }
                for &(idx, _, btn) in &layout.commands {
                    if rect_contains(btn, mouse) {
                        self.focus = GoToLineFocus::Command(idx);
                        if matches!(mouse.kind, MouseEventKind::Down(_)) {
                            self.pending_command = Self::command_at(idx);
                        }
                        return true;
                    }
                }
            }
            MouseEventKind::Moved => {
                for &(idx, _, btn) in &layout.commands {
                    if rect_contains(btn, mouse) {
                        self.focus = GoToLineFocus::Command(idx);
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn take_pending_command(&mut self) -> Option<GoToLineCommand> {
        self.pending_command.take()
    }

    fn layout(&self, content: Rect) -> Layout {
        let w = content.width;
        let mut y = content.y;

        let line_label = Rect {
            x: content.x,
            y,
            width: w,
            height: 1,
        };
        y += 1;
        let line_field = Rect {
            x: content.x,
            y,
            width: w,
            height: FIELD_HEIGHT,
        };
        y += FIELD_HEIGHT;

        let col_label = Rect {
            x: content.x,
            y,
            width: w,
            height: 1,
        };
        y += 1;
        let col_field = Rect {
            x: content.x,
            y,
            width: w,
            height: FIELD_HEIGHT,
        };
        y += FIELD_HEIGHT + 1;

        let commands = layout_button_row(content.x, y, w, COMMANDS);

        Layout {
            line_label,
            line_field,
            col_label,
            col_field,
            commands,
        }
    }

    fn cycle_focus(&mut self, delta: i32) {
        let order = [
            GoToLineFocus::Line,
            GoToLineFocus::Column,
            GoToLineFocus::Command(0),
            GoToLineFocus::Command(1),
        ];
        let current = order
            .iter()
            .position(|&f| f == self.focus)
            .unwrap_or(0) as i32;
        let len = order.len() as i32;
        let next = (current + delta).rem_euclid(len) as usize;
        self.focus = order[next];
    }
}
