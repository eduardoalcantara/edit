use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Dark,
    Light,
    ClassicBlue,
    Vga,
    Matrix,
}

impl ThemeId {
    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Dark => "Escuro",
            ThemeId::Light => "Claro",
            ThemeId::ClassicBlue => "Azul Clássico",
            ThemeId::Vga => "VGA 16 cores",
            ThemeId::Matrix => "Matrix",
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            ThemeId::Dark => ThemePalette::dark(),
            ThemeId::Light => ThemePalette::light(),
            ThemeId::ClassicBlue => ThemePalette::classic_blue(),
            ThemeId::Vga => ThemePalette::vga16(),
            ThemeId::Matrix => ThemePalette::matrix(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub background: Color,
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
    pub menu_hotkey: Color,
    pub menu_shortcut: Color,
    pub menu_focus_bg: Color,
    pub menu_top_active_bg: Color,
    pub button_bg: Color,
    pub button_fg: Color,
    pub shadow: Color,
}

impl ThemePalette {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(28, 28, 28),
            foreground: Color::Rgb(220, 220, 220),
            border: Color::Rgb(180, 180, 180),
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
            menu_hotkey: Color::Red,
            menu_shortcut: Color::Rgb(110, 110, 110),
            menu_focus_bg: Color::Green,
            menu_top_active_bg: Color::Red,
            button_bg: Color::Green,
            button_fg: Color::Black,
            shadow: Color::Black,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(245, 245, 245),
            foreground: Color::Rgb(30, 30, 30),
            border: Color::Rgb(100, 100, 100),
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
            menu_hotkey: Color::Red,
            menu_shortcut: Color::Rgb(110, 110, 110),
            menu_focus_bg: Color::Green,
            menu_top_active_bg: Color::Red,
            button_bg: Color::Green,
            button_fg: Color::Black,
            shadow: Color::DarkGray,
        }
    }

    /// Paleta inspirada Turbo Pascal / Turbo Vision (variante suavizada).
    pub fn classic_blue() -> Self {
        Self {
            background: Color::Blue,
            foreground: Color::White,
            border: Color::White,
            header_bg: Color::Gray,
            header_fg: Color::Black,
            footer_bg: Color::Gray,
            footer_fg: Color::Black,
            accent: Color::Yellow,
            status: Color::Yellow,
            editor_bg: Color::Blue,
            editor_fg: Color::White,
            cursor: Color::Yellow,
            selection: Color::Cyan,
            menu_hotkey: Color::Red,
            menu_shortcut: Color::DarkGray,
            menu_focus_bg: Color::Green,
            menu_top_active_bg: Color::Red,
            button_bg: Color::Green,
            button_fg: Color::Black,
            shadow: Color::Black,
        }
    }

    /// Paleta fixa DOS VGA 16 cores (índices 0–15): gray(7), blue(1), red(4), green(2), yellow(14), white(15).
    pub fn vga16() -> Self {
        Self {
            background: Color::Rgb(0, 0, 170),
            foreground: Color::Rgb(255, 255, 255),
            border: Color::Rgb(255, 255, 255),
            header_bg: Color::Rgb(170, 170, 170),
            header_fg: Color::Rgb(0, 0, 0),
            footer_bg: Color::Rgb(170, 170, 170),
            footer_fg: Color::Rgb(0, 0, 0),
            accent: Color::Rgb(255, 255, 85),
            status: Color::Rgb(255, 255, 85),
            editor_bg: Color::Rgb(0, 0, 170),
            editor_fg: Color::Rgb(255, 255, 255),
            cursor: Color::Rgb(255, 255, 85),
            selection: Color::Rgb(85, 85, 255),
            menu_hotkey: Color::Rgb(255, 0, 0),
            menu_shortcut: Color::Rgb(0, 0, 0),
            menu_focus_bg: Color::Rgb(0, 170, 0),
            menu_top_active_bg: Color::Rgb(255, 0, 0),
            button_bg: Color::Rgb(0, 170, 0),
            button_fg: Color::Rgb(0, 0, 0),
            shadow: Color::Rgb(0, 0, 0),
        }
    }

    /// Terminal fosforescente anos 70 / Matrix — verde sobre preto.
    pub fn matrix() -> Self {
        Self {
            background: Color::Rgb(0, 0, 0),
            foreground: Color::Rgb(0, 255, 65),
            border: Color::Rgb(0, 140, 35),
            header_bg: Color::Rgb(0, 35, 8),
            header_fg: Color::Rgb(0, 255, 65),
            footer_bg: Color::Rgb(0, 28, 6),
            footer_fg: Color::Rgb(0, 200, 50),
            accent: Color::Rgb(80, 255, 120),
            status: Color::Rgb(120, 255, 140),
            editor_bg: Color::Rgb(0, 0, 0),
            editor_fg: Color::Rgb(0, 255, 65),
            cursor: Color::Rgb(0, 255, 65),
            selection: Color::Rgb(0, 70, 18),
            menu_hotkey: Color::Rgb(180, 255, 100),
            menu_shortcut: Color::Rgb(0, 110, 32),
            menu_focus_bg: Color::Rgb(0, 90, 22),
            menu_top_active_bg: Color::Rgb(0, 110, 28),
            button_bg: Color::Rgb(0, 75, 18),
            button_fg: Color::Rgb(0, 255, 65),
            shadow: Color::Rgb(0, 18, 4),
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

    pub fn footer_hotkey_style(self) -> Style {
        Style::default()
            .fg(self.menu_hotkey)
            .bg(self.footer_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn accent_style(self) -> Style {
        Style::default().fg(self.accent).bg(self.footer_bg)
    }

    pub fn line_number_style(self) -> Style {
        Style::default().fg(self.menu_shortcut).bg(self.editor_bg)
    }

    pub fn line_number_active_style(self) -> Style {
        self.editor_text_style().add_modifier(Modifier::BOLD)
    }

    pub fn editor_text_style(self) -> Style {
        Style::default().fg(self.editor_fg).bg(self.editor_bg)
    }

    pub fn desktop_style(self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
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

    pub fn search_match_style(self) -> Style {
        Style::default().fg(self.status).bg(self.selection)
    }

    pub fn search_match_current_style(self) -> Style {
        Style::default()
            .fg(self.editor_bg)
            .bg(self.status)
            .add_modifier(Modifier::BOLD)
    }

    pub fn placeholder_style(self) -> Style {
        Style::default()
            .fg(self.border)
            .bg(self.editor_bg)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn menu_bar_style(self) -> Style {
        Style::default().fg(self.footer_fg).bg(self.footer_bg)
    }

    pub fn menu_top_active_style(self) -> Style {
        Style::default()
            .fg(Color::White)
            .bg(self.menu_top_active_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_panel_style(self) -> Style {
        Style::default().fg(self.footer_fg).bg(self.footer_bg)
    }

    pub fn menu_border_style(self) -> Style {
        Style::default().fg(self.border).bg(self.footer_bg)
    }

    pub fn dialog_title_style(self) -> Style {
        Style::default()
            .fg(self.footer_fg)
            .bg(self.footer_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_item_style(self) -> Style {
        Style::default().fg(self.footer_fg).bg(self.footer_bg)
    }

    pub fn menu_item_focus_style(self) -> Style {
        Style::default()
            .fg(self.footer_fg)
            .bg(self.menu_focus_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_item_disabled_style(self) -> Style {
        Style::default()
            .fg(Color::DarkGray)
            .bg(self.footer_bg)
    }

    pub fn menu_hotkey_style(self) -> Style {
        Style::default()
            .fg(self.menu_hotkey)
            .bg(self.footer_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Marcadores: `√` na coluna esquerda; `>` submenu à direita.
    pub fn menu_marker_style(self, focused: bool) -> Style {
        Style::default()
            .fg(self.menu_hotkey)
            .bg(if focused {
                self.menu_focus_bg
            } else {
                self.footer_bg
            })
            .add_modifier(Modifier::BOLD)
    }

    /// Atalhos à direita do item (`Ctrl+S`, etc.) — mais discretos que o rótulo.
    pub fn menu_shortcut_style(self, focused: bool) -> Style {
        Style::default()
            .fg(self.menu_shortcut)
            .bg(if focused {
                self.menu_focus_bg
            } else {
                self.footer_bg
            })
    }

    pub fn button_style(self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.button_fg)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.button_fg).bg(self.button_bg)
        }
    }

    /// Borda de caixa De/Para no modal de conversão (sem fundo verde).
    pub fn convert_field_border_style(self, focused: bool) -> Style {
        Style::default()
            .fg(if focused { self.footer_fg } else { self.border })
            .bg(self.footer_bg)
    }

    /// Título `[ De ]` / `[ Para ]` embutido na borda da caixa.
    pub fn convert_field_title_style(self, focused: bool) -> Style {
        Style::default()
            .fg(self.footer_fg)
            .bg(self.footer_bg)
            .add_modifier(if focused { Modifier::BOLD } else { Modifier::empty() })
    }

    /// Item dentro da lista De/Para (texto sobre o fundo do modal).
    pub fn convert_field_item_style(self, focused: bool, selected: bool) -> Style {
        Style::default()
            .fg(self.footer_fg)
            .bg(self.footer_bg)
            .add_modifier(if focused && selected {
                Modifier::BOLD
            } else {
                Modifier::empty()
            })
    }

    pub fn shadow_vertical_style(self) -> Style {
        Style::default().fg(self.shadow).bg(self.shadow)
    }

    /// Meio bloco superior (▀): fg = sombra, bg = editor (metade inferior “vazia”).
    pub fn shadow_horizontal_style(self) -> Style {
        Style::default().fg(self.shadow).bg(self.editor_bg)
    }
}
