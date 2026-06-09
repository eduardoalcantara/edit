mod app;
mod app_editor_split;
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
mod recent;
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

fn run_app(opts: &LaunchOptions) -> io::Result<()> {
    let (mut terminal, guard) = TerminalGuard::enter()?;
    let mut app = App::new(guard.mouse_enabled, opts.files.is_empty());
    app.open_cli_files(&opts.files);

    let result = app.run(&mut terminal);
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
            eprintln!("Erro: {error}");
            eprintln!("Use {program} --help para ajuda.");
            std::process::exit(1);
        }
    };

    if let Err(error) = cli::prepare_launch(&opts) {
        eprintln!("Erro: {error}");
        std::process::exit(1);
    }

    if let Err(error) = run_app(&opts) {
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
