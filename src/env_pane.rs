//! Painel estruturado de variáveis de ambiente (split direito).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::env_editor::{EnvScope, EnvVar};
use crate::theme::ThemePalette;
use crate::widgets::panel::{self, PanelBorder};

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
            status_hint: "Secrets visíveis. Novas apps herdam após salvar (pode precisar logout)."
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
    }

    pub fn handle_key(&mut self, key: &KeyEvent, visible_rows: usize) -> EnvPaneAction {
        use crossterm::event::KeyEventKind;
        if key.kind != KeyEventKind::Press {
            return EnvPaneAction::None;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('f' | 'F')
                if !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                EnvPaneAction::Close
            }
            KeyCode::Char('s' | 'S') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EnvPaneAction::Save
            }
            KeyCode::Char('r' | 'R') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EnvPaneAction::Reload
            }
            KeyCode::Char('n' | 'N') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                EnvPaneAction::New
            }
            KeyCode::Enter | KeyCode::Char('e' | 'E')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || matches!(key.code, KeyCode::Enter) =>
            {
                EnvPaneAction::Edit
            }
            KeyCode::Delete | KeyCode::Char('d' | 'D')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || matches!(key.code, KeyCode::Delete) =>
            {
                EnvPaneAction::Delete
            }
            KeyCode::Tab => EnvPaneAction::ToggleScope,
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::Down => {
                if !self.vars.is_empty() && self.selected + 1 < self.vars.len() {
                    self.selected += 1;
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(visible_rows.max(1));
                self.clamp_scroll(visible_rows);
                EnvPaneAction::None
            }
            KeyCode::PageDown => {
                if !self.vars.is_empty() {
                    self.selected =
                        (self.selected + visible_rows.max(1)).min(self.vars.len() - 1);
                    self.clamp_scroll(visible_rows);
                }
                EnvPaneAction::None
            }
            KeyCode::Home => {
                self.selected = 0;
                self.clamp_scroll(visible_rows);
                EnvPaneAction::None
            }
            KeyCode::End => {
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
        visible_rows: usize,
    ) -> EnvPaneAction {
        if !matches!(
            mouse.kind,
            MouseEventKind::Down(crossterm::event::MouseButton::Left)
        ) {
            return EnvPaneAction::None;
        }
        if mouse.column < list_area.x
            || mouse.row < list_area.y
            || mouse.column >= list_area.x.saturating_add(list_area.width)
            || mouse.row >= list_area.y.saturating_add(list_area.height)
        {
            return EnvPaneAction::None;
        }
        let row = (mouse.row - list_area.y) as usize;
        let idx = self.scroll + row;
        if idx < self.vars.len() {
            self.selected = idx;
            self.clamp_scroll(visible_rows);
        }
        EnvPaneAction::None
    }
}

pub fn paint_env_pane(
    frame: &mut Frame,
    area: Rect,
    pane: &mut EnvPane,
    palette: &ThemePalette,
    focused: bool,
    close_label: &str,
) -> (Rect, Option<Rect>) {
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

    if content.height >= 1 {
        let help = Line::from(vec![Span::styled(
            " Tab:escopo  ^N:novo  Enter:editar  Del:excluir  ^S:salvar  ^R:recarregar  Esc:fechar ",
            Style::default().fg(palette.menu_shortcut),
        )]);
        frame.render_widget(
            Paragraph::new(help),
            Rect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: 1,
            },
        );
    }

    let list_y = content.y.saturating_add(1);
    let list_h = content.height.saturating_sub(1);
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
            " (nenhuma variável neste ficheiro/escopo) ",
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
            let value_w = content
                .width
                .saturating_sub(name_w as u16 + 3) as usize;
            let value = truncate(&var.value, value_w);
            let text = format!(" {name:<name_w$} {value}");
            lines.push(Line::from(Span::styled(text, style)));
        }
    }
    frame.render_widget(Paragraph::new(lines), list_area);
    (list_area, action_hit)
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
