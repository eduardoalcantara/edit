use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::EditorCommand;

pub fn key_to_command(key: KeyEvent) -> Option<EditorCommand> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let extend = key.modifiers.contains(KeyModifiers::SHIFT);

    if ctrl {
        match key.code {
            KeyCode::Home => {
                return Some(EditorCommand::DocumentStart { extend });
            }
            KeyCode::End => return Some(EditorCommand::DocumentEnd { extend }),
            _ => return None,
        }
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

pub fn is_suspend_screen_chord(key: &KeyEvent) -> bool {
    if !crate::platform::terminal_suspend_to_shell_supported() {
        return false;
    }
    use crossterm::event::KeyEventKind;
    key.kind == KeyEventKind::Press
        && matches!(key.code, KeyCode::Char('e' | 'E'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && key.modifiers.contains(KeyModifiers::SHIFT)
        && key.modifiers.contains(KeyModifiers::ALT)
}
