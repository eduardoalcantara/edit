use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::App;

/// Retângulos estáveis da shell (menu, editor, terminal, rodapé).
#[derive(Debug, Clone, Copy)]
pub struct UiLayout {
    pub menu_bar: Rect,
    pub editor: Rect,
    pub terminal: Option<Rect>,
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

        let (main, footer) = if app.view.footer_visible {
            (chunks[1], Some(chunks[2]))
        } else {
            (chunks[1], None)
        };

        let (editor, terminal) = if app.view.terminal {
            let split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(6)])
                .split(main);
            (split[0], Some(split[1]))
        } else {
            (main, None)
        };

        Self {
            menu_bar: chunks[0],
            editor,
            terminal,
            footer,
        }
    }
}
