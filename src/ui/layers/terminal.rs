use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use crate::input::mouse::{point_in_rect, TERMINAL_WHEEL_LINES};
use crate::terminal::{
    default_spawn_cwd, key_event_to_pty_bytes, mouse_to_coord, paint_terminal_panel,
    sidebar_button_help, sidebar_click, terminal_panel_outer, SidebarClick,
};
use crate::theme::ThemePalette;
use crate::view_state::InputFocus;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

pub struct TerminalLayer;

impl UiLayer for TerminalLayer {
    fn id(&self) -> LayerId {
        LayerId::Terminal
    }

    fn is_visible(&self, app: &crate::app::App) -> bool {
        app.view.terminal
    }

    fn captures_input(&self, app: &crate::app::App) -> bool {
        app.view.terminal && app.input_focus == InputFocus::Terminal
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        app: &mut crate::app::App,
        layout: UiLayout,
        palette: ThemePalette,
    ) {
        let Some(panel) = layout.terminal else {
            return;
        };
        let rows = layout
            .terminal_panel_rows
            .unwrap_or(crate::terminal::TERMINAL_PANEL_ROWS_DEFAULT);
        let term_outer = terminal_panel_outer(layout.shell, rows);
        let active = app.terminal.active;
        let show_cursor = app.input_focus == InputFocus::Terminal;
        let output_scheme = app.view.terminal_color_scheme;
        paint_terminal_panel(
            frame,
            layout.shell,
            term_outer,
            panel,
            &mut app.terminal,
            palette,
            active,
            show_cursor,
            output_scheme,
        );
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, layout: UiLayout) -> InputResult {
        if !self.captures_input(app) {
            return InputResult::Unhandled;
        }
        use crossterm::event::KeyModifiers;
        if key.modifiers.intersects(KeyModifiers::ALT | KeyModifiers::SUPER) {
            return InputResult::Unhandled;
        }
        if key.code == KeyCode::Esc {
            app.input_focus = InputFocus::Editor;
            app.set_status("Editor: foco");
            return InputResult::Consumed;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C'))
        {
            if app.terminal.copy_selection(&mut app.clipboard) {
                app.set_status("Terminal: copiado");
                return InputResult::Consumed;
            }
        }
        if key.code == KeyCode::End {
            app.terminal.scroll_active_to_tail();
            return InputResult::Consumed;
        }
        if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown) {
            let Some(panel) = layout.terminal else {
                return InputResult::Consumed;
            };
            let visible = panel.output.height as usize;
            let delta = if key.code == KeyCode::PageUp {
                visible as isize
            } else {
                -(visible as isize)
            };
            app.terminal.scroll_active_page(delta, visible);
            return InputResult::Consumed;
        }
        if let Some(bytes) = key_event_to_pty_bytes(key) {
            app.terminal.write_active(&bytes);
            return InputResult::Consumed;
        }
        InputResult::Unhandled
    }

    fn on_mouse(
        &self,
        mouse: MouseEvent,
        app: &mut crate::app::App,
        layout: UiLayout,
    ) -> InputResult {
        if !app.view.terminal {
            return InputResult::Unhandled;
        }
        let Some(panel) = layout.terminal else {
            return InputResult::Unhandled;
        };

        if matches!(
            mouse.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        ) && point_in_rect(&mouse, panel.outer)
        {
            app.input_focus = InputFocus::Terminal;
            let visible = panel.output.height as usize;
            if mouse.kind == MouseEventKind::ScrollUp {
                app.terminal
                    .scroll_active_page(TERMINAL_WHEEL_LINES, visible);
            } else {
                app.terminal
                    .scroll_active_page(-TERMINAL_WHEEL_LINES, visible);
            }
            return InputResult::Consumed;
        }

        if mouse.kind == MouseEventKind::Moved {
            if app.terminal.selection_drag {
                if let Some(session) = app.terminal.sessions.get(app.terminal.active) {
                    if let Some(coord) = mouse_to_coord(session, panel.output, &mouse) {
                        app.terminal.extend_selection(coord);
                    }
                }
                return InputResult::Consumed;
            }
            let hover = if point_in_rect(&mouse, panel.sidebar) {
                sidebar_click(
                    panel.sidebar,
                    mouse.column,
                    mouse.row,
                    app.terminal.sessions.len(),
                )
            } else {
                None
            };
            if app.terminal.sidebar_hover != hover {
                app.terminal.sidebar_hover = hover;
            }
            return if hover.is_some() {
                InputResult::Consumed
            } else {
                InputResult::Unhandled
            };
        }

        if point_in_rect(&mouse, panel.output) {
            app.input_focus = InputFocus::Terminal;
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(session) = app.terminal.sessions.get(app.terminal.active) {
                        if let Some(coord) = mouse_to_coord(session, panel.output, &mouse) {
                            app.terminal.begin_selection(coord);
                        }
                    }
                    return InputResult::Consumed;
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    if let Some(session) = app.terminal.sessions.get(app.terminal.active) {
                        if let Some(coord) = mouse_to_coord(session, panel.output, &mouse) {
                            app.terminal.extend_selection(coord);
                        }
                    }
                    return InputResult::Consumed;
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    app.terminal.finish_selection();
                    return InputResult::Consumed;
                }
                _ => return InputResult::Consumed,
            }
        }

        if !matches!(
            mouse.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
        ) {
            return InputResult::Unhandled;
        }
        if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
            return InputResult::Unhandled;
        }

        if point_in_rect(&mouse, panel.sidebar) {
            app.input_focus = InputFocus::Terminal;
            if let Some(action) = sidebar_click(
                panel.sidebar,
                mouse.column,
                mouse.row,
                app.terminal.sessions.len(),
            ) {
                handle_sidebar_action(app, action, panel.output.width, panel.output.height);
            }
            return InputResult::Consumed;
        }

        InputResult::Unhandled
    }

    fn footer_hint(&self, app: &crate::app::App) -> Option<String> {
        app.terminal
            .sidebar_hover
            .map(sidebar_button_help)
            .map(str::to_string)
    }
}

fn close_session_at(app: &mut crate::app::App, index: usize, cols: u16, rows: u16) {
    app.terminal.close_session(index);
    app.terminal.clear_selection();
    if app.terminal.sessions.is_empty() {
        let cwd = default_spawn_cwd(app.document.path());
        let _ = app.terminal.spawn_session(cwd, cols, rows);
    }
    app.set_status("Terminal: sessão fechada");
}

fn handle_sidebar_action(
    app: &mut crate::app::App,
    action: SidebarClick,
    cols: u16,
    rows: u16,
) {
    match action {
        SidebarClick::NewSession => {
            let cwd = default_spawn_cwd(app.document.path());
            match app.terminal.spawn_session(cwd, cols, rows) {
                Ok(()) => app.set_status("Terminal: nova sessão"),
                Err(msg) => app.set_status(msg),
            }
        }
        SidebarClick::FocusSession(index) => {
            app.terminal.set_active(index);
            app.set_status(format!("Terminal: sessão {}", index + 1));
        }
        SidebarClick::CloseSession(index) => {
            close_session_at(app, index, cols, rows);
        }
        SidebarClick::GrowPanel => {
            app.grow_terminal_panel();
        }
        SidebarClick::ShrinkPanel => {
            app.shrink_terminal_panel();
        }
        SidebarClick::ClosePanel => {
            app.toggle_terminal_panel();
            app.set_status("Terminal: oculto");
        }
        SidebarClick::ToggleColorScheme => {
            app.toggle_terminal_color_scheme();
        }
    }
}
