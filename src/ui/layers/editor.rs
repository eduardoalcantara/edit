use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::editor::EditorCommand;
use crate::input::{keyboard, mouse};
use crate::modal::Modal;
use crate::theme::ThemePalette;
use crate::terminal::{
    render_terminal_bottom_row, render_terminal_divider, terminal_reserved_rows,
};
use crate::view_state::EditorBorder;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

pub struct EditorLayer;

impl UiLayer for EditorLayer {
    fn id(&self) -> LayerId {
        LayerId::Editor
    }

    fn is_visible(&self, _: &crate::app::App) -> bool {
        true
    }

    fn captures_input(&self, _: &crate::app::App) -> bool {
        false
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        app: &mut crate::app::App,
        layout: UiLayout,
        palette: ThemePalette,
    ) {
        let shell = layout.shell;
        let border_visible = app.view.border == EditorBorder::Visible;
        let terminal_block = layout.terminal_panel_rows.map(|rows| {
            terminal_reserved_rows(shell, rows)
        });
        let text_viewport = crate::editor::editor_viewport_rect(
            shell,
            app.view.border,
            terminal_block,
            app.view.margin,
        );
        let (_, _, left_margin, _) = app.view.margin.insets();
        let left_inset = if border_visible { 1u16 } else { 0 };
        let gutter_w = if app.view.show_line_numbers {
            let line_count = app.editor.engine().text.len_lines().max(1);
            crate::editor::line_numbers::layout(line_count, app.view.margin).total_width as u16
        } else {
            0
        };
        let content = text_viewport;
        if let Some(col) = app.view.guide_column.column() {
            let guide_x = shell
                .x
                .saturating_add(left_inset)
                .saturating_add(left_margin as u16)
                .saturating_add(gutter_w)
                .saturating_add(col as u16);
            if guide_x < shell.x.saturating_add(shell.width) {
                frame.render_widget(
                    Paragraph::new(" ").style(
                        Style::default()
                            .fg(palette.border)
                            .bg(palette.editor_bg),
                    ),
                    ratatui::layout::Rect {
                        x: guide_x,
                        y: content.y,
                        width: 1,
                        height: content.height,
                    },
                );
            }
        }
        let title = app.document_title();
        let show_cursor = !app.menu_state.is_open()
            && !app.modal.is_active()
            && app.input_focus == crate::view_state::InputFocus::Editor;
        app.editor.render(
            frame,
            shell,
            &title,
            palette,
            app.view.margin,
            app.view.border,
            terminal_block,
            Some(text_viewport),
            show_cursor,
            app.view.show_tabs,
            app.view.show_line_numbers,
        );

        if let Some(div_y) = layout.terminal_divider_y {
            let border_style = Style::default()
                .fg(palette.border)
                .bg(palette.editor_bg);
            let title_style = Style::default()
                .fg(palette.editor_fg)
                .bg(palette.editor_bg);
            render_terminal_divider(
                frame,
                shell,
                div_y,
                app.terminal.active_label(),
                border_visible,
                border_style,
                title_style,
            );
            render_terminal_bottom_row(
                frame,
                shell,
                shell.y.saturating_add(shell.height.saturating_sub(1)),
                border_visible,
                border_style,
            );
        }
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if app.menu_state.is_open() || app.modal.is_active() {
            return InputResult::Unhandled;
        }
        if app.input_focus == crate::view_state::InputFocus::Terminal {
            return InputResult::Unhandled;
        }

        let ctrl = key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL);

        if ctrl
            && key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
            && matches!(key.code, crossterm::event::KeyCode::Char('S' | 's'))
        {
            app.request_save_as();
            return InputResult::Consumed;
        }

        if ctrl
            && key.modifiers.contains(crossterm::event::KeyModifiers::ALT)
            && matches!(key.code, crossterm::event::KeyCode::Char('S' | 's'))
        {
            app.save_all_dirty();
            return InputResult::Consumed;
        }

        if ctrl
            && key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
            && matches!(key.code, crossterm::event::KeyCode::Char('W' | 'w'))
        {
            app.request_close_all();
            return InputResult::Consumed;
        }

