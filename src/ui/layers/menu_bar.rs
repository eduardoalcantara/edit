use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::menus::{self, MenuEventResult};
use crate::theme::ThemePalette;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

pub struct MenuBarLayer;

impl UiLayer for MenuBarLayer {
    fn id(&self) -> LayerId {
        LayerId::MenuBar
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
        app: &mut crate::app::App,
        layout: UiLayout,
        palette: ThemePalette,
    ) {
        menus::render_bar(
            frame,
            layout.menu_bar,
            &app.menu_bar,
            &mut app.menu_state,
            palette,
        );
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if app.menu_state.is_open() {
            return InputResult::Unhandled;
        }
        match menus::handle_key(&app.menu_bar, &mut app.menu_state, key) {
            MenuEventResult::Action(action) => {
                app.dispatch_action(action);
                InputResult::Consumed
            }
            MenuEventResult::Closed => {
                app.set_status("Menu fechado");
                InputResult::Consumed
            }
            MenuEventResult::Consumed => InputResult::Consumed,
            MenuEventResult::None => InputResult::Unhandled,
        }
    }

    fn on_mouse(&self, mouse: MouseEvent, app: &mut crate::app::App, layout: UiLayout) -> InputResult {
        if !app.mouse_enabled || app.menu_state.is_open() {
            return InputResult::Unhandled;
        }
        if !is_bar_click(layout.menu_bar, &mouse) {
            return InputResult::Unhandled;
        }
        match menus::handle_mouse(&app.menu_bar, &mut app.menu_state, mouse) {
            MenuEventResult::Action(action) => {
                app.dispatch_action(action);
                InputResult::Consumed
            }
            MenuEventResult::Closed => {
                app.set_status("Menu fechado");
                InputResult::Consumed
            }
            _ => InputResult::Consumed,
        }
    }
}

fn is_bar_click(bar: Rect, mouse: &MouseEvent) -> bool {
    if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
        return false;
    }
    mouse.row == bar.y
        && mouse.column >= bar.x
        && mouse.column < bar.x.saturating_add(bar.width)
}
