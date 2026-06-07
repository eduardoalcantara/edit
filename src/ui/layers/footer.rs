use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::widgets::Paragraph;

use crate::theme::ThemePalette;
use crate::ui::compositor;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;
use crate::widgets::panel;

pub struct FooterLayer;

impl UiLayer for FooterLayer {
    fn id(&self) -> LayerId {
        LayerId::Footer
    }

    fn is_visible(&self, app: &crate::app::App) -> bool {
        app.view.footer_visible
    }

    fn captures_input(&self, _: &crate::app::App) -> bool {
        false
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        app: &mut crate::app::App,
        layout: UiLayout,
        palette: ThemePalette,
    ) {
        let Some(area) = layout.footer else {
            return;
        };
        panel::fill_rect(frame, area, palette.footer_style());
        let inner = compositor::footer_inner(area);
        let line = compositor::compose_footer_line(
            &compositor::footer_help_left(app),
            &compositor::footer_status_right(app),
            inner.width as usize,
        );
        frame.render_widget(
            Paragraph::new(line).style(palette.footer_style()),
            inner,
        );
    }

    fn on_key(&self, _: KeyEvent, _: &mut crate::app::App, _: UiLayout) -> InputResult {
        InputResult::Unhandled
    }

    fn on_mouse(&self, _: MouseEvent, _: &mut crate::app::App, _: UiLayout) -> InputResult {
        InputResult::Unhandled
    }
}
