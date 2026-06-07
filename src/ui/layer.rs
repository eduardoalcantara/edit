//! Identidade e contrato das camadas de UI.
//!
//! Cada camada declara ordem (z), visibilidade, se captura input e como pinta.
//! O compositor pinta de baixo para cima e entrega eventos de cima para baixo.

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::Frame;

use crate::app::App;
use crate::theme::ThemePalette;

use super::layout::UiLayout;

/// Ordem de empilhamento (maior = mais acima).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerId {
    Desktop = 0,
    Editor = 10,
    Terminal = 15,
    Footer = 20,
    MenuBar = 30,
    MenuDropdown = 40,
    Modal = 50,
}

impl LayerId {
    pub fn z(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputResult {
    Unhandled,
    Consumed,
}

pub trait UiLayer {
    fn id(&self) -> LayerId;

    fn is_visible(&self, app: &App) -> bool;

    fn captures_input(&self, app: &App) -> bool;

    fn paint(
        &self,
        frame: &mut Frame<'_>,
        app: &mut App,
        layout: UiLayout,
        palette: ThemePalette,
    );

    fn on_key(&self, key: KeyEvent, app: &mut App, layout: UiLayout) -> InputResult;

    fn on_mouse(&self, mouse: MouseEvent, app: &mut App, layout: UiLayout) -> InputResult;

    fn footer_hint(&self, app: &App) -> Option<String> {
        let _ = app;
        None
    }
}
