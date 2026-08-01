//! Lista estruturada de configs Linux (split direito).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::catalog::{list_existing_configs, ResolvedConfig};
use crate::modal::form_controls::{button_width, paint_button_with_mnemonic, rect_contains};
use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

const WHEEL_LINES: usize = 3;

const TOOLBAR: &[ToolbarBtn] = &[
    ToolbarBtn {
        label: "Ler",
        mnemonic: 'L',
        action: LinuxConfigsAction::Reload,
        help: "Alt+L — atualiza a lista (ficheiros existentes)",
    },
    ToolbarBtn {
        label: "Fechar",
        mnemonic: 'C',
        action: LinuxConfigsAction::Close,
        help: "Alt+C — fecha o painel",
    },
];

struct ToolbarBtn {
    label: &'static str,
    mnemonic: char,
    action: LinuxConfigsAction,
    help: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxConfigsAction {
    None,
    Close,
    Reload,
    /// Abrir a entrada selecionada no Editor 1.
    OpenSelected,
}

#[derive(Debug, Clone, Copy)]
pub struct LinuxConfigsButtonHit {
    pub rect: Rect,
    pub action: LinuxConfigsAction,
    pub help: &'static str,
}

#[derive(Debug)]
pub struct LinuxConfigsPane {
    pub items: Vec<ResolvedConfig>,
    pub selected: usize,
    pub scroll: usize,
    pub stashed_right_tab: Option<usize>,
}

impl LinuxConfigsPane {
    pub fn new(stashed_right_tab: Option<usize>) -> Self {
        let mut pane = Self {
            items: list_existing_configs(),
            selected: 0,
            scroll: 0,
            stashed_right_tab,
        };
        pane.ensure_selection();
        pane
    }

    pub fn title(&self) -> &'static str {
        "Configs Linux"
    }

    pub fn reload(&mut self) {
        self.items = list_existing_configs();
        self.ensure_selection();
    }

    pub fn selected_item(&self) -> Option<&ResolvedConfig> {
        self.items.get(self.selected)
    }

    pub fn ensure_selection(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn clamp_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 || self.items.is_empty() {
            self.scroll = 0;
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        let bottom = self.scroll + visible_rows - 1;
        if self.selected > bottom {
            self.scroll = self.selected + 1 - visible_rows;
        }
        let max_scroll = self.items.len().saturating_sub(visible_rows);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub fn scroll_by(&mut self, delta: i32, visible_rows: usize) {
        if self.items.is_empty() || visible_rows == 0 {
            return;
        }
        let max_sel = self.items.len() - 1;
        if delta < 0 {
            self.selected = self.selected.saturating_sub((-delta) as usize);
        } else {
            self.selected = (self.selected + delta as usize).min(max_sel);
        }
        self.clamp_scroll(visible_rows);
    }

    pub fn handle_key(&mut self, key: &KeyEvent, visible_rows: usize) -> LinuxConfigsAction {
        use crossterm::event::KeyEventKind;
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return LinuxConfigsAction::None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        if alt && !ctrl {
            if let KeyCode::Char(ch) = key.code {
                let lower = ch.to_ascii_lowercase();
                for btn in TOOLBAR {
                    if btn.mnemonic.to_ascii_lowercase() == lower {
                        return btn.action;
                    }
                }
            }
            return LinuxConfigsAction::None;
        }

        match key.code {
            KeyCode::Enter if !ctrl && !alt => LinuxConfigsAction::OpenSelected,
            KeyCode::Up if !ctrl && !alt => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.clamp_scroll(visible_rows);
                }
                LinuxConfigsAction::None
            }
            KeyCode::Down if !ctrl && !alt => {
                if !self.items.is_empty() && self.selected + 1 < self.items.len() {
                    self.selected += 1;
                    self.clamp_scroll(visible_rows);
                }
                LinuxConfigsAction::None
            }
            KeyCode::PageUp if !ctrl && !alt => {
                self.scroll_by(-(visible_rows.max(1) as i32), visible_rows);
                LinuxConfigsAction::None
            }
            KeyCode::PageDown if !ctrl && !alt => {
                self.scroll_by(visible_rows.max(1) as i32, visible_rows);
                LinuxConfigsAction::None
            }
            KeyCode::Home if !ctrl && !alt => {
                self.selected = 0;
                self.clamp_scroll(visible_rows);
                LinuxConfigsAction::None
            }
            KeyCode::End if !ctrl && !alt => {
                if !self.items.is_empty() {
                    self.selected = self.items.len() - 1;
                    self.clamp_scroll(visible_rows);
                }
                LinuxConfigsAction::None
            }
            _ => LinuxConfigsAction::None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: &MouseEvent,
        list_area: Rect,
        button_hits: &[LinuxConfigsButtonHit],
        visible_rows: usize,
    ) -> Option<LinuxConfigsAction> {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if rect_contains(list_area, mouse) {
                    self.scroll_by(-(WHEEL_LINES as i32), visible_rows);
                    return Some(LinuxConfigsAction::None);
                }
                return None;
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(list_area, mouse) {
                    self.scroll_by(WHEEL_LINES as i32, visible_rows);
                    return Some(LinuxConfigsAction::None);
                }
                return None;
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return None,
        }

        for hit in button_hits {
            if rect_contains(hit.rect, mouse) {
                return Some(hit.action);
            }
        }

        if !rect_contains(list_area, mouse) {
            return None;
        }
        let row = (mouse.row - list_area.y) as usize;
        let idx = self.scroll + row;
        if idx < self.items.len() {
            self.selected = idx;
            self.clamp_scroll(visible_rows);
            return Some(LinuxConfigsAction::OpenSelected);
        }
        Some(LinuxConfigsAction::None)
    }
}

