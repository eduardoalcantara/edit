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

const FIND_WIDTH: u16 = 78;
const REPLACE_WIDTH: u16 = 58;
const FIELD_HEIGHT: u16 = 1;

const FIND_COMMANDS: &[&str] = &[
    "Buscar Próximo (F3)",
    "Anterior",
    "Primeiro",
    "Último",
    "Limpar",
    "Fechar",
];

const REPLACE_COMMANDS: &[&str] = &[
    "Substituir Próximo",
    "Substituir Tudo",
    "Limpar",
    "Fechar",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindReplaceCommand {
    FindNext,
    FindPrev,
    FindFirst,
    FindLast,
    ReplaceNext,
    ReplaceAll,
    Clear,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindReplaceFocus {
    Pattern,
    Replacement,
    Command(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindReplaceKeyResult {
    Consumed,
    Command(FindReplaceCommand),
    Cancel,
}

#[derive(Debug, Clone)]
struct Layout {
    pattern_label: Rect,
    pattern_field: Rect,
    replacement_label: Rect,
    replacement_field: Rect,
    commands: Vec<(usize, &'static str, Rect)>,
}

#[derive(Debug, Clone)]
pub struct FindReplaceModal {
    pub dialog: Dialog,
    pub pattern: TextInput,
    pub replacement: TextInput,
    pub replace_mode: bool,
    pub focus: FindReplaceFocus,
    pub pending_command: Option<FindReplaceCommand>,
}

impl FindReplaceModal {
    pub fn find(pattern: impl Into<String>) -> Self {
        Self {
            dialog: Dialog::form("Buscar", String::new(), &[]),
            pattern: TextInput::new(pattern),
            replacement: TextInput::new(""),
            replace_mode: false,
            focus: FindReplaceFocus::Pattern,
            pending_command: None,
        }
    }

    pub fn replace(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            dialog: Dialog::form("Substituir", String::new(), &[]),
            pattern: TextInput::new(pattern),
            replacement: TextInput::new(replacement),
            replace_mode: true,
            focus: FindReplaceFocus::Pattern,
            pending_command: None,
        }
    }

    fn dialog_width(&self) -> u16 {
        if self.replace_mode {
            REPLACE_WIDTH
        } else {
            FIND_WIDTH
        }
    }

    fn command_labels(&self) -> &'static [&'static str] {
        if self.replace_mode {
            REPLACE_COMMANDS
        } else {
            FIND_COMMANDS
        }
    }

    fn command_at(&self, index: usize) -> Option<FindReplaceCommand> {
        if self.replace_mode {
            match index {
                0 => Some(FindReplaceCommand::ReplaceNext),
                1 => Some(FindReplaceCommand::ReplaceAll),
                2 => Some(FindReplaceCommand::Clear),
                3 => Some(FindReplaceCommand::Close),
                _ => None,
            }
        } else {
            match index {
                0 => Some(FindReplaceCommand::FindNext),
                1 => Some(FindReplaceCommand::FindPrev),
                2 => Some(FindReplaceCommand::FindFirst),
                3 => Some(FindReplaceCommand::FindLast),
                4 => Some(FindReplaceCommand::Clear),
                5 => Some(FindReplaceCommand::Close),
                _ => None,
            }
        }
    }

    fn content_rows(&self) -> u16 {
        if self.replace_mode {
            6
        } else {
            3
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = self.dialog_width().min(area.width);
        let height = dialog_outer_height(self.content_rows()).min(area.height);
        centered_dialog_rect(area, width, height)
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let title = if self.replace_mode {
            "Substituir"
        } else {
            "Buscar"
        };
        let content = paint_titled_dialog_content(frame, area, title, palette);
        let layout = self.layout(content);

        if self.replace_mode {
            paint_label(frame, layout.pattern_label, "Texto", palette);
        }
        paint_text_input(
            frame,
            layout.pattern_field,
            &self.pattern,
            self.focus == FindReplaceFocus::Pattern,
            palette,
        );

        if self.replace_mode {
            paint_label(frame, layout.replacement_label, "Substituir por", palette);
            paint_text_input(
                frame,
                layout.replacement_field,
                &self.replacement,
                self.focus == FindReplaceFocus::Replacement,
                palette,
            );
        }

        for &(idx, label, btn) in &layout.commands {
            paint_button(
                frame,
                btn,
                label,
                self.focus == FindReplaceFocus::Command(idx),
                palette,
            );
        }
    }

    pub fn focused_help(&self) -> Option<String> {
        let help = match self.focus {
            FindReplaceFocus::Pattern => "Texto a buscar no documento",
            FindReplaceFocus::Replacement => "Texto que substitui a ocorrência encontrada",
            FindReplaceFocus::Command(i) => match self.command_at(i) {
                Some(FindReplaceCommand::FindNext) => {
                    "Busca a próxima ocorrência (F3 fora do modal)"
                }
                Some(FindReplaceCommand::FindPrev) => "Busca a ocorrência anterior",
                Some(FindReplaceCommand::FindFirst) => "Vai para a primeira ocorrência",
                Some(FindReplaceCommand::FindLast) => "Vai para a última ocorrência",
                Some(FindReplaceCommand::ReplaceNext) => "Substitui a próxima ocorrência",
                Some(FindReplaceCommand::ReplaceAll) => "Substitui todas as ocorrências",
                Some(FindReplaceCommand::Clear) => {
                    "Limpa os campos e remove o destaque no texto"
                }
                Some(FindReplaceCommand::Close) => "Fecha o diálogo",
                None => "Comando",
            },
        };
        Some(help.to_string())
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        clipboard: &mut Clipboard,
    ) -> FindReplaceKeyResult {
        if key.code == KeyCode::Esc {
            return FindReplaceKeyResult::Command(FindReplaceCommand::Close);
        }
        if key.code == KeyCode::F(3) && !self.replace_mode {
            return FindReplaceKeyResult::Command(FindReplaceCommand::FindNext);
        }

        match self.focus {
            FindReplaceFocus::Pattern => self.handle_pattern_key(key, clipboard),
            FindReplaceFocus::Replacement => self.handle_replacement_key(key, clipboard),
            FindReplaceFocus::Command(i) => self.handle_command_key(key, i),
        }
    }

    fn handle_tab_navigation(&mut self, key: KeyEvent) -> Option<FindReplaceKeyResult> {
        match key.code {
            KeyCode::Tab => {
                self.cycle_focus(1);
                Some(FindReplaceKeyResult::Consumed)
            }
            KeyCode::BackTab => {
                self.cycle_focus(-1);
                Some(FindReplaceKeyResult::Consumed)
            }
            _ => None,
        }
    }

    fn handle_pattern_key(
        &mut self,
        key: KeyEvent,
        clipboard: &mut Clipboard,
    ) -> FindReplaceKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            return FindReplaceKeyResult::Command(if self.replace_mode {
                FindReplaceCommand::ReplaceNext
            } else {
                FindReplaceCommand::FindNext
            });
        }
        if self.pattern.handle_key(key, clipboard, CharAccept::Any) {
            return FindReplaceKeyResult::Consumed;
        }
        FindReplaceKeyResult::Consumed
    }

    fn handle_replacement_key(
        &mut self,
        key: KeyEvent,
        clipboard: &mut Clipboard,
    ) -> FindReplaceKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        if key.code == KeyCode::Enter {
            return FindReplaceKeyResult::Command(FindReplaceCommand::ReplaceNext);
        }
        if self.replacement.handle_key(key, clipboard, CharAccept::Any) {
            return FindReplaceKeyResult::Consumed;
        }
        FindReplaceKeyResult::Consumed
    }

    fn handle_command_key(&mut self, key: KeyEvent, index: usize) -> FindReplaceKeyResult {
        if let Some(result) = self.handle_tab_navigation(key) {
            return result;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(cmd) = self.command_at(index) {
                    FindReplaceKeyResult::Command(cmd)
                } else {
                    FindReplaceKeyResult::Consumed
                }
            }
            KeyCode::Left => {
                self.cycle_focus(-1);
                FindReplaceKeyResult::Consumed
            }
            KeyCode::Right => {
                self.cycle_focus(1);
                FindReplaceKeyResult::Consumed
            }
            _ => FindReplaceKeyResult::Consumed,
        }
    }

    pub fn handle_mouse(&mut self, mouse: &MouseEvent, outer: Rect) -> bool {
        let content = dialog_content_rect(outer);
        let layout = self.layout(content);
        match mouse.kind {
            MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                if rect_contains(layout.pattern_field, mouse) {
                    self.focus = FindReplaceFocus::Pattern;
                    return true;
                }
                if self.replace_mode && rect_contains(layout.replacement_field, mouse) {
                    self.focus = FindReplaceFocus::Replacement;
                    return true;
                }
                for &(idx, _, btn) in &layout.commands {
                    if rect_contains(btn, mouse) {
                        self.focus = FindReplaceFocus::Command(idx);
                        if matches!(mouse.kind, MouseEventKind::Down(_)) {
                            self.pending_command = self.command_at(idx);
                        }
                        return true;
                    }
                }
            }
            MouseEventKind::Moved => {
                for &(idx, _, btn) in &layout.commands {
                    if rect_contains(btn, mouse) {
                        self.focus = FindReplaceFocus::Command(idx);
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn take_pending_command(&mut self) -> Option<FindReplaceCommand> {
        self.pending_command.take()
    }

    pub fn hit_command(&self, mouse: &MouseEvent, outer: Rect) -> Option<FindReplaceCommand> {
        let layout = self.layout(dialog_content_rect(outer));
        for &(idx, _, btn) in &layout.commands {
            if rect_contains(btn, mouse) {
                return self.command_at(idx);
            }
        }
        None
    }

    fn layout(&self, content: Rect) -> Layout {
        let w = content.width;
        let mut y = content.y;

        let (pattern_label, pattern_field, replacement_label, replacement_field) =
            if self.replace_mode {
                let pattern_label = Rect {
                    x: content.x,
                    y,
                    width: w,
                    height: 1,
                };
                y += 1;
                let pattern_field = Rect {
                    x: content.x,
                    y,
                    width: w,
                    height: FIELD_HEIGHT,
                };
                y += FIELD_HEIGHT;
                let replacement_label = Rect {
                    x: content.x,
                    y,
                    width: w,
                    height: 1,
                };
                y += 1;
                let replacement_field = Rect {
                    x: content.x,
                    y,
                    width: w,
                    height: FIELD_HEIGHT,
                };
                y += FIELD_HEIGHT + 1;
                (pattern_label, pattern_field, replacement_label, replacement_field)
            } else {
                let pattern_field = Rect {
                    x: content.x,
                    y,
                    width: w,
                    height: FIELD_HEIGHT,
                };
                y += FIELD_HEIGHT + 1;
                (
                    Rect::default(),
                    pattern_field,
                    Rect::default(),
                    Rect::default(),
                )
            };

        let commands = layout_button_row(content.x, y, w, self.command_labels());

        Layout {
            pattern_label,
            pattern_field,
            replacement_label,
            replacement_field,
            commands,
        }
    }

    fn cycle_focus(&mut self, delta: i32) {
        let order: Vec<FindReplaceFocus> = if self.replace_mode {
            vec![
                FindReplaceFocus::Pattern,
                FindReplaceFocus::Replacement,
                FindReplaceFocus::Command(0),
                FindReplaceFocus::Command(1),
                FindReplaceFocus::Command(2),
                FindReplaceFocus::Command(3),
            ]
        } else {
            vec![
                FindReplaceFocus::Pattern,
                FindReplaceFocus::Command(0),
                FindReplaceFocus::Command(1),
                FindReplaceFocus::Command(2),
                FindReplaceFocus::Command(3),
                FindReplaceFocus::Command(4),
                FindReplaceFocus::Command(5),
            ]
        };
        let current = order
            .iter()
            .position(|&f| f == self.focus)
            .unwrap_or(0) as i32;
        let len = order.len() as i32;
        let next = (current + delta).rem_euclid(len) as usize;
        self.focus = order[next];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modal::dialog::dialog_content_rect;

    #[test]
    fn find_fits_command_row() {
        let modal = FindReplaceModal::find("");
        let outer = modal.outer_rect(Rect::new(0, 0, FIND_WIDTH, 24));
        let content = dialog_content_rect(outer);
        let layout = modal.layout(content);
        assert_eq!(layout.commands.len(), FIND_COMMANDS.len());
        assert!(
            layout.commands[0].2.y > layout.pattern_field.y,
            "comandos devem ficar abaixo do campo"
        );
    }

    #[test]
    fn replace_taller_than_find() {
        let find = FindReplaceModal::find("");
        let replace = FindReplaceModal::replace("", "");
        assert!(
            replace.outer_rect(Rect::new(0, 0, 80, 24)).height
                > find.outer_rect(Rect::new(0, 0, 80, 24)).height
        );
    }

    #[test]
    fn tab_cycles_replace_fields() {
        let mut modal = FindReplaceModal::replace("a", "b");
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FindReplaceFocus::Replacement);
        modal.cycle_focus(1);
        assert_eq!(modal.focus, FindReplaceFocus::Command(0));
    }
}
