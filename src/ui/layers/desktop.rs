use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::widgets::Block;

use crate::ui::layer::{InputResult, LayerId, UiLayer};

pub struct DesktopLayer;

impl UiLayer for DesktopLayer {
    fn id(&self) -> LayerId {
        LayerId::Desktop
    }

    fn is_visible(&self, _: &crate::app::App) -> bool {
        true
    }

    fn captures_input(&self, _: &crate::app::App) -> bool {
        false
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        _: &mut crate::app::App,
        _: crate::ui::layout::UiLayout,
        palette: crate::theme::ThemePalette,
    ) {
        frame.render_widget(
            Block::default().style(palette.desktop_style()),
            frame.area(),
        );
    }

    fn on_key(&self, _: KeyEvent, _: &mut crate::app::App, _: crate::ui::layout::UiLayout) -> InputResult {
        InputResult::Unhandled
    }

    fn on_mouse(&self, _: MouseEvent, _: &mut crate::app::App, _: crate::ui::layout::UiLayout) -> InputResult {
        InputResult::Unhandled
    }
}
