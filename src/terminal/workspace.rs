//! Workspace de sessões de terminal (até 10).

use std::path::PathBuf;

use ratatui::layout::Rect;

use crate::clipboard::Clipboard;

use super::selection::TerminalSelection;
use super::session::{shell_label_for_spawn, TerminalSession};

pub const MAX_SESSIONS: usize = 10;
pub const SIDEBAR_COLS: u16 = 16;

/// Ação de clique na coluna de sessões.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarClick {
    NewSession,
    GrowPanel,
    ShrinkPanel,
    FocusSession(usize),
    CloseSession(usize),
    ClosePanel,
    ToggleColorScheme,
}

pub struct TerminalWorkspace {
    pub sessions: Vec<TerminalSession>,
    pub active: usize,
    /// Botão da sidebar sob o cursor (para destaque e ajuda no rodapé).
    pub sidebar_hover: Option<SidebarClick>,
    pub selection: Option<TerminalSelection>,
    pub selection_drag: bool,
}

impl Default for TerminalWorkspace {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            active: 0,
            sidebar_hover: None,
            selection: None,
            selection_drag: false,
        }
    }
}

impl TerminalWorkspace {
    pub fn active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active)
    }

    pub fn active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        let idx = self.active;
        self.sessions.get_mut(idx)
    }

    pub fn active_label(&self) -> &str {
        self.sessions
            .get(self.active)
            .map(|s| s.label.as_str())
            .unwrap_or("shell")
    }

    pub fn drain_all(&mut self) {
        for session in &mut self.sessions {
            session.drain_output();
        }
    }

    /// Índices das sessões cujo shell terminou (ordenados decrescente para remoção segura).
    pub fn exited_session_indices(&mut self) -> Vec<usize> {
        let mut exited: Vec<usize> = self
            .sessions
            .iter_mut()
            .enumerate()
            .filter_map(|(i, session)| session.has_exited().then_some(i))
            .collect();
        exited.sort_by(|a, b| b.cmp(a));
        exited
    }

    pub fn spawn_session(&mut self, cwd: PathBuf, cols: u16, rows: u16) -> Result<(), String> {
        if self.sessions.len() >= MAX_SESSIONS {
            return Err("Limite de sessões de terminal atingido".to_string());
        }
        let id = format!("term-{}", self.sessions.len() + 1);
        let session = TerminalSession::spawn(id, cwd, cols.max(1), rows.max(1))
            .map_err(|e| e.to_string())?;
        self.sessions.push(session);
        self.active = self.sessions.len() - 1;
        Ok(())
    }

    pub fn ensure_session(&mut self, cwd: PathBuf, cols: u16, rows: u16) {
        if self.sessions.is_empty() {
            let _ = self.spawn_session(cwd, cols, rows);
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.active = index;
        }
    }

    pub fn close_session(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }
        self.sessions.remove(index);
        if self.sessions.is_empty() {
            self.active = 0;
        } else if self.active >= self.sessions.len() {
            self.active = self.sessions.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }
    }

    pub fn close_active(&mut self) {
        let idx = self.active;
        self.close_session(idx);
    }

    pub fn resize_active(&mut self, cols: u16, rows: u16) {
        if let Some(session) = self.active_session_mut() {
            session.resize(cols, rows);
        }
    }

    pub fn write_active(&mut self, data: &[u8]) {
        if let Some(session) = self.active_session_mut() {
            session.write_bytes(data);
        }
    }

    pub fn scroll_active_page(&mut self, delta: isize, visible_height: usize) {
        if let Some(session) = self.active_session_mut() {
            session.scroll_page(delta, visible_height);
        }
    }

    pub fn scroll_active_to_tail(&mut self) {
        if let Some(session) = self.active_session_mut() {
            session.scroll_to_tail();
        }
    }

    pub fn begin_selection(&mut self, anchor: super::selection::TextCoord) {
        self.selection = Some(TerminalSelection {
            anchor,
            cursor: anchor,
        });
        self.selection_drag = true;
    }

    pub fn extend_selection(&mut self, cursor: super::selection::TextCoord) {
        if let Some(sel) = &mut self.selection {
            sel.cursor = cursor;
        }
    }

    pub fn finish_selection(&mut self) {
        self.selection_drag = false;
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_drag = false;
    }

    pub fn copy_selection(&self, clipboard: &mut Clipboard) -> bool {
        let Some(sel) = self.selection else {
            return false;
        };
        let Some(session) = self.sessions.get(self.active) else {
            return false;
        };
        let text = session.extract_selection(sel);
        if text.is_empty() {
            return false;
        }
        clipboard.push(text);
        true
    }

    pub fn shutdown(&mut self) {
        self.sessions.clear();
        self.clear_selection();
    }
}

