//! Painel estruturado de variáveis de ambiente (split direito).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::env_editor::{EnvScope, EnvVar};
use crate::modal::form_controls::{button_width, paint_button_with_mnemonic, rect_contains};
use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

/// Linhas por notch da roda no painel de variáveis.
const ENV_WHEEL_LINES: usize = 3;

/// Botões da barra (mnemônicos ≠ menus Alt+A/E/X/F/S/T/H).
const TOOLBAR: &[ToolbarBtn] = &[
    ToolbarBtn {
        label: "Origem",
        mnemonic: 'O',
        action: EnvPaneAction::ToggleScope,
        help: "Alt+O — alterna Usuário ↔ Sistema",
    },
    ToolbarBtn {
        label: "Nova",
        mnemonic: 'N',
        action: EnvPaneAction::New,
        help: "Alt+N — cria variável",
    },
    ToolbarBtn {
        label: "Mudar",
        mnemonic: 'M',
        action: EnvPaneAction::Edit,
        help: "Alt+M — edita a variável selecionada",
    },
    ToolbarBtn {
        label: "Remover",
        mnemonic: 'R',
        action: EnvPaneAction::Delete,
        help: "Alt+R — exclui a variável selecionada",
    },
    ToolbarBtn {
        label: "Gravar",
        mnemonic: 'G',
        action: EnvPaneAction::Save,
        help: "Alt+G — salva o escopo atual no disco",
    },
    ToolbarBtn {
        label: "Ler",
        mnemonic: 'L',
        action: EnvPaneAction::Reload,
        help: "Alt+L — recarrega do disco / ambiente",
    },
    ToolbarBtn {
        label: "Fechar",
        mnemonic: 'C',
        action: EnvPaneAction::Close,
        help: "Alt+C — fecha o painel",
    },
];
struct ToolbarBtn {
    label: &'static str,
    mnemonic: char,
    action: EnvPaneAction,
    help: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvPaneAction {
    None,
    Close,
    Save,
    Reload,
    New,
    Edit,
    Delete,
    ToggleScope,
    SwitchScope(EnvScope),
}

#[derive(Debug, Clone, Copy)]
pub struct EnvButtonHit {
    pub rect: Rect,
    pub action: EnvPaneAction,
    pub help: &'static str,
}

#[derive(Debug)]
pub struct EnvPane {
    pub scope: EnvScope,
    pub vars: Vec<EnvVar>,
    pub dirty: bool,
    pub selected: usize,
    pub scroll: usize,
    pub stashed_right_tab: Option<usize>,
    pub status_hint: String,
}

impl EnvPane {
    pub fn new(scope: EnvScope, vars: Vec<EnvVar>, stashed_right_tab: Option<usize>) -> Self {
        Self {
            scope,
            vars,
            dirty: false,
            selected: 0,
            scroll: 0,
            stashed_right_tab,
            status_hint: "Secrets visíveis. Novas apps herdam após gravar (pode precisar logout)."
                .into(),
        }
    }

    pub fn title(&self) -> String {
        let star = if self.dirty { "*" } else { "" };
        format!("Variáveis — {}{star}", self.scope.label())
    }

    pub fn selected_var(&self) -> Option<&EnvVar> {
        self.vars.get(self.selected)
    }

    pub fn ensure_selection(&mut self) {
        if self.vars.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        if self.selected >= self.vars.len() {
            self.selected = self.vars.len() - 1;
        }
    }

