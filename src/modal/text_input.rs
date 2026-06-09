//! Campo de texto de linha única com edição padrão (cursor, seleção, clipboard).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::clipboard::Clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharAccept {
    #[default]
    Any,
    AsciiDigit,
}

#[derive(Debug, Clone)]
pub struct TextInput {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
}

impl TextInput {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.chars().count();
        Self {
            text,
            cursor,
            anchor: None,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.anchor = None;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, value: &str) {
        self.text = value.to_string();
        self.cursor = self.text.chars().count();
        self.anchor = None;
    }

    pub fn display_focused(&self) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        let pos = self.cursor.min(chars.len());
        let before: String = chars[..pos].iter().collect();
        let after: String = chars[pos..].iter().collect();
        format!(" {before}▌{after}")
    }

    pub fn display_unfocused(&self) -> String {
        format!(" {}", self.text)
    }

    /// `true` se a tecla foi consumida pelo campo.
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        clipboard: &mut Clipboard,
        accept: CharAccept,
    ) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl {
            match key.code {
                KeyCode::Char('c' | 'C') => {
                    if let Some(text) = self.selected_text() {
                        clipboard.push(text);
                    }
                    return true;
                }
                KeyCode::Char('x' | 'X') => {
                    if let Some(text) = self.take_selected_text() {
                        clipboard.push(text);
                    }
                    return true;
                }
                KeyCode::Char('v' | 'V') => {
                    if let Some(paste) = clipboard.paste_text() {
                        self.insert_filtered(&paste, accept);
                    }
                    return true;
                }
                KeyCode::Char('a' | 'A') => {
                    self.anchor = Some(0);
                    self.cursor = self.text.chars().count();
                    return true;
                }
                _ => {}
            }
        }

        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Left => {
                self.move_by(-1, shift);
                true
            }
            KeyCode::Right => {
                self.move_by(1, shift);
                true
            }
            KeyCode::Home => {
                self.move_to(0, shift);
                true
            }
            KeyCode::End => {
                self.move_to(self.text.chars().count(), shift);
                true
            }
            KeyCode::Backspace => {
                if !self.delete_selection() {
                    self.delete_before();
                }
                true
            }
            KeyCode::Delete => {
                if !self.delete_selection() {
                    self.delete_after();
                }
                true
            }
            KeyCode::Char(ch) if !ctrl && accepts_char(ch, accept) => {
                self.delete_selection();
                self.insert_char(ch);
                true
            }
            _ => false,
        }
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        if start == end {
            return None;
        }
        Some(self.text.chars().skip(start).take(end - start).collect())
    }

    fn take_selected_text(&mut self) -> Option<String> {
        let text = self.selected_text()?;
        self.delete_selection();
        Some(text)
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.anchor?;
        Some(if anchor <= self.cursor {
            (anchor, self.cursor)
        } else {
            (self.cursor, anchor)
        })
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        if start == end {
            self.anchor = None;
            return false;
        }
        let before: String = self.text.chars().take(start).collect();
        let after: String = self.text.chars().skip(end).collect();
        self.text = format!("{before}{after}");
        self.cursor = start;
        self.anchor = None;
        true
    }

    fn delete_before(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before: String = self.text.chars().take(self.cursor - 1).collect();
        let after: String = self.text.chars().skip(self.cursor).collect();
        self.text = format!("{before}{after}");
        self.cursor -= 1;
    }

    fn delete_after(&mut self) {
        let len = self.text.chars().count();
        if self.cursor >= len {
            return;
        }
        let before: String = self.text.chars().take(self.cursor).collect();
        let after: String = self.text.chars().skip(self.cursor + 1).collect();
        self.text = format!("{before}{after}");
    }

    fn insert_char(&mut self, ch: char) {
        let before: String = self.text.chars().take(self.cursor).collect();
        let after: String = self.text.chars().skip(self.cursor).collect();
        self.text = format!("{before}{ch}{after}");
        self.cursor += 1;
    }

    fn insert_filtered(&mut self, paste: &str, accept: CharAccept) {
        self.delete_selection();
        for ch in paste.chars().filter(|&ch| accepts_char(ch, accept)) {
            self.insert_char(ch);
        }
    }

    fn move_by(&mut self, delta: i32, extend_selection: bool) {
        let len = self.text.chars().count() as i32;
        let next = (self.cursor as i32 + delta).clamp(0, len) as usize;
        self.move_to(next, extend_selection);
    }

    fn move_to(&mut self, pos: usize, extend_selection: bool) {
        if extend_selection {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = pos;
    }
}

fn accepts_char(ch: char, accept: CharAccept) -> bool {
    match accept {
        CharAccept::Any => !ch.is_control(),
        CharAccept::AsciiDigit => ch.is_ascii_digit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn key_ctrl(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }

    #[test]
    fn inserts_and_deletes_with_cursor() {
        let mut input = TextInput::new("");
        let mut clip = Clipboard::default();
        assert!(input.handle_key(key(KeyCode::Char('a')), &mut clip, CharAccept::Any));
        assert!(input.handle_key(key(KeyCode::Char('b')), &mut clip, CharAccept::Any));
        assert_eq!(input.text(), "ab");
        input.handle_key(key(KeyCode::Left), &mut clip, CharAccept::Any);
        input.handle_key(key(KeyCode::Delete), &mut clip, CharAccept::Any);
        assert_eq!(input.text(), "a");
        input.handle_key(key(KeyCode::Backspace), &mut clip, CharAccept::Any);
        assert_eq!(input.text(), "");
    }

    #[test]
    fn home_end_and_select_all_copy() {
        let mut input = TextInput::new("hello");
        let mut clip = Clipboard::default();
        input.handle_key(key(KeyCode::Home), &mut clip, CharAccept::Any);
        input.handle_key(key_ctrl('a'), &mut clip, CharAccept::Any);
        input.handle_key(key_ctrl('c'), &mut clip, CharAccept::Any);
        assert_eq!(clip.latest(), Some("hello"));
        input.handle_key(key(KeyCode::End), &mut clip, CharAccept::Any);
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn digit_filter_blocks_letters() {
        let mut input = TextInput::new("");
        let mut clip = Clipboard::default();
        assert!(!input.handle_key(key(KeyCode::Char('x')), &mut clip, CharAccept::AsciiDigit));
        assert!(input.handle_key(key(KeyCode::Char('9')), &mut clip, CharAccept::AsciiDigit));
        assert_eq!(input.text(), "9");
    }
}
