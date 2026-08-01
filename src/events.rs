use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;

pub fn poll(timeout: Duration) -> io::Result<bool> {
    event::poll(timeout)
}

pub fn read() -> io::Result<Event> {
    // No Linux/macOS, leitura bloqueante com mouse activo pode interpretar
    // sequências CSI incompletas da roda como tecla Esc (crossterm#854).
    #[cfg(unix)]
    {
        let _guard = StdinNonblockGuard::enter();
        event::read()
    }
    #[cfg(not(unix))]
    {
        event::read()
    }
}

pub fn dispatch(app: &mut App, event: Event) {
    if matches!(event, Event::Key(ref k) if k.kind != KeyEventKind::Press) {
        return;
    }
    crate::ui::dispatch(app, event);
}

#[cfg(unix)]
struct StdinNonblockGuard {
    fd: i32,
    old_flags: i32,
}

#[cfg(unix)]
impl StdinNonblockGuard {
    fn enter() -> Option<Self> {
        use std::os::fd::AsRawFd;
        let fd = io::stdin().as_raw_fd();
        // libc fcntl — evita dependência nix só para isto.
        let old = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if old < 0 {
            return None;
        }
        let new_flags = old | libc::O_NONBLOCK;
        if unsafe { libc::fcntl(fd, libc::F_SETFL, new_flags) } < 0 {
            return None;
        }
        Some(Self {
            fd,
            old_flags: old,
        })
    }
}

#[cfg(unix)]
impl Drop for StdinNonblockGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::fcntl(self.fd, libc::F_SETFL, self.old_flags);
        }
    }
}