    pub fn clamp_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 || self.vars.is_empty() {
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
        let max_scroll = self.vars.len().saturating_sub(visible_rows);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub fn scroll_by(&mut self, delta: i32, visible_rows: usize) {
        if self.vars.is_empty() || visible_rows == 0 {
            return;
        }
        let max_sel = self.vars.len() - 1;
        if delta < 0 {
            self.selected = self.selected.saturating_sub((-delta) as usize);
        } else {
            self.selected = (self.selected + delta as usize).min(max_sel);
        }
        self.clamp_scroll(visible_rows);
    }

    pub fn handle_key(&mut self, key: &KeyEvent, visible_rows: usize) -> EnvPaneAction {
        use crossterm::event::KeyEventKind;
        if key.kind != KeyEventKind::Press {
            return EnvPaneAction::None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Mnemônicos dos botões: Alt+letra (padrão Windows / diálogos).
        // Menus usam Alt+A/E/X/F/S/T/H — estes botões usam O/N/M/R/G/L/C (sem colisão).
        if alt && !ctrl {
            if let KeyCode::Char(ch) = key.code {
                let lower = ch.to_ascii_lowercase();
                for btn in TOOLBAR {
                    if btn.mnemonic.to_ascii_lowercase() == lower {
                        return btn.action;
                    }
                }
            }
            return EnvPaneAction::None;
        }

        match key.code {
            KeyCode::Esc if !ctrl && !alt => EnvPaneAction::None, // Esc global: ver compositor
            KeyCode::Enter if !ctrl && !alt => EnvPaneAction::Edit,
            KeyCode::Delete if !ctrl && !alt => EnvPaneAction::Delete,
            KeyCode::Tab if !ctrl && !alt => EnvPaneAction::ToggleScope,
            KeyCode::Up if !ctrl && !alt => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::Down if !ctrl && !alt => {
                if !self.vars.is_empty() && self.selected + 1 < self.vars.len() {
                    self.selected += 1;
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::PageUp if !ctrl && !alt => {
                self.selected = self.selected.saturating_sub(visible_rows.max(1));
                self.clamp_scroll(visible_rows);
                EnvPaneAction::None
            }
            KeyCode::PageDown if !ctrl && !alt => {
                if !self.vars.is_empty() {
                    self.selected =
                        (self.selected + visible_rows.max(1)).min(self.vars.len() - 1);
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::Home if !ctrl && !alt => {
                self.selected = 0;
                self.clamp_scroll(visible_rows);
                EnvPaneAction::None
            }
            KeyCode::End if !ctrl && !alt => {
                if !self.vars.is_empty() {
                    self.selected = self.vars.len() - 1;
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            _ => EnvPaneAction::None,
        }
    }

    pub fn handle_mouse(
        &mut self,
        mouse: &MouseEvent,
        list_area: Rect,
        button_hits: &[EnvButtonHit],
        visible_rows: usize,
    ) -> Option<EnvPaneAction> {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if rect_contains(list_area, mouse) {
                    self.scroll_by(-(ENV_WHEEL_LINES as i32), visible_rows);
                    return Some(EnvPaneAction::None);
                }
                return None;
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(list_area, mouse) {
                    self.scroll_by(ENV_WHEEL_LINES as i32, visible_rows);
                    return Some(EnvPaneAction::None);
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
        if idx < self.vars.len() {
            self.selected = idx;
            self.clamp_scroll(visible_rows);
        }
        Some(EnvPaneAction::None)
    }
}

pub fn paint_env_pane(
    frame: &mut Frame,
    area: Rect,
    pane: &mut EnvPane,
    palette: &ThemePalette,
    focused: bool,
    close_label: &str,
    use_paren_mnemonics: bool,
) -> (Rect, Option<Rect>, Vec<EnvButtonHit>) {
    let _ = use_paren_mnemonics;
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

    // Linha em branco + botões + linha em branco (mín. 3 linhas de conteúdo).
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
            button_hits.push(EnvButtonHit {
                rect,
                action: btn.action,
                help: btn.help,
            });
            cx = cx.saturating_add(w).saturating_add(1);
        }
        list_y = content.y.saturating_add(3);
        list_h = content.height.saturating_sub(3);
    } else if content.height >= 1 {
        // Painel muito baixo: só botões, sem paddings.
        let toolbar_y = content.y;
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
            button_hits.push(EnvButtonHit {
                rect,
                action: btn.action,
                help: btn.help,
            });
            cx = cx.saturating_add(w).saturating_add(1);
        }
        list_y = content.y.saturating_add(1);
        list_h = content.height.saturating_sub(1);
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

    let name_w = ((content.width as usize) / 3).clamp(12, 32);
    let mut lines = Vec::new();
    if pane.vars.is_empty() {
        lines.push(Line::from(Span::styled(
            " (nenhuma variável neste escopo) ",
            Style::default().fg(palette.menu_shortcut),
        )));
    } else {
        for (i, var) in pane.vars.iter().enumerate().skip(pane.scroll).take(visible) {
            let selected = i == pane.selected && focused;
            let style = if selected {
                palette.menu_item_focus_style()
            } else {
                palette.editor_text_style()
            };
            let name = truncate(&var.name, name_w);
            let value_w = content.width.saturating_sub(name_w as u16 + 3) as usize;
            let value = truncate(&var.value, value_w);
            let text = format!(" {name:<name_w$} {value}");
            lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(lines), list_area);
    (list_area, action_hit, button_hits)
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }
    let take = max - 1;
    let mut out: String = s.chars().take(take).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_mnemonics_avoid_menu_letters() {
        const MENU: &[char] = &['A', 'E', 'X', 'F', 'S', 'T', 'H'];
        for btn in TOOLBAR {
            let m = btn.mnemonic.to_ascii_uppercase();
            assert!(
                !MENU.contains(&m),
                "mnemônico '{}' de '{}' colide com menu",
                btn.mnemonic,
                btn.label
            );
        }
        let mut seen = std::collections::HashSet::new();
        for btn in TOOLBAR {
            assert!(
                seen.insert(btn.mnemonic.to_ascii_uppercase()),
                "mnemônico duplicado: {}",
                btn.mnemonic
            );
        }
    }
}
