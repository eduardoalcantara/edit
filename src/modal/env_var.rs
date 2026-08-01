//! Modal Novo/Editar variável de ambiente.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::clipboard::Clipboard;
use crate::modal::dialog::{
    centered_dialog_rect, dialog_outer_height, paint_titled_dialog_content, Dialog,
};
use crate::modal::form_controls::{
    layout_button_row, paint_button, paint_label, paint_text_input, rect_contains,
};
use crate::modal::text_input::{CharAccept, TextInput};
use crate::theme::ThemePalette;
use crate::widgets::panel;

const DIALOG_WIDTH: u16 = 56;
const CONTENT_ROWS: u16 = 7;
const COMMANDS: &[&str] = &["Sim", "Cancelar"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarCommand {
    Ok,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarFocus {
    Name,
    Value,
    Command(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarKeyResult {
    Consumed,
    Command(EnvVarCommand),
    Cancel,
}

#[derive(Debug, Clone)]
struct Layout {
    name_label: Rect,
    name_field: Rect,
    value_label: Rect,
    value_field: Rect,
    commands: Vec<(usize, &'static str, Rect)>,
}

#[derive(Debug, Clone)]
pub struct EnvVarModal {
    pub dialog: Dialog,
    pub is_new: bool,
    pub name: TextInput,
    pub value: TextInput,
    pub focus: EnvVarFocus,
}

impl EnvVarModal {
    pub fn new(is_new: bool, name: String, value: String) -> Self {
        let title = if is_new {
            "Nova variável"
        } else {
            "Editar variável"
        };
        Self {
            dialog: Dialog::form(title, String::new(), &[]),
            is_new,
            name: TextInput::new(name),
            value: TextInput::new(value),
            focus: EnvVarFocus::Name,
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = DIALOG_WIDTH.min(area.width);
        let height = dialog_outer_height(CONTENT_ROWS).min(area.height);
        centered_dialog_rect(area, width, height)
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let title = if self.is_new {
            "Nova variável"
        } else {
            "Editar variável"
        };
        let content = paint_titled_dialog_content(frame, area, title, palette);
        let layout = self.layout(content);
        paint_label(frame, layout.name_label, "Nome", palette);
        paint_text_input(
            frame,
            layout.name_field,
            &self.name,
            self.focus == EnvVarFocus::Name,
            palette,
        );
        paint_label(frame, layout.value_label, "Valor", palette);
        paint_text_input(
            frame,
            layout.value_field,
            &self.value,
            self.focus == EnvVarFocus::Value,
            palette,
        );
        for &(idx, label, btn) in &layout.commands {
            paint_button(
                frame,
                btn,
                label,
                self.focus == EnvVarFocus::Command(idx),
                palette,
            );
        }
    }

    pub fn focused_help(&self) -> Option<String> {
        match self.focus {
            EnvVarFocus::Name => Some("Nome da variável (A-Z, 0-9, _)".into()),
            EnvVarFocus::Value => Some("Valor (secrets ficam visíveis)".into()),
            EnvVarFocus::Command(0) => Some("Confirma e aplica na lista (ainda não salva no disco)".into()),
            EnvVarFocus::Command(_) => Some("Cancela sem alterar".into()),
        }
    }

    fn layout(&self, content: Rect) -> Layout {
        let name_label = Rect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: 1,
        };
        let name_field = Rect {
            x: content.x,
            y: content.y.saturating_add(1),
            width: content.width,
            height: 1,
        };
        let value_label = Rect {
            x: content.x,
            y: content.y.saturating_add(3),
            width: content.width,
            height: 1,
        };
        let value_field = Rect {
            x: content.x,
            y: content.y.saturating_add(4),
            width: content.width,
            height: 1,
        };
        let btn_y = content.y.saturating_add(6);
        let commands = layout_button_row(content.x, btn_y, content.width, COMMANDS);
        Layout {
            name_label,
            name_field,
            value_label,
            value_field,
            commands,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, clipboard: &mut Clipboard) -> EnvVarKeyResult {
        use crossterm::event::KeyEventKind;
        if key.kind != KeyEventKind::Press {
            return EnvVarKeyResult::Consumed;
        }
        match key.code {
            KeyCode::Esc => EnvVarKeyResult::Cancel,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    EnvVarFocus::Name => EnvVarFocus::Value,
                    EnvVarFocus::Value => EnvVarFocus::Command(0),
                    EnvVarFocus::Command(0) => EnvVarFocus::Command(1),
                    EnvVarFocus::Command(_) => EnvVarFocus::Name,
                };
                EnvVarKeyResult::Consumed
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    EnvVarFocus::Name => EnvVarFocus::Command(1),
                    EnvVarFocus::Value => EnvVarFocus::Name,
                    EnvVarFocus::Command(0) => EnvVarFocus::Value,
                    EnvVarFocus::Command(_) => EnvVarFocus::Command(0),
                };
                EnvVarKeyResult::Consumed
            }
            KeyCode::Enter => match self.focus {
                EnvVarFocus::Command(0) | EnvVarFocus::Name | EnvVarFocus::Value => {
                    EnvVarKeyResult::Command(EnvVarCommand::Ok)
                }
                EnvVarFocus::Command(_) => EnvVarKeyResult::Command(EnvVarCommand::Cancel),
            },
            KeyCode::Left | KeyCode::Right if matches!(self.focus, EnvVarFocus::Command(_)) => {
                self.focus = match self.focus {
                    EnvVarFocus::Command(0) => EnvVarFocus::Command(1),
                    _ => EnvVarFocus::Command(0),
                };
                EnvVarKeyResult::Consumed
            }
            _ => {
                match self.focus {
                    EnvVarFocus::Name => {
                        self.name.handle_key(key, clipboard, CharAccept::Any);
                    }
                    EnvVarFocus::Value => {
                        self.value.handle_key(key, clipboard, CharAccept::Any);
                    }
                    EnvVarFocus::Command(_) => {}
                }
                EnvVarKeyResult::Consumed
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: &MouseEvent, area: Rect) -> EnvVarKeyResult {
        let content = dialog_content_approx(area);
        let layout = self.layout(content);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) {
            return EnvVarKeyResult::Consumed;
        }
        if rect_contains(layout.name_field, mouse) {
            self.focus = EnvVarFocus::Name;
        } else if rect_contains(layout.value_field, mouse) {
            self.focus = EnvVarFocus::Value;
        } else {
            for &(idx, _, btn) in &layout.commands {
                if rect_contains(btn, mouse) {
                    self.focus = EnvVarFocus::Command(idx);
                    return if idx == 0 {
                        EnvVarKeyResult::Command(EnvVarCommand::Ok)
                    } else {
                        EnvVarKeyResult::Command(EnvVarCommand::Cancel)
                    };
                }
            }
        }
        EnvVarKeyResult::Consumed
    }
}

fn dialog_content_approx(outer: Rect) -> Rect {
    Rect {
        x: outer.x.saturating_add(2),
        y: outer.y.saturating_add(2),
        width: outer.width.saturating_sub(4),
        height: outer.height.saturating_sub(4),
    }
}
