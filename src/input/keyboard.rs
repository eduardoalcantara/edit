use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::EditorCommand;

pub fn key_to_command(key: KeyEvent) -> Option<EditorCommand> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let extend = key.modifiers.contains(KeyModifiers::SHIFT);

    if ctrl {
        return None;
    }

    match key.code {
        KeyCode::Enter => Some(EditorCommand::InsertChar('\n')),
        KeyCode::Char(ch) => Some(EditorCommand::InsertChar(ch)),
        KeyCode::Backspace => Some(EditorCommand::Backspace),
        KeyCode::Delete => Some(EditorCommand::Delete),
        KeyCode::Left => Some(EditorCommand::MoveLeft { extend }),
        KeyCode::Right => Some(EditorCommand::MoveRight { extend }),
        KeyCode::Up => Some(EditorCommand::MoveUp { extend }),
        KeyCode::Down => Some(EditorCommand::MoveDown { extend }),
        KeyCode::Home => Some(EditorCommand::Home { extend }),
        KeyCode::End => Some(EditorCommand::End { extend }),
        KeyCode::Tab => Some(EditorCommand::Tab),
        _ => None,
    }
}
