use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use tui_textarea::Input;

use crate::app::App;

pub fn poll(timeout: Duration) -> io::Result<bool> {
    event::poll(timeout)
}

pub fn read() -> io::Result<Event> {
    event::read()
}

pub fn dispatch(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) if app.mouse_enabled => handle_mouse(app, mouse),
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
        app.should_quit = true;
        return;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        app.status_message = "Salvar: em breve nesta versão".to_string();
        return;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('o') {
        app.status_message = "Abrir: em breve nesta versão".to_string();
        return;
    }

    let input: Input = key.into();
    app.editor.handle_input(input);
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    let input: Input = mouse.into();
    app.editor.handle_input(input);
}
