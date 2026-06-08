//! Shell de UI: compositor de camadas com z-order e dispatch unificado.

mod compositor;
mod layer;
pub mod layout;
mod layers;

use ratatui::Frame;

use crate::app::App;

pub use compositor::Compositor;
pub use layout::UiLayout;

pub fn draw(frame: &mut Frame, app: &mut App) {
    app.last_frame_width = frame.area().width;
    app.last_frame_height = frame.area().height;
    Compositor::paint(frame, app);
}

pub fn dispatch(app: &mut App, event: crossterm::event::Event) {
    Compositor::dispatch(app, event);
}