/// Regiões do painel terminal dentro da shell unificada.
#[derive(Debug, Clone, Copy)]
pub struct TerminalPanelLayout {
    pub outer: Rect,
    pub output: Rect,
    pub sidebar: Rect,
}

/// Largura efetiva da coluna de sessões (alinhada ao divisor da shell).
pub fn effective_sidebar_width(shell_width: u16) -> u16 {
    SIDEBAR_COLS.min(shell_width.saturating_sub(8))
}

/// Coluna do divisor vertical `│` / junção `┬` (a sidebar começa na coluna seguinte).
pub fn terminal_split_col(shell: Rect, border_visible: bool) -> u16 {
    let inner_left = u16::from(border_visible);
    let inner_right = u16::from(border_visible);
    let sidebar_w = effective_sidebar_width(shell.width);
    let inner_w = shell.width.saturating_sub(inner_left + inner_right);
    let output_w = inner_w.saturating_sub(sidebar_w).saturating_sub(1);
    shell.x.saturating_add(inner_left).saturating_add(output_w)
}

pub fn layout_terminal_panel(
    shell: Rect,
    term_outer: Rect,
    border_visible: bool,
) -> TerminalPanelLayout {
    let sidebar_w = effective_sidebar_width(shell.width);
    let inner_left = u16::from(border_visible);
    let inner_right = u16::from(border_visible);
    let content_y = term_outer.y.saturating_add(1);
    let content_h = term_outer.height.saturating_sub(2);
    let inner_x = shell.x.saturating_add(inner_left);
    let inner_w = shell.width.saturating_sub(inner_left + inner_right);
    let output_w = inner_w.saturating_sub(sidebar_w).saturating_sub(1);
    let split_x = inner_x.saturating_add(output_w);
    let output = Rect {
        x: inner_x,
        y: content_y,
        width: output_w,
        height: content_h,
    };
    let sidebar = Rect {
        x: split_x.saturating_add(1),
        y: content_y,
        width: sidebar_w,
        height: content_h,
    };
    TerminalPanelLayout {
        outer: Rect {
            x: inner_x,
            y: content_y,
            width: inner_w,
            height: content_h,
        },
        output,
        sidebar,
    }
}

pub fn default_spawn_cwd(document_path: Option<&std::path::Path>) -> PathBuf {
    let raw = if let Some(path) = document_path {
        if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            } else if parent.is_absolute() {
                parent.to_path_buf()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(parent)
            }
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    shell_spawn_path(raw)
}

/// Caminho legível pelo `cmd.exe` — sem prefixo `\\?\` de canonicalize.
fn shell_spawn_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{}", rest));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    path
}

pub fn next_session_label_hint() -> String {
    shell_label_for_spawn()
}

pub fn sidebar_button_help(action: SidebarClick) -> &'static str {
    match action {
        SidebarClick::NewSession => "Nova sessão de terminal",
        SidebarClick::GrowPanel => "Aumentar altura do painel",
        SidebarClick::ShrinkPanel => "Diminuir altura do painel",
        SidebarClick::ClosePanel => "Fechar painel de terminal",
        SidebarClick::ToggleColorScheme => "Alternar cores do output (tema / clássico)",
        SidebarClick::FocusSession(_) => "Selecionar sessão de terminal",
        SidebarClick::CloseSession(_) => "Fechar sessão de terminal",
    }
}

