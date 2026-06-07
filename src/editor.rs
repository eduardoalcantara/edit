use ratatui::widgets::Block;
use tui_textarea::{Input, TextArea};

use crate::theme::ThemePalette;

pub struct Editor {
    textarea: TextArea<'static>,
}

impl Editor {
    pub fn new(palette: &ThemePalette) -> Self {
        let mut editor = Self {
            textarea: TextArea::default(),
        };
        editor
            .textarea
            .set_placeholder_text("Digite aqui para começar...");
        editor.apply_theme(palette);
        editor
    }

    pub fn apply_theme(&mut self, palette: &ThemePalette) {
        self.textarea.set_style(palette.editor_text_style());
        self.textarea.set_cursor_style(palette.cursor_style());
        self.textarea.set_selection_style(palette.selection_style());
        self.textarea
            .set_placeholder_style(palette.placeholder_style());
        self.textarea.set_block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(
                    ratatui::style::Style::default()
                        .fg(palette.border)
                        .bg(palette.editor_bg),
                )
                .title(" Editor ")
                .style(palette.editor_text_style()),
        );
    }

    pub fn handle_input(&mut self, input: Input) {
        self.textarea.input(input);
    }

    pub fn textarea(&self) -> &TextArea<'static> {
        &self.textarea
    }
}
