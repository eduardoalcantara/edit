mod app;
mod app_editor_split;
mod app_env_pane;
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
mod env_editor;
mod env_pane;
mod events;
mod file_io;
mod input;
mod memory;
mod menus;
mod modal;
mod paths;
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
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::cursor::Show;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::style::{force_color_output, ResetColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use cli::{CliError, LaunchOptions};

/// Evita restaurar o TTY duas vezes (leave explícito + Drop).
static TERMINAL_RESTORED: AtomicBool = AtomicBool::new(true);

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

        // Temas Turbo Vision exigem cor; NO_COLOR no ambiente (ex. Cursor/CI) não deve
        // apagar a paleta do editor.
        force_color_output(true);

        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;

        let mouse_enabled = execute!(out, EnableMouseCapture).is_ok();
        TERMINAL_RESTORED.store(false, Ordering::SeqCst);

        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;

        Ok((terminal, Self { mouse_enabled }))
    }

    pub fn leave_ui_briefly(
        &self,
        _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        Self::teardown_ui(self.mouse_enabled)?;
        Ok(())
    }

    pub fn resume(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        force_color_output(true);
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        if self.mouse_enabled {
            let _ = execute!(out, EnableMouseCapture);
        }
        TERMINAL_RESTORED.store(false, Ordering::SeqCst);
        terminal.clear()?;
        if let Ok(size) = terminal.size() {
            let _ = terminal.resize(size.into());
        }
        Ok(())
    }

    pub fn leave(self) -> io::Result<()> {
        Self::restore_host_terminal(self.mouse_enabled)
    }

    fn teardown_ui(mouse_enabled: bool) -> io::Result<()> {
        // Sai de raw mode primeiro para o shell/host não engolir CSI.
        let _ = disable_raw_mode();
        let mut out = stdout();
        if mouse_enabled {
            let _ = execute!(out, DisableMouseCapture);
        }
        // Ordem: reset visual → sair da alternate screen → cursor visível.
        let _ = execute!(out, ResetColor, Show, LeaveAlternateScreen);
        out.flush()?;
        Ok(())
    }

    fn restore_host_terminal(mouse_enabled: bool) -> io::Result<()> {
        if TERMINAL_RESTORED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let _ = disable_raw_mode();
        let mut out = stdout();
        if mouse_enabled {
            let _ = execute!(out, DisableMouseCapture);
        }
        // Desliga modos de rato SGR/X10 que alguns emuladores deixam activos.
        let _ = write!(
            out,
            "\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l\x1b[?2004l"
        );
        let _ = execute!(out, ResetColor, Show, LeaveAlternateScreen);
        out.flush()?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = Self::restore_host_terminal(self.mouse_enabled);
    }
}

fn run_app(opts: &LaunchOptions) -> io::Result<()> {
    let (mut terminal, guard) = TerminalGuard::enter()?;
    let mut app = App::new(guard.mouse_enabled, true);
    app.open_cli_files(&opts.files);

    let result = app.run(&mut terminal, &guard);
    // Encerra PTYs ainda dentro da alternate screen para não derramar CSI no shell.
    app.shutdown();
    let leave = guard.leave();
    match (result, leave) {
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
        (Ok(()), Ok(())) => Ok(()),
    }
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
