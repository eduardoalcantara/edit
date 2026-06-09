use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::App;
use crate::editor_split::{EditorSplitLayout, split_editor_horizontally};
use crate::terminal::{
    editor_content_in_shell, layout_terminal_panel, terminal_panel_outer,
    terminal_reserved_rows, TerminalPanelLayout,
};

/// Retângulos estáveis da shell (menu, editor+terminal, rodapé).
#[derive(Debug, Clone, Copy)]
pub struct UiLayout {
    pub menu_bar: Rect,
    /// Área unificada do editor (e terminal, se visível).
    pub shell: Rect,
    pub editor_content: Rect,
    pub editor_split: Option<EditorSplitLayout>,
    pub terminal_divider_y: Option<u16>,
    pub terminal_panel_rows: Option<u16>,
    pub terminal: Option<TerminalPanelLayout>,
    pub footer: Option<Rect>,
}

impl UiLayout {
    pub fn compute(frame: Rect, app: &App) -> Self {
        let mut constraints = vec![Constraint::Length(1), Constraint::Min(3)];
        if app.view.footer_visible {
            constraints.push(Constraint::Length(1));
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(frame);

        let (shell, footer) = if app.view.footer_visible {
            (chunks[1], Some(chunks[2]))
        } else {
            (chunks[1], None)
        };

        let border_visible = app.view.border == crate::view_state::EditorBorder::Visible;

        let terminal_reserve = if app.view.terminal {
            terminal_reserved_rows(shell, app.view.terminal_panel_rows)
        } else {
            0
        };

        let editor_split = if app.split_active() {
            Some(split_editor_horizontally(shell, terminal_reserve))
        } else {
            None
        };

        if app.view.terminal {
            let panel_rows = app.view.terminal_panel_rows;
            let reserve = terminal_reserved_rows(shell, panel_rows);
            let term_outer = terminal_panel_outer(shell, panel_rows);
            let panel = layout_terminal_panel(shell, term_outer, border_visible);
            let editor_content =
                editor_content_in_shell(shell, reserve, border_visible);
            Self {
                menu_bar: chunks[0],
                shell,
                editor_content,
                editor_split,
                terminal_divider_y: Some(term_outer.y),
                terminal_panel_rows: Some(panel_rows),
                terminal: Some(panel),
                footer,
            }
        } else {
            let editor_content =
                editor_content_in_shell(shell, 0, border_visible);
            Self {
                menu_bar: chunks[0],
                shell,
                editor_content,
                editor_split,
                terminal_divider_y: None,
                terminal_panel_rows: None,
                terminal: None,
                footer,
            }
        }
    }

    /// Compat: área externa do editor (shell).
    pub fn editor(&self) -> Rect {
        self.shell
    }
}
