use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::editor::EditorCommand;
use crate::editor_split::{pane_at_column, SplitPane};
use crate::input::{keyboard, mouse};
use crate::modal::Modal;
use crate::theme::ThemePalette;
use crate::terminal::{
    render_terminal_bottom_row, render_terminal_divider, terminal_reserved_rows,
};
use crate::view_state::EditorBorder;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;
use crate::widgets::panel::{self, PanelBorder};

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
        let pane_terminal = if app.split_active() {
            None
        } else {
            terminal_block
        };
        let show_cursor = !app.menu_state.is_open()
            && !app.modal.is_active()
            && app.input_focus == crate::view_state::InputFocus::Editor;

        if let Some(split) = layout.editor_split {
            let focused = app.editor_split.focused_pane;
            let left_idx = Some(app.editor_split.left_tab);
            let right_idx = app.editor_split.right_tab;
            let right_reference = app.editor_split.has_reference();

            if focused != SplitPane::Left && split.left.width > 0 {
                paint_pane(
                    app,
                    frame,
                    split.left,
                    left_idx,
                    palette,
                    pane_terminal,
                    false,
                    false,
                    PanelBorder::Plain,
                    SplitPane::Left,
                );
            }
            if focused != SplitPane::Right && split.right.width > 0 {
                paint_pane(
                    app,
                    frame,
                    split.right,
                    if right_reference { None } else { right_idx },
                    palette,
                    pane_terminal,
                    false,
                    false,
                    PanelBorder::Plain,
                    SplitPane::Right,
                );
            }

            let (active_area, active_tab, active_pane) = match focused {
                SplitPane::Left => (split.left, left_idx, SplitPane::Left),
                SplitPane::Right => (
                    split.right,
                    if right_reference { None } else { right_idx },
                    SplitPane::Right,
                ),
            };
            if active_area.width > 0 {
                paint_pane(
                    app,
                    frame,
                    active_area,
                    active_tab,
                    palette,
                    pane_terminal,
                    show_cursor,
                    true,
                    PanelBorder::Double,
                    active_pane,
                );
            }
        } else {
            let text_viewport = crate::editor::editor_viewport_rect(
                shell,
                app.view.border,
                terminal_block,
                app.view.margin,
            );
            paint_guide_column(app, frame, shell, text_viewport, palette, border_visible);
            let title = app.document_title();
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
                PanelBorder::Plain,
                None,
            );
        }

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

        if matches!(key.code, crossterm::event::KeyCode::Esc)
            && !key.modifiers.intersects(
                crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT,
            )
            && app.reference_pane_active()
        {
            app.close_reference_pane();
            return InputResult::Consumed;
        }

        if app.reference_pane_active()
            && matches!(key.code, crossterm::event::KeyCode::Char('f' | 'F'))
            && !key.modifiers.intersects(
                crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::ALT,
            )
        {
            app.close_reference_pane();
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

    fn on_mouse(&self, mouse: MouseEvent, app: &mut crate::app::App, layout: UiLayout) -> InputResult {
        if !app.mouse_enabled {
            return InputResult::Unhandled;
        }

        if let Some(hit) = app.reference_close_hit {
            if mouse::point_in_rect(&mouse, hit)
                && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            {
                app.close_reference_pane();
                return InputResult::Consumed;
            }
        }

        if let Some(split) = layout.editor_split {
            if let Some(pane) = pane_at_column(split, mouse.column) {
                if pane != app.editor_split.focused_pane {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            app.focus_editor_pane(pane);
                            return InputResult::Consumed;
                        }
                        _ => return InputResult::Unhandled,
                    }
                }
            }
        }

        let content = app.editor.content_area();
        if !mouse::is_in_editor(&mouse, content) {
            return InputResult::Unhandled;
        }
        mouse::handle_editor_mouse(app, mouse);
        InputResult::Consumed
    }
}

fn paint_pane(
    app: &mut crate::app::App,
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    tab_index: Option<usize>,
    palette: ThemePalette,
    terminal_block: Option<u16>,
    show_cursor: bool,
    use_active_editor: bool,
    pane_border: PanelBorder,
    pane: SplitPane,
) {
    let border_visible = app.view.border == EditorBorder::Visible;
    let text_viewport = crate::editor::editor_viewport_rect(
        area,
        app.view.border,
        terminal_block,
        app.view.margin,
    );
    let is_reference = pane == SplitPane::Right && app.editor_split.has_reference();
    let has_file_tab = tab_index.is_some_and(|index| index < app.workspace.tabs.len());
    if use_active_editor && has_file_tab && !is_reference {
        paint_guide_column(app, frame, area, text_viewport, palette, border_visible);
    }
    let title = if is_reference {
        app.reference_title()
            .unwrap_or_else(|| "Referência".to_string())
    } else {
        app.tab_pane_title(tab_index)
    };
    let trailing = if is_reference && use_active_editor {
        Some(app.reference_close_label())
    } else {
        None
    };
    let trailing_ref = trailing.as_deref();

    if use_active_editor && (is_reference || has_file_tab) {
        let action_hit = app.editor.render(
            frame,
            area,
            &title,
            palette,
            app.view.margin,
            app.view.border,
            terminal_block,
            Some(text_viewport),
            show_cursor,
            app.view.show_tabs,
            app.view.show_line_numbers,
            pane_border,
            trailing_ref,
        );
        if is_reference {
            app.set_reference_close_hit(action_hit);
        }
    } else if is_reference {
        if let Some(reference) = app.editor_split.reference.as_mut() {
            reference.editor.render(
                frame,
                area,
                &title,
                palette,
                app.view.margin,
                app.view.border,
                terminal_block,
                Some(text_viewport),
                false,
                app.view.show_tabs,
                app.view.show_line_numbers,
                pane_border,
                trailing_ref,
            );
        }
    } else if let Some(index) = tab_index {
        if index < app.workspace.tabs.len() {
            app.workspace.tabs[index].editor.render(
                frame,
                area,
                &title,
                palette,
                app.view.margin,
                app.view.border,
                terminal_block,
                Some(text_viewport),
                false,
                app.view.show_tabs,
                app.view.show_line_numbers,
                pane_border,
                None,
            );
        }
    } else {
        panel::render_editor_frame_with_border(
            frame,
            area,
            &title,
            palette.editor_text_style(),
            Style::default().fg(palette.border).bg(palette.editor_bg),
            Style::default().fg(palette.editor_fg).bg(palette.editor_bg),
            border_visible,
            terminal_block,
            pane_border,
        );
    }
}

fn paint_guide_column(
    app: &crate::app::App,
    frame: &mut ratatui::Frame<'_>,
    shell: Rect,
    text_viewport: Rect,
    palette: ThemePalette,
    border_visible: bool,
) {
    let (_, _, left_margin, _) = app.view.margin.insets();
    let left_inset = if border_visible { 1u16 } else { 0 };
    let gutter_w = if app.view.show_line_numbers {
        let line_count = app.editor.engine().text.len_lines().max(1);
        crate::editor::line_numbers::layout(line_count, app.view.margin).total_width as u16
    } else {
        0
    };
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
                    y: text_viewport.y,
                    width: 1,
                    height: text_viewport.height,
                },
            );
        }
    }
}

fn handle_ctrl_key(app: &mut crate::app::App, key: KeyEvent) -> InputResult {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('1') => {
            app.chord_editor_single_or_left();
        }
        KeyCode::Char('2') => {
            app.chord_editor_split_or_right();
        }
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
        KeyCode::Char('r' | 'R') => {
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