pub fn paint_linux_configs_pane(
    frame: &mut Frame,
    area: Rect,
    pane: &mut LinuxConfigsPane,
    palette: &ThemePalette,
    focused: bool,
    close_label: &str,
) -> (Rect, Option<Rect>, Vec<LinuxConfigsButtonHit>) {
    let border = if focused {
        PanelBorder::Double
    } else {
        PanelBorder::Plain
    };
    let title = format!("[ {} ]", pane.title());
    let (content, action_hit) = panel::render_editor_frame_with_border_and_action(
        frame,
        area,
        &title,
        Some(close_label),
        palette.button_style(true),
        Style::default().bg(palette.editor_bg).fg(palette.editor_fg),
        Style::default().fg(palette.border),
        Style::default()
            .fg(palette.header_fg)
            .bg(palette.header_bg)
            .add_modifier(Modifier::BOLD),
        true,
        None,
        border,
    );

    let mut button_hits = Vec::new();
    let mut list_y = content.y;
    let mut list_h = content.height;

    if content.height >= 3 {
        let toolbar_y = content.y.saturating_add(1);
        let mut cx = content.x.saturating_add(1);
        let max_x = content.x.saturating_add(content.width);
        for btn in TOOLBAR {
            let w = button_width(btn.label);
            if cx.saturating_add(w) > max_x {
                break;
            }
            let rect = Rect {
                x: cx,
                y: toolbar_y,
                width: w,
                height: 1,
            };
            paint_button_with_mnemonic(
                frame,
                rect,
                btn.label,
                Some(btn.mnemonic),
                false,
                *palette,
            );
            button_hits.push(LinuxConfigsButtonHit {
                rect,
                action: btn.action,
                help: btn.help,
            });
            cx = cx.saturating_add(w).saturating_add(1);
        }
        list_y = content.y.saturating_add(3);
        list_h = content.height.saturating_sub(3);
    }

    let list_area = Rect {
        x: content.x,
        y: list_y,
        width: content.width,
        height: list_h,
    };

    let visible = list_h as usize;
    pane.ensure_selection();
    pane.clamp_scroll(visible);

    let total_w = content.width as usize;
    // Reserva mínima para a descrição; o resto vai para nome + path (sem reticências).
    let desc_min = 16.min(total_w / 4);
    let left_budget = total_w.saturating_sub(desc_min);

    let mut lines = Vec::new();
    if pane.items.is_empty() {
        lines.push(Line::from(Span::styled(
            " Nenhum ficheiro do catálogo encontrado neste sistema.",
            Style::default().fg(palette.footer_fg),
        )));
    } else {
        // Largura da coluna esquerda = maior linha nome+path visível, limitada ao orçamento.
        let mut left_w = 8usize;
        for item in pane.items.iter().skip(pane.scroll).take(visible) {
            let left = format_left(item);
            left_w = left_w.max(left.chars().count());
        }
        left_w = left_w.min(left_budget).max(8);

        for (i, item) in pane
            .items
            .iter()
            .enumerate()
            .skip(pane.scroll)
            .take(visible)
        {
            let selected = i == pane.selected && focused;
            let style = if selected {
                palette.menu_item_focus_style()
            } else {
                palette.editor_text_style()
            };
            let left_trim = clip_pad(&format_left(item), left_w);
            let desc_w = total_w.saturating_sub(left_w);
            let desc = clip_pad(&format!(" {}", item.description), desc_w);
            lines.push(Line::from(vec![
                Span::styled(left_trim, style),
                Span::styled(desc, style),
            ]));
        }
    }

    if list_h > 0 && list_area.width > 0 {
        frame.render_widget(Paragraph::new(lines), list_area);
    }

    (list_area, action_hit, button_hits)
}

fn format_left(item: &ResolvedConfig) -> String {
    format!(" {}  {}", item.name, item.path.display())
}

/// Corta ou preenche até `width` caracteres — sem reticências (não partem o nome do ficheiro).
fn clip_pad(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        let mut out: String = chars.into_iter().collect();
        while out.chars().count() < width {
            out.push(' ');
        }
        out
    } else {
        chars.into_iter().take(width).collect()
    }
}