        if key.modifiers.contains(crossterm::event::KeyModifiers::ALT)
            && matches!(key.code, crossterm::event::KeyCode::Char(c) if c.is_ascii_digit())
        {
            let n = match key.code {
                crossterm::event::KeyCode::Char('0') => 0,
                crossterm::event::KeyCode::Char(d) => d.to_digit(10).unwrap_or(1) as usize,
                _ => 1,
            };
            app.focus_tab_by_menu_number(n);
            return InputResult::Consumed;
        }

        if ctrl && matches!(key.code, crossterm::event::KeyCode::Tab) {
            if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                app.focus_tab_relative(-1);
            } else {
                app.focus_tab_relative(1);
            }
            return InputResult::Consumed;
        }

        if ctrl
            && key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
            && matches!(key.code, crossterm::event::KeyCode::Char('V' | 'v'))
        {
            if let Some(text) = app
                .clipboard
                .get(1)
                .or_else(|| app.clipboard.get(0))
                .map(str::to_string)
            {
                app.editor.execute(EditorCommand::Paste(text));
                app.set_status("Colar anterior");
            }
            return InputResult::Consumed;
        }

        if ctrl {
            return handle_ctrl_key(app, key);
        }

        if app.input_focus == crate::view_state::InputFocus::Editor {
            if matches!(key.code, crossterm::event::KeyCode::PageUp | crossterm::event::KeyCode::PageDown)
            {
                let cmd = if key.code == crossterm::event::KeyCode::PageUp {
                    EditorCommand::PageUp
                } else {
                    EditorCommand::PageDown
                };
                app.editor.execute(cmd);
                return InputResult::Consumed;
            }
        }

        if let Some(cmd) = keyboard::key_to_command(key) {
            app.editor.execute(cmd);
            return InputResult::Consumed;
        }

        InputResult::Unhandled
    }

    fn on_mouse(&self, mouse: MouseEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if !app.mouse_enabled {
            return InputResult::Unhandled;
        }
        let content = app.editor.content_area();
        if !mouse::is_in_editor(&mouse, content) {
            return InputResult::Unhandled;
        }
        mouse::handle_editor_mouse(app, mouse);
        InputResult::Consumed
    }
}

fn handle_ctrl_key(app: &mut crate::app::App, key: KeyEvent) -> InputResult {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('q' | 'Q') => app.request_quit(),
        KeyCode::Char('s' | 'S') => app.request_save(),
        KeyCode::Char('o' | 'O') => app.request_open(),
        KeyCode::Char('n' | 'N') => app.request_new_document(),
        KeyCode::Char('t' | 'T' | '\'') => app.chord_terminal_toggle(),
        KeyCode::Char('w' | 'W') => app.request_close(),
        KeyCode::Char('z' | 'Z') => app.editor.execute(EditorCommand::Undo),
        KeyCode::Char('y' | 'Y') => app.editor.execute(EditorCommand::Redo),
        KeyCode::Char('a' | 'A') => app.editor.execute(EditorCommand::SelectAll),
        KeyCode::Char('f' | 'F') => app.modal = Modal::find("Buscar", &app.find_pattern),
        KeyCode::Char('h' | 'H') => {
            app.modal = Modal::find_replace("Substituir", &app.find_pattern, "");
        }
        KeyCode::Char('c' | 'C') => {
            if app.editor.copy_selection(&mut app.clipboard) {
                app.set_status("Copiado");
            } else {
                app.set_status("Nada selecionado");
            }
        }
        KeyCode::Char('x' | 'X') => {
            if app.editor.cut_selection(&mut app.clipboard) {
                app.set_status("Recortado");
            } else {
                app.set_status("Nada selecionado");
            }
        }
        KeyCode::Char('v' | 'V') => {
            if let Some(text) = app.clipboard.paste_text() {
                app.editor.execute(EditorCommand::Paste(text));
                app.set_status("Colado");
            }
        }
        KeyCode::Left => {
            let extend = key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT);
            app.editor.execute(EditorCommand::MoveWordLeft { extend });
        }
        KeyCode::Right => {
            let extend = key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT);
            app.editor.execute(EditorCommand::MoveWordRight { extend });
        }
        _ => {
            if let Some(cmd) = keyboard::key_to_command(key) {
                app.editor.execute(cmd);
            }
        }
    }
    InputResult::Consumed
}
