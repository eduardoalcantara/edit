use crossterm::event::{KeyEvent, MouseEvent};

use crate::menus::{self, MenuEventResult};
use crate::theme::ThemePalette;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

/// Overlay modal: dropdown opaco; captura input enquanto aberto.
pub struct MenuDropdownLayer;

impl UiLayer for MenuDropdownLayer {
    fn id(&self) -> LayerId {
        LayerId::MenuDropdown
    }

    fn is_visible(&self, app: &crate::app::App) -> bool {
        app.menu_state.is_open()
    }

    fn captures_input(&self, app: &crate::app::App) -> bool {
        app.menu_state.is_open()
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        app: &mut crate::app::App,
        _: UiLayout,
        palette: ThemePalette,
    ) {
        menus::render_dropdown(
            frame,
            &app.menu_bar,
            &mut app.menu_state,
            palette,
            app.view.use_paren_mnemonics,
        );
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        match menus::handle_key(&app.menu_bar, &mut app.menu_state, key) {
            MenuEventResult::Action(action) => {
                app.dispatch_action(action);
                InputResult::Consumed
            }
            MenuEventResult::Closed => {
                app.set_status("Menu fechado");
                InputResult::Consumed
            }
            MenuEventResult::Consumed | MenuEventResult::None => InputResult::Consumed,
        }
    }

    fn on_mouse(&self, mouse: MouseEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if !app.mouse_enabled {
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

    fn footer_hint(&self, app: &crate::app::App) -> Option<String> {
        menus::focused_help(&app.menu_bar, &app.menu_state)
            .map(str::to_string)
            .or_else(|| Some("←/→ menus | ↑/↓ itens | Enter seleciona | Esc fecha".to_string()))
    }
}
