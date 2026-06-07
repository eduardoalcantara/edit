use crate::theme::ThemeId;

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
    pub guide_column: GuideColumn,
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
            guide_column: GuideColumn::Unlimited,
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
