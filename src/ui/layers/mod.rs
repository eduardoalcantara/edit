pub mod desktop;
pub mod editor;
pub mod footer;
pub mod menu_bar;
pub mod menu_dropdown;
pub mod modal;
pub mod terminal;

use desktop::DesktopLayer;
use editor::EditorLayer;
use footer::FooterLayer;
use menu_bar::MenuBarLayer;
use menu_dropdown::MenuDropdownLayer;
use modal::ModalLayer;
use terminal::TerminalLayer;

use super::layer::UiLayer;

/// Registro estático de todas as camadas (ordem de declaração irrelevante; z-order vem de `LayerId`).
pub fn all_layers() -> [&'static dyn UiLayer; 7] {
    [
        &DesktopLayer,
        &EditorLayer,
        &TerminalLayer,
        &FooterLayer,
        &MenuBarLayer,
        &MenuDropdownLayer,
        &ModalLayer,
    ]
}
