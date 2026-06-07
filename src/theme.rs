use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Dark,
    Light,
    ClassicBlue,
}

impl ThemeId {
    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Dark => "Escuro",
            ThemeId::Light => "Claro",
            ThemeId::ClassicBlue => "Azul Clássico",
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            ThemeId::Dark => ThemePalette::dark(),
            ThemeId::Light => ThemePalette::light(),
            ThemeId::ClassicBlue => ThemePalette::classic_blue(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    #[allow(dead_code)]
    pub background: Color,
    #[allow(dead_code)]
    pub foreground: Color,
    pub border: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub footer_bg: Color,
    pub footer_fg: Color,
    pub accent: Color,
    pub status: Color,
    pub editor_bg: Color,
    pub editor_fg: Color,
    pub cursor: Color,
    pub selection: Color,
}

impl ThemePalette {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(28, 28, 28),
            foreground: Color::Rgb(220, 220, 220),
            border: Color::Rgb(80, 80, 80),
            header_bg: Color::Rgb(45, 45, 45),
            header_fg: Color::Rgb(230, 230, 230),
            footer_bg: Color::Rgb(35, 35, 35),
            footer_fg: Color::Rgb(180, 180, 180),
            accent: Color::Cyan,
            status: Color::Yellow,
            editor_bg: Color::Rgb(24, 24, 24),
            editor_fg: Color::Rgb(230, 230, 230),
            cursor: Color::Rgb(255, 255, 255),
            selection: Color::Rgb(60, 80, 120),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(245, 245, 245),
            foreground: Color::Rgb(30, 30, 30),
            border: Color::Rgb(180, 180, 180),
            header_bg: Color::Rgb(230, 230, 230),
            header_fg: Color::Rgb(20, 20, 20),
            footer_bg: Color::Rgb(235, 235, 235),
            footer_fg: Color::Rgb(60, 60, 60),
            accent: Color::Blue,
            status: Color::Rgb(120, 80, 0),
            editor_bg: Color::White,
            editor_fg: Color::Black,
            cursor: Color::Black,
            selection: Color::Rgb(180, 200, 255),
        }
    }

    pub fn classic_blue() -> Self {
        Self {
            background: Color::Blue,
            foreground: Color::White,
            border: Color::Cyan,
            header_bg: Color::Rgb(0, 0, 170),
            header_fg: Color::White,
            footer_bg: Color::Rgb(0, 0, 140),
            footer_fg: Color::White,
            accent: Color::Yellow,
            status: Color::Green,
            editor_bg: Color::Rgb(0, 0, 128),
            editor_fg: Color::White,
            cursor: Color::Yellow,
            selection: Color::Rgb(0, 100, 200),
        }
    }

    pub fn header_style(self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .bg(self.header_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn footer_style(self) -> Style {
        Style::default().fg(self.footer_fg).bg(self.footer_bg)
    }

    pub fn status_style(self) -> Style {
        Style::default().fg(self.status).bg(self.footer_bg)
    }

    pub fn accent_style(self) -> Style {
        Style::default().fg(self.accent).bg(self.footer_bg)
    }

    pub fn editor_text_style(self) -> Style {
        Style::default().fg(self.editor_fg).bg(self.editor_bg)
    }

    pub fn cursor_style(self) -> Style {
        Style::default()
            .fg(self.editor_bg)
            .bg(self.cursor)
            .add_modifier(Modifier::REVERSED)
    }

    pub fn cursor_style_for_mode(self, mode: crate::edit_mode::EditMode) -> Style {
        match mode {
            crate::edit_mode::EditMode::Insert => self.cursor_style(),
            crate::edit_mode::EditMode::Replace => Style::default()
                .fg(self.editor_bg)
                .bg(self.status)
                .add_modifier(Modifier::REVERSED | Modifier::UNDERLINED),
        }
    }

    pub fn selection_style(self) -> Style {
        Style::default().fg(self.editor_fg).bg(self.selection)
    }

    pub fn placeholder_style(self) -> Style {
        Style::default()
            .fg(self.border)
            .bg(self.editor_bg)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn menu_bar_style(self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }

    pub fn menu_top_active_style(self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_panel_style(self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }

    pub fn menu_border_style(self) -> Style {
        Style::default().fg(self.border).bg(self.header_bg)
    }

    pub fn menu_item_style(self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }

    pub fn menu_item_focus_style(self) -> Style {
        Style::default()
            .fg(self.header_bg)
            .bg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_item_disabled_style(self) -> Style {
        Style::default()
            .fg(self.border)
            .bg(self.header_bg)
            .add_modifier(Modifier::DIM)
    }
}
