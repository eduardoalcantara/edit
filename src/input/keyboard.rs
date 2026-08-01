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

pub fn is_env_variables_chord(key: &KeyEvent) -> bool {
    use crossterm::event::KeyEventKind;
    // Ctrl+Alt+V — Ctrl+Shift+V é Paste na maioria dos emuladores Linux (GNOME Terminal,
    // Tilix, etc.) e nunca chega à aplicação; o texto colado parecia “escrever” no editor.
    matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && matches!(key.code, KeyCode::Char('v' | 'V'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && key.modifiers.contains(KeyModifiers::ALT)
        && !key.modifiers.contains(KeyModifiers::SHIFT)
}

pub fn is_linux_configs_chord(key: &KeyEvent) -> bool {
    use crossterm::event::KeyEventKind;
    if !crate::app_linux_configs::linux_configs_supported() {
        return false;
    }
    matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && matches!(key.code, KeyCode::Char('l' | 'L'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && key.modifiers.contains(KeyModifiers::ALT)
        && !key.modifiers.contains(KeyModifiers::SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: mods,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn env_chord_is_ctrl_alt_v_not_ctrl_shift_v() {
        assert!(is_env_variables_chord(&key(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        )));
        assert!(!is_env_variables_chord(&key(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        )));
        assert!(!is_env_variables_chord(&key(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn linux_configs_chord_is_ctrl_alt_l() {
        assert!(is_linux_configs_chord(&key(
            KeyCode::Char('l'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        )));
        assert!(!is_linux_configs_chord(&key(
            KeyCode::Char('l'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        )));
        assert!(!is_linux_configs_chord(&key(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL | KeyModifiers::ALT
        )));
    }
}
