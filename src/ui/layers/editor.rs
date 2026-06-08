use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::editor::EditorCommand;
use crate::input::{keyboard, mouse};
use crate::modal::Modal;
use crate::theme::ThemePalette;
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
        let area = layout.editor;
        let terminal_below = layout.terminal.is_some();
        let border_visible = app.view.border == EditorBorder::Visible;
        let (_, _, left_margin, _) = app.view.margin.insets();
        let left_inset = if border_visible { 1u16 } else { 0 };
        let top_inset = 1u16;
        let bottom_inset = if border_visible || terminal_below { 1u16 } else { 0 };
        if let Some(col) = app.view.guide_column.column() {
            let guide_x = area
                .x
                .saturating_add(left_inset)
                .saturating_add(left_margin as u16)
                .saturating_add(col as u16);
            if guide_x < area.x.saturating_add(area.width) {
                frame.render_widget(
                    Paragraph::new(" ").style(
                        Style::default()
                            .fg(palette.border)
                            .bg(palette.editor_bg),
                    ),
                    ratatui::layout::Rect {
                        x: guide_x,
                        y: area.y.saturating_add(top_inset),
                        width: 1,
                        height: area.height.saturating_sub(top_inset + bottom_inset),
                    },
                );
            }
        }
        let title = app.document_title();
        let show_cursor = !app.menu_state.is_open() && !app.modal.is_active();
        app.editor.render(
            frame,
            area,
            &title,
            palette,
            app.view.margin,
            app.view.border,
            terminal_below,
            show_cursor,
            app.view.show_tabs,
        );
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if app.menu_state.is_open() || app.modal.is_active() {
            return InputResult::Unhandled;
        }
        if key.code == crossterm::event::KeyCode::F(1) {
            app.set_status("Ajuda: em breve");
            return InputResult::Consumed;
        }
        if key.code == crossterm::event::KeyCode::F(2) {
            app.request_save();
            return InputResult::Consumed;
        }
        if key.code == crossterm::event::KeyCode::F(3) {
            if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                app.find_prev();
            } else {
                app.find_next();
            }
            return InputResult::Consumed;
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
        let inner = app.editor.inner_area();
        if !mouse::is_in_editor(&mouse, inner) {
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
