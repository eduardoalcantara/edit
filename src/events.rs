use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;

pub fn poll(timeout: Duration) -> io::Result<bool> {
    event::poll(timeout)
}

pub fn read() -> io::Result<Event> {
    event::read()
}

pub fn dispatch(app: &mut App, event: Event) {
    if matches!(event, Event::Key(ref k) if k.kind != KeyEventKind::Press) {
        return;
    }
    crate::ui::dispatch(app, event);
}
