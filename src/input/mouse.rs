use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::app::App;
use crate::editor::EditorCommand;
use crate::view_state::InputFocus;

/// Linhas por notch da roda do mouse no editor.
pub const MOUSE_WHEEL_LINES: i32 = 3;

/// Linhas por notch da roda no scrollback do terminal.
pub const TERMINAL_WHEEL_LINES: isize = 3;

pub fn point_in_rect(mouse: &MouseEvent, area: Rect) -> bool {
    mouse.column >= area.x
        && mouse.column < area.x.saturating_add(area.width)
        && mouse.row >= area.y
        && mouse.row < area.y.saturating_add(area.height)
}

pub fn terminal_to_doc(mouse: &MouseEvent, inner: Rect) -> Option<(usize, usize)> {
    if !point_in_rect(mouse, inner) {
        return None;
    }
    let line = (mouse.row - inner.y) as usize;
    let col = (mouse.column - inner.x) as usize;
    Some((line, col))
}

pub fn viewport_to_doc(
    viewport_line: usize,
    viewport_col: usize,
    top_line: usize,
    left_col: usize,
) -> (usize, usize) {
    (top_line + viewport_line, left_col + viewport_col)
}

pub fn handle_editor_mouse(app: &mut App, mouse: MouseEvent) {
    let content = app.editor.content_area();
    let text = app.editor.text_area();

    if point_in_rect(&mouse, content) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                app.editor
                    .execute(EditorCommand::ScrollWheel {
                        delta: -MOUSE_WHEEL_LINES,
                    });
                return;
            }
            MouseEventKind::ScrollDown => {
                app.editor
                    .execute(EditorCommand::ScrollWheel {
                        delta: MOUSE_WHEEL_LINES,
                    });
                return;
            }
            _ => {}
        }
    }

    if !point_in_rect(&mouse, text) {
        if point_in_rect(&mouse, content)
            && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
        {
            app.input_focus = InputFocus::Editor;
        }
        return;
    }

    let Some((vp_line, vp_col)) = terminal_to_doc(&mouse, text) else {
        return;
    };
    let engine = app.editor.engine();
    let doc_line = (engine.viewport.top_line + vp_line)
        .min(engine.text.len_lines().saturating_sub(1));
    let vis_col = engine.viewport.left_col + vp_col;
    let (line, col) = app.editor.viewport_to_doc(vp_line, vp_col);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.input_focus = InputFocus::Editor;
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                app.editor
                    .execute(EditorCommand::StartBlockSelect {
                        line: doc_line,
                        col: vis_col,
                    });
                return;
            }
            if mouse.modifiers.contains(KeyModifiers::CONTROL) {
                app.editor.execute(EditorCommand::AddCursor { line, col });
                return;
            }
            app.editor.begin_mouse_select(line, col);
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.editor.is_block_dragging() {
                app.editor.execute(EditorCommand::UpdateBlockSelect {
                    line: doc_line,
                    col: vis_col,
                });
                return;
            }
            if app.editor.is_linear_dragging() {
                app.editor.drag_mouse_select(line, col);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app.editor.is_block_dragging() {
                app.editor.execute(EditorCommand::EndBlockSelect);
                app.set_status(format!(
                    "Modo: {}",
                    app.editor.selection_mode_label()
                ));
                return;
            }
            app.editor.finish_mouse_select();
        }
        _ => {}
    }
}

pub fn is_in_editor(mouse: &MouseEvent, content: Rect) -> bool {
    point_in_rect(mouse, content)
}
