//! Capacidades dependentes do sistema operacional.

/// Suspende a TUI e devolve o prompt do shell (SIGTSTP + job control).
pub fn terminal_suspend_to_shell_supported() -> bool {
    cfg!(unix)
}
