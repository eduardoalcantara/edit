//! Capacidades dependentes do sistema operacional.

/// Suspende a TUI (sai da tela alternativa) e aguarda Ctrl+Shift+Alt+E para retomar.
/// Visível no menu apenas em Unix/Linux; o loop `ui_suspended` captura o chord.
pub fn terminal_suspend_to_shell_supported() -> bool {
    cfg!(unix)
}
