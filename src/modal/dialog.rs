//! Shell reutilizável de diálogos modais (layout, botões, mouse, rodapé).
//!
//! Novos modais declaram título, corpo e botões com ajuda; render e input ficam aqui.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder, DIALOG_MARGIN};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogButtonAction {
    Primary,
    Secondary,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogButton {
    pub label: &'static str,
    pub help: &'static str,
    pub action: DialogButtonAction,
}

impl DialogButton {
    pub const fn new(label: &'static str, help: &'static str, action: DialogButtonAction) -> Self {
        Self {
            label,
            help,
            action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogSize {
    /// Altura proporcional fixa (formulários).
    Form,
    /// Altura calculada pelo texto do corpo.
    Message,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub title: String,
    pub body: String,
    pub buttons: &'static [DialogButton],
    pub size: DialogSize,
    pub selected: usize,
}

impl Dialog {
    pub fn message(
        title: impl Into<String>,
        body: impl Into<String>,
        buttons: &'static [DialogButton],
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            buttons,
            size: DialogSize::Message,
            selected: 0,
        }
    }

    pub fn form(
        title: impl Into<String>,
        body: impl Into<String>,
        buttons: &'static [DialogButton],
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            buttons,
            size: DialogSize::Form,
            selected: 0,
        }
    }

    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }

    pub fn selected_button(&self) -> Option<&DialogButton> {
        self.buttons.get(self.selected)
    }

    pub fn selected_action(&self) -> Option<DialogButtonAction> {
        self.selected_button().map(|button| button.action)
    }

    pub fn focused_help(&self) -> Option<&'static str> {
        self.selected_button().map(|button| button.help)
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.buttons.len() {
            self.selected = index;
        }
    }

    pub fn navigate_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn navigate_next(&mut self) {
        if self.buttons.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.buttons.len();
    }

    pub fn handle_button_keys(&mut self, key: KeyEvent) -> DialogKeyResult {
        match key.code {
            KeyCode::Left => {
                self.navigate_prev();
                DialogKeyResult::Consumed
            }
            KeyCode::Right | KeyCode::Tab => {
                self.navigate_next();
                DialogKeyResult::Consumed
            }
            KeyCode::Enter => DialogKeyResult::Activate(self.selected),
            KeyCode::Esc => DialogKeyResult::Cancel,
            _ => DialogKeyResult::Ignored,
        }
    }

    pub fn outer_rect(&self, frame: Rect) -> Rect {
        match self.size {
            DialogSize::Message => message_dialog_rect(frame, &self.body),
            DialogSize::Form => form_dialog_rect(frame),
        }
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = draw_titled_content(frame, area, &self.title, palette);
        let body_lines = wrapped_line_count(&self.body, content.width as usize);
        let max_body_h = match self.size {
            DialogSize::Message => content.height as usize,
            DialogSize::Form => content.height.saturating_sub(2) as usize,
        };
        let body_height = body_lines.min(max_body_h).max(1) as u16;
        frame.render_widget(
            Paragraph::new(self.body.as_str())
                .style(palette.menu_panel_style())
                .wrap(Wrap { trim: true }),
            Rect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: body_height,
            },
        );
        let button_y = content.y.saturating_add(content.height.saturating_sub(1));
        paint_dialog_buttons(frame, content, button_y, self.selected, self.buttons, palette);
    }

    pub fn hit_button(&self, mouse: &MouseEvent, dialog: Rect) -> Option<usize> {
        hit_dialog_button(mouse, dialog, self.buttons)
    }
}

pub(crate) fn paint_titled_dialog_content(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    palette: ThemePalette,
) -> Rect {
    draw_titled_content(frame, area, title, palette)
}

pub(crate) fn dialog_content_rect(area: Rect) -> Rect {
    content_rect(area)
}

pub(crate) fn dialog_button_row_y(content: Rect) -> u16 {
    button_row_y(content)
}

pub(crate) fn centered_dialog_rect(area: Rect, width: u16, height: u16) -> Rect {
    centered_rect_fixed(area, width, height)
}

