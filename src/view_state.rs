use crate::theme::ThemeId;

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

#[derive(Debug, Clone)]
pub struct ViewState {
    pub zoom: u8,
    pub word_wrap: bool,
    pub show_symbols: bool,
    pub show_spaces: bool,
    pub show_tabs: bool,
    pub show_eol: bool,
    pub side_panel: bool,
    pub terminal: bool,
    pub footer_visible: bool,
    pub show_memory: bool,
    pub guide_column: GuideColumn,
    pub margin: EditorMargin,
    pub border: EditorBorder,
    pub theme: ThemeId,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            zoom: 1,
            word_wrap: false,
            show_symbols: false,
            show_spaces: false,
            show_tabs: false,
            show_eol: false,
            side_panel: false,
            terminal: false,
            footer_visible: true,
            show_memory: true,
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
            "Wrap:{wrap} Col:{} Zoom:{}",
            self.guide_column.label(),
            self.zoom
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
