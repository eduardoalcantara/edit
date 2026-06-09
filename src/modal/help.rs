use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::modal::buttons::HELP_CLOSE;
use crate::modal::dialog::{
    centered_dialog_rect, dialog_button_row_y, hit_dialog_button,
    paint_dialog_buttons, paint_titled_dialog_content, Dialog, DialogKeyResult,
};
use crate::modal::help_content::{about_text, features_text, shortcuts_text};
use crate::theme::ThemePalette;
use crate::widgets::panel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpKind {
    Features,
    Shortcuts,
    About,
}

impl HelpKind {
    fn title(self) -> &'static str {
        match self {
            HelpKind::Features => "Funcionalidades",
            HelpKind::Shortcuts => "Atalhos",
            HelpKind::About => "Sobre",
        }
    }

    fn body(self) -> String {
        match self {
            HelpKind::Features => features_text().to_string(),
            HelpKind::Shortcuts => shortcuts_text().to_string(),
            HelpKind::About => about_text(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpKeyResult {
    Consumed,
    Close,
}

#[derive(Debug, Clone)]
pub struct HelpModal {
    pub dialog: Dialog,
    pub kind: HelpKind,
    pub scroll: usize,
    pub lines: Vec<String>,
}

impl HelpModal {
    pub fn new(kind: HelpKind) -> Self {
        let body = kind.body();
        let lines: Vec<String> = body.lines().map(str::to_string).collect();
        Self {
            dialog: Dialog::form(kind.title(), String::new(), &HELP_CLOSE),
            kind,
            scroll: 0,
            lines,
        }
    }

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let width = area.width.saturating_mul(70).saturating_div(100).max(50);
        let height = area.height.saturating_mul(65).saturating_div(100).max(16);
        centered_dialog_rect(area, width.min(area.width), height.min(area.height))
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = paint_titled_dialog_content(frame, area, self.kind.title(), palette);
        let body_h = content.height.saturating_sub(3).max(1);
        let visible = body_h as usize;
        let end = (self.scroll + visible).min(self.lines.len());
        let slice = self.lines.get(self.scroll..end).unwrap_or(&[]);
        let text = slice.join("\n");
        frame.render_widget(
            Paragraph::new(text)
                .style(palette.menu_panel_style())
                .wrap(Wrap { trim: false }),
            Rect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: body_h,
            },
        );
        let button_y = dialog_button_row_y(content);
        paint_dialog_buttons(
            frame,
            content,
            button_y,
            self.dialog.selected,
            self.dialog.buttons,
            palette,
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> HelpKeyResult {
        if key.code == KeyCode::Esc {
            return HelpKeyResult::Close;
        }
        match key.code {
            KeyCode::PageUp | KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(5);
                return HelpKeyResult::Consumed;
            }
            KeyCode::PageDown | KeyCode::Down
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.scroll = (self.scroll + 5).min(self.lines.len().saturating_sub(1));
                return HelpKeyResult::Consumed;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(8);
                HelpKeyResult::Consumed
            }
            KeyCode::PageDown => {
                self.scroll = (self.scroll + 8).min(self.lines.len().saturating_sub(1));
                HelpKeyResult::Consumed
            }
            _ => match self.dialog.handle_button_keys(key) {
                DialogKeyResult::Activate(_) | DialogKeyResult::Cancel => HelpKeyResult::Close,
                DialogKeyResult::Consumed => HelpKeyResult::Consumed,
                DialogKeyResult::Ignored => HelpKeyResult::Consumed,
            },
        }
    }

    pub fn hit_button(&self, mouse: &MouseEvent, outer: Rect) -> Option<usize> {
        hit_dialog_button(mouse, outer, self.dialog.buttons)
    }

    pub fn focused_help(&self) -> Option<&'static str> {
        self.dialog.focused_help()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_contains_version() {
        let modal = HelpModal::new(HelpKind::About);
        let body = HelpKind::About.body();
        assert!(body.contains(env!("CARGO_PKG_VERSION")));
        assert!(!modal.lines.is_empty());
    }
}
