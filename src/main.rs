mod app;
mod app_workspace;
mod clipboard;
mod config;
mod document;
mod edit_mode;
mod editor;
mod encoding;
mod events;
mod file_io;
mod input;
mod memory;
mod menus;
mod modal;
mod recent;
mod session;
mod theme;
mod workspace;
mod ui;
mod view_state;
mod widgets;

use std::io::{self, stdout, IsTerminal, Write};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

struct TerminalGuard {
    mouse_enabled: bool,
}

impl TerminalGuard {
    fn enter() -> io::Result<(Terminal<CrosstermBackend<io::Stdout>>, Self)> {
        if !stdout().is_terminal() {
            eprintln!("Erro: Editor Linux requer um terminal interativo (TTY).");
            eprintln!("Execute diretamente em um terminal local ou via SSH com TTY alocado.");
            std::process::exit(1);
        }

        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;

        let mouse_enabled = execute!(out, EnableMouseCapture).is_ok();

        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;

        Ok((terminal, Self { mouse_enabled }))
    }

    fn leave(self) -> io::Result<()> {
        disable_raw_mode()?;
        let mut out = stdout();
        if self.mouse_enabled {
            let _ = execute!(out, DisableMouseCapture);
        }
        execute!(out, LeaveAlternateScreen)?;
        out.flush()?;
        Ok(())
    }
}

fn run_app() -> io::Result<()> {
    let (mut terminal, guard) = TerminalGuard::enter()?;
    let mut app = App::new(guard.mouse_enabled);

    let result = app.run(&mut terminal);
    app.shutdown();

    guard.leave()?;
    result
}

fn main() {
    if let Err(error) = run_app() {
        let _ = restore_terminal_on_error();
        eprintln!("Erro: {error}");
        std::process::exit(1);
    }
}

fn restore_terminal_on_error() -> io::Result<()> {
    disable_raw_mode()?;
    let mut out = stdout();
    let _ = execute!(out, DisableMouseCapture);
    execute!(out, LeaveAlternateScreen)?;
    out.flush()?;
    Ok(())
}
