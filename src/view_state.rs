use crate::theme::ThemeId;
use ratatui::style::{Color, Modifier, Style};

use crate::theme::ThemePalette;

/// Onde o teclado principal é entregue (editor de texto vs painel terminal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputFocus {
    #[default]
    Editor,
    Terminal,
}

/// Visibilidade da borda externa do editor de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorBorder {
    #[default]
    Visible,
    Hidden,
}

impl EditorBorder {
    pub fn label(self) -> &'static str {
        match self {
            EditorBorder::Visible => "Visível",
            EditorBorder::Hidden => "Invisível",
        }
    }
}

/// Margem interna entre a borda do editor e a área de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMargin {
    #[default]
    None,
    OneLine,
    TwoLines,
}

impl EditorMargin {
    pub fn label(self) -> &'static str {
        match self {
            EditorMargin::None => "Sem Margem",
            EditorMargin::OneLine => "Uma linha",
            EditorMargin::TwoLines => "Duas linhas",
        }
    }

    /// (topo, baixo, esquerda, direita) em linhas/colunas de célula.
    pub fn insets(self) -> (usize, usize, usize, usize) {
        match self {
            EditorMargin::None => (0, 0, 0, 0),
            EditorMargin::OneLine => (1, 1, 2, 2),
            EditorMargin::TwoLines => (2, 2, 4, 4),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideColumn {
    Col80,
    Col120,
    Col160,
    Unlimited,
}

impl GuideColumn {
    pub fn label(self) -> &'static str {
        match self {
            GuideColumn::Col80 => "80",
            GuideColumn::Col120 => "120",
            GuideColumn::Col160 => "160",
            GuideColumn::Unlimited => "∞",
        }
    }

    pub fn column(self) -> Option<usize> {
        match self {
            GuideColumn::Col80 => Some(80),
            GuideColumn::Col120 => Some(120),
            GuideColumn::Col160 => Some(160),
            GuideColumn::Unlimited => None,
        }
    }
}

/// Cores do output PTY (área dentro da moldura, sem bordas/divisor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerminalColorScheme {
    #[default]
    Theme,
    Classic,
}

impl TerminalColorScheme {
    pub fn toggle(self) -> Self {
        match self {
            Self::Theme => Self::Classic,
            Self::Classic => Self::Theme,
        }
    }

    pub fn config_key(self) -> &'static str {
        match self {
            Self::Theme => "tema",
            Self::Classic => "classico",
        }
    }

    pub fn status_label(self) -> &'static str {
        match self {
            Self::Theme => "cores do tema",
            Self::Classic => "clássico (preto/cinza)",
        }
    }

    pub fn output_styles(self, palette: ThemePalette) -> (Style, Style) {
        match self {
            Self::Theme => {
                let text = Style::default()
                    .fg(palette.editor_fg)
                    .bg(palette.editor_bg);
                let sel = Style::default()
                    .fg(palette.editor_bg)
                    .bg(palette.editor_fg)
                    .add_modifier(Modifier::BOLD);
                (text, sel)
            }
            Self::Classic => {
                let text = Style::default().fg(Color::from_u32(0x00c0c0c0)).bg(Color::Black);
                let sel = Style::default()
                    .fg(Color::Black)
                    .bg(Color::from_u32(0x00c0c0c0))
                    .add_modifier(Modifier::BOLD);
                (text, sel)
            }
        }
    }
}

pub fn parse_terminal_color_scheme(raw: &str) -> TerminalColorScheme {
    match raw.trim().to_lowercase().as_str() {
        "classico" | "classic" | "preto" => TerminalColorScheme::Classic,
        _ => TerminalColorScheme::Theme,
    }
}

#[derive(Debug, Clone)]
pub struct ViewState {
    pub word_wrap: bool,
    pub show_symbols: bool,
    pub show_spaces: bool,
    pub show_tabs: bool,
    pub show_eol: bool,
    pub terminal: bool,
    /// Altura do painel terminal em linhas de conteúdo (7–11); persistido em `edit.json`.
    pub terminal_panel_rows: u16,
    /// Cores do output PTY (`tema` ou `classico` em `edit.json`).
    pub terminal_color_scheme: TerminalColorScheme,
    pub footer_visible: bool,
    pub show_memory: bool,
    pub show_line_numbers: bool,
    pub guide_column: GuideColumn,
    pub margin: EditorMargin,
    pub border: EditorBorder,
    pub theme: ThemeId,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            word_wrap: false,
            show_symbols: false,
            show_spaces: false,
            show_tabs: false,
            show_eol: false,
            terminal: false,
            terminal_panel_rows: crate::terminal::TERMINAL_PANEL_ROWS_DEFAULT,
            terminal_color_scheme: TerminalColorScheme::default(),
            footer_visible: true,
            show_memory: true,
            show_line_numbers: false,
            guide_column: GuideColumn::Unlimited,
            margin: EditorMargin::None,
            border: EditorBorder::Visible,
            theme: ThemeId::Dark,
        }
    }
}

impl ViewState {
    pub fn show_all(&mut self, on: bool) {
        self.show_symbols = on;
        self.show_spaces = on;
        self.show_tabs = on;
        self.show_eol = on;
    }

    pub fn status_flags(&self) -> String {
        let wrap = if self.word_wrap { "on" } else { "off" };
        format!(
            "Wrap:{wrap} Col:{}",
            self.guide_column.label(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_insets() {
        assert_eq!(EditorMargin::None.insets(), (0, 0, 0, 0));
        assert_eq!(EditorMargin::OneLine.insets(), (1, 1, 2, 2));
        assert_eq!(EditorMargin::TwoLines.insets(), (2, 2, 4, 4));
    }

    #[test]
    fn editor_border_labels() {
        assert_eq!(EditorBorder::Visible.label(), "Visível");
        assert_eq!(EditorBorder::Hidden.label(), "Invisível");
    }

    #[test]
    fn show_memory_enabled_by_default() {
        assert!(ViewState::default().show_memory);
    }
}
