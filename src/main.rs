mod app;
mod app_editor_split;
mod app_reference_pane;
mod app_workspace;
mod cli;
mod clipboard;
mod config;
mod document;
mod edit_mode;
mod editor;
mod editor_split;
mod encoding;
mod events;
mod file_io;
mod input;
mod memory;
mod menus;
mod modal;
mod platform;
mod recent;
mod reference_pane;
mod session;
mod terminal;
mod theme;
mod workspace;
mod ui;
mod view_state;
mod widgets;

use std::env;
use std::io::{self, stdout, IsTerminal, Write};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use cli::{CliError, LaunchOptions};

pub struct TerminalGuard {
    mouse_enabled: bool,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<(Terminal<CrosstermBackend<io::Stdout>>, Self)> {
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

    pub fn suspend(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        disable_raw_mode()?;
        let mut out = stdout();
        if self.mouse_enabled {
            let _ = execute!(out, DisableMouseCapture);
        }
        execute!(out, LeaveAlternateScreen)?;
        #[cfg(unix)]
        writeln!(
            out,
            "Editor suspenso — digite fg no shell para retomar (ou Ctrl+Shift+Alt+E)"
        )?;
        #[cfg(not(unix))]
        writeln!(
            out,
            "Editor suspenso — Ctrl+Shift+Alt+E para retomar"
        )?;
        out.flush()?;
        #[cfg(unix)]
        unsafe {
            libc::raise(libc::SIGTSTP);
        }
        let _ = terminal;
        Ok(())
    }

    pub fn resume(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        if self.mouse_enabled {
            let _ = execute!(out, EnableMouseCapture);
        }
        terminal.clear()?;
        if let Ok(size) = terminal.size() {
            let _ = terminal.resize(size.into());
        }
        Ok(())
    }

    pub fn leave(self) -> io::Result<()> {
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

fn run_app(opts: &LaunchOptions) -> io::Result<()> {
    let (mut terminal, guard) = TerminalGuard::enter()?;
    let mut app = App::new(guard.mouse_enabled, true);
    app.open_cli_files(&opts.files);

    let result = app.run(&mut terminal, &guard);
    app.shutdown();

    guard.leave()?;
    result
}

fn main() {
    let program = env::args()
        .next()
        .unwrap_or_else(|| "edit".to_string());

    let opts = match cli::parse_args(env::args()) {
        Ok(opts) => opts,
        Err(CliError::HelpRequested) => {
            cli::print_help(&program);
            return;
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    if let Err(error) = run_app(&opts) {
        eprintln!("Erro fatal: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod cli_tests {
    use super::cli;

    #[test]
    fn help_flag_exits_cleanly() {
        let err = cli::parse_args(["edit".to_string(), "--help".to_string()].into_iter());
        assert!(matches!(err, Err(cli::CliError::HelpRequested)));
    }
}
