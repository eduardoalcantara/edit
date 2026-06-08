//! Converte eventos Crossterm em bytes para o PTY.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn key_event_to_pty_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return ctrl_key_bytes(key.code);
    }
    match key.code {
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Esc => Some(b"\x1b".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            Some(s.as_bytes().to_vec())
        }
        _ => None,
    }
}

fn ctrl_key_bytes(code: KeyCode) -> Option<Vec<u8>> {
    match code {
        KeyCode::Char('c' | 'C') => Some(vec![0x03]),
        KeyCode::Char('d' | 'D') => Some(vec![0x04]),
        KeyCode::Char('z' | 'Z') => Some(vec![0x1a]),
        KeyCode::Char('l' | 'L') => Some(b"\x0c".to_vec()),
        KeyCode::Char(c) if c.is_ascii() => {
            let upper = c.to_ascii_uppercase();
            let byte = (upper as u8).saturating_sub(b'A').saturating_add(1);
            Some(vec![byte])
        }
        _ => None,
    }
}
