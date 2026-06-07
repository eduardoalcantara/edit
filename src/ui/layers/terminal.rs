use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::theme::ThemePalette;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

pub struct TerminalLayer;

impl UiLayer for TerminalLayer {
    fn id(&self) -> LayerId {
        LayerId::Terminal
    }

    fn is_visible(&self, app: &crate::app::App) -> bool {
        app.view.terminal
    }

    fn captures_input(&self, _: &crate::app::App) -> bool {
        false
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        _: &mut crate::app::App,
        layout: UiLayout,
        palette: ThemePalette,
    ) {
        let Some(area) = layout.terminal else {
            return;
        };
        let style = Style::default()
            .fg(palette.editor_fg)
            .bg(palette.editor_bg);
        for row in 0..area.height {
            frame.render_widget(
                Paragraph::new(" ").style(style),
                ratatui::layout::Rect {
                    x: area.x,
                    y: area.y.saturating_add(row),
                    width: area.width,
                    height: 1,
                },
            );
        }
        let hint = " Terminal (em breve) ";
        let x = area.x.saturating_add(1);
        if x < area.x.saturating_add(area.width) {
            frame.render_widget(
                Paragraph::new(hint).style(style),
                ratatui::layout::Rect {
                    x,
                    y: area.y.saturating_add(1),
                    width: area.width.saturating_sub(2),
                    height: 1,
                },
            );
        }
    }

    fn on_key(
        &self,
        _: crossterm::event::KeyEvent,
        _: &mut crate::app::App,
        _: UiLayout,
    ) -> InputResult {
        InputResult::Unhandled
    }

    fn on_mouse(
        &self,
        _: crossterm::event::MouseEvent,
        _: &mut crate::app::App,
        _: UiLayout,
    ) -> InputResult {
        InputResult::Unhandled
    }
}