pub fn sidebar_click(
    sidebar: Rect,
    x: u16,
    y: u16,
    session_count: usize,
) -> Option<SidebarClick> {
    if sidebar.width == 0 || sidebar.height == 0 {
        return None;
    }
    if x < sidebar.x
        || x >= sidebar.x.saturating_add(sidebar.width)
        || y < sidebar.y
        || y >= sidebar.y.saturating_add(sidebar.height)
    {
        return None;
    }
    let rel_y = y.saturating_sub(sidebar.y);
    let rel_x = x.saturating_sub(sidebar.x);
    let w = sidebar.width as usize;

    if rel_y == 0 {
        if w >= 3 && rel_x as usize >= w.saturating_sub(3) {
            return Some(SidebarClick::ClosePanel);
        }
        if w >= 6
            && (rel_x as usize) >= w.saturating_sub(6)
            && (rel_x as usize) < w.saturating_sub(3)
        {
            return Some(SidebarClick::ToggleColorScheme);
        }
        if rel_x < 3 {
            return Some(SidebarClick::NewSession);
        }
        if rel_x < 6 {
            return Some(SidebarClick::GrowPanel);
        }
        if rel_x < 9 {
            return Some(SidebarClick::ShrinkPanel);
        }
        return None;
    }
    let session_row = rel_y as usize - 1;
    if session_row < session_count {
        if w >= 3 && rel_x as usize >= w.saturating_sub(3) {
            Some(SidebarClick::CloseSession(session_row))
        } else {
            Some(SidebarClick::FocusSession(session_row))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod sidebar_tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn sidebar_click_new_session_on_n_button() {
        let area = Rect::new(10, 5, 16, 6);
        assert_eq!(
            sidebar_click(area, 10, 5, 0),
            Some(SidebarClick::NewSession)
        );
    }

    #[test]
    fn sidebar_click_grow_and_shrink_panel() {
        let area = Rect::new(0, 0, 16, 6);
        assert_eq!(sidebar_click(area, 3, 0, 0), Some(SidebarClick::GrowPanel));
        assert_eq!(sidebar_click(area, 6, 0, 0), Some(SidebarClick::ShrinkPanel));
    }

    #[test]
    fn sidebar_click_toggle_colors_on_c_button() {
        let area = Rect::new(0, 0, 16, 6);
        assert_eq!(
            sidebar_click(area, 10, 0, 0),
            Some(SidebarClick::ToggleColorScheme)
        );
    }

    #[test]
    fn sidebar_click_close_panel_on_f_button() {
        let area = Rect::new(0, 0, 16, 6);
        assert_eq!(
            sidebar_click(area, 13, 0, 1),
            Some(SidebarClick::ClosePanel)
        );
    }

    #[test]
    fn sidebar_width_matches_split() {
        let shell = Rect::new(0, 0, 80, 24);
        let term_outer =
            crate::terminal::terminal_panel_outer(shell, crate::terminal::TERMINAL_PANEL_ROWS_DEFAULT);
        let panel = layout_terminal_panel(shell, term_outer, true);
        assert_eq!(panel.sidebar.width, 16);
        assert_eq!(panel.output.width + 1 + panel.sidebar.width, 78);
        assert_eq!(panel.sidebar.x, panel.output.x + panel.output.width + 1);
    }

    #[test]
    fn sidebar_click_close_session_on_q_button() {
        let area = Rect::new(0, 0, 16, 6);
        assert_eq!(
            sidebar_click(area, 13, 2, 2),
            Some(SidebarClick::CloseSession(1))
        );
        assert_eq!(
            sidebar_click(area, 14, 2, 2),
            Some(SidebarClick::CloseSession(1))
        );
    }

    #[test]
    fn shell_spawn_path_strips_extended_prefix_on_windows() {
        #[cfg(windows)]
        {
            let path = shell_spawn_path(PathBuf::from(r"\\?\D:\proj\edit"));
            assert_eq!(path, PathBuf::from(r"D:\proj\edit"));
        }
    }
}
