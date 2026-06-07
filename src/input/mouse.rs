use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::app::App;
use crate::editor::EditorCommand;

pub fn terminal_to_doc(mouse: &MouseEvent, inner: Rect) -> Option<(usize, usize)> {
    if mouse.column < inner.x
        || mouse.column >= inner.x.saturating_add(inner.width)
        || mouse.row < inner.y
        || mouse.row >= inner.y.saturating_add(inner.height)
    {
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
    let inner = app.editor.inner_area();
    let Some((vp_line, vp_col)) = terminal_to_doc(&mouse, inner) else {
        return;
    };
    let (line, col) = app.editor.viewport_to_doc(vp_line, vp_col);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                app.editor
                    .execute(EditorCommand::StartBlockSelect { line, col });
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
                app.editor
                    .execute(EditorCommand::UpdateBlockSelect { line, col });
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

pub fn is_in_editor(mouse: &MouseEvent, inner: Rect) -> bool {
    terminal_to_doc(mouse, inner).is_some()
}