pub(crate) fn hit_dialog_button(
    mouse: &MouseEvent,
    dialog: Rect,
    buttons: &[DialogButton],
) -> Option<usize> {
    hit_button(mouse, dialog, buttons)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKeyResult {
    Ignored,
    Consumed,
    Activate(usize),
    Cancel,
}

fn draw_titled_content(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    palette: ThemePalette,
) -> Rect {
    let inner = panel::render_titled_frame(
        frame,
        area,
        title,
        palette.menu_panel_style(),
        palette.menu_border_style(),
        palette.dialog_title_style(),
        true,
        PanelBorder::Double,
    );
    panel::fill_rect(frame, inner, palette.menu_panel_style());
    let (top, bottom, left, right) = DIALOG_MARGIN;
    panel::inset_rect(inner, top, bottom, left, right)
}

pub(crate) fn paint_dialog_buttons(
    frame: &mut Frame<'_>,
    content: Rect,
    y: u16,
    selected: usize,
    buttons: &[DialogButton],
    palette: ThemePalette,
) {
    let mut x = content.x;
    for (i, button) in buttons.iter().enumerate() {
        let width = button.label.chars().count() as u16 + 2;
        let area = Rect {
            x,
            y,
            width,
            height: 1,
        };
        let focused = i == selected;
        let text = format!("[{}]", button.label);
        frame.render_widget(
            Paragraph::new(text).style(palette.button_style(focused)),
            area,
        );
        x = x.saturating_add(width.saturating_add(1));
    }
}

fn content_rect(area: Rect) -> Rect {
    let inner = panel::inner_rect(area);
    let (top, bottom, left, right) = DIALOG_MARGIN;
    panel::inset_rect(inner, top, bottom, left, right)
}

fn button_row_y(content: Rect) -> u16 {
    content.y.saturating_add(content.height.saturating_sub(1))
}

fn dialog_width(area: Rect) -> u16 {
    area.width.saturating_mul(62).saturating_div(100).max(30)
}

fn message_dialog_rect(area: Rect, body: &str) -> Rect {
    let width = dialog_width(area);
    let content_w = width.saturating_sub(6) as usize;
    let body_lines = wrapped_line_count(body, content_w).max(1);
    let inner_h = (body_lines as u16 + 4).max(5);
    let height = inner_h.saturating_add(2);
    centered_rect_fixed(area, width, height)
}

fn form_dialog_rect(area: Rect) -> Rect {
    centered_rect(62, 28, area)
}

fn centered_rect_fixed(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(area.width.saturating_sub(width) / 2),
        y: area.y.saturating_add(area.height.saturating_sub(height) / 2),
        width,
        height,
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn wrapped_line_count(text: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let mut lines = 1usize;
    let mut col = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            lines += 1;
            col = 0;
            continue;
        }
        col += 1;
        if col >= width {
            lines += 1;
            col = 0;
        }
    }
    lines.max(1)
}

fn hit_button(mouse: &MouseEvent, dialog: Rect, buttons: &[DialogButton]) -> Option<usize> {
    if buttons.is_empty() {
        return None;
    }
    let content = content_rect(dialog);
    let row_y = button_row_y(content);
    if mouse.row != row_y {
        return None;
    }
    let mut x = content.x;
    for (i, button) in buttons.iter().enumerate() {
        let width = button.label.chars().count() as u16 + 2;
        if mouse.column >= x && mouse.column < x.saturating_add(width) {
            return Some(i);
        }
        x = x.saturating_add(width.saturating_add(1));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::MouseEventKind;

    const SAMPLE: [DialogButton; 3] = [
        DialogButton::new("Salvar", "Salva o documento", DialogButtonAction::Primary),
        DialogButton::new("Não Salvar", "Descarta alterações", DialogButtonAction::Secondary),
        DialogButton::new("Cancelar", "Volta ao editor", DialogButtonAction::Cancel),
    ];

    #[test]
    fn focused_help_follows_selection() {
        let mut dialog = Dialog::message("Sair", "msg", &SAMPLE);
        assert_eq!(dialog.focused_help(), Some("Salva o documento"));
        dialog.set_selected(1);
        assert_eq!(dialog.focused_help(), Some("Descarta alterações"));
    }

    #[test]
    fn hit_button_matches_rendered_row() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        let dialog = Dialog::message("Sair", "Sair sem salvar o arquivo test.txt?", &SAMPLE);
        let outer = dialog.outer_rect(area);
        let content = content_rect(outer);
        let row_y = button_row_y(content);

        let mut x = content.x;
        for (idx, button) in SAMPLE.iter().enumerate() {
            let width = button.label.chars().count() as u16 + 2;
            let mouse = MouseEvent {
                kind: MouseEventKind::Moved,
                column: x + 1,
                row: row_y,
                modifiers: crossterm::event::KeyModifiers::empty(),
            };
            assert_eq!(dialog.hit_button(&mouse, outer), Some(idx));
            x = x.saturating_add(width.saturating_add(1));
        }
    }

    #[test]
    fn navigate_wraps_buttons() {
        let mut dialog = Dialog::message("t", "b", &SAMPLE);
        dialog.set_selected(2);
        dialog.navigate_next();
        assert_eq!(dialog.selected, 0);
    }
}
