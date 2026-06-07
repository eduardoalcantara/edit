use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Normal,
    Block,
    Multi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub char_idx: usize,
    pub virtual_col: usize,
    pub anchor: Option<usize>,
}

impl Cursor {
    pub fn new(char_idx: usize) -> Self {
        Self {
            char_idx,
            virtual_col: 0,
            anchor: None,
        }
    }
}

pub fn char_idx_to_line_col(text: &Rope, char_idx: usize) -> (usize, usize) {
    if text.len_chars() == 0 {
        return (0, 0);
    }
    if char_idx >= text.len_chars() {
        let last_line = text.len_lines().saturating_sub(1);
        let line_start = text.line_to_char(last_line);
        return (last_line, char_idx.saturating_sub(line_start));
    }
    let line = text.char_to_line(char_idx);
    let line_start = text.line_to_char(line);
    (line, char_idx.saturating_sub(line_start))
}

pub fn line_col_to_char_idx(text: &Rope, line: usize, col: usize) -> usize {
    if text.len_lines() == 0 {
        return 0;
    }
    let line = line.min(text.len_lines().saturating_sub(1));
    let line_start = text.line_to_char(line);
    let line_len = text.line(line).len_chars();
    line_start + col.min(line_len)
}

pub fn sync_virtual_col(text: &Rope, cursor: &mut Cursor) {
    let (_, col) = char_idx_to_line_col(text, cursor.char_idx);
    cursor.virtual_col = col;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn line_col_roundtrip_utf8() {
        let text = Rope::from_str("aç\nemoji🎉");
        let idx = line_col_to_char_idx(&text, 1, 5);
        let (line, col) = char_idx_to_line_col(&text, idx);
        assert_eq!(line, 1);
        assert_eq!(col, 5);
    }

    #[test]
    fn char_idx_after_newline() {
        let text = Rope::from_str("hello\n");
        let (line, col) = char_idx_to_line_col(&text, 6);
        assert_eq!((line, col), (1, 0));
    }
}
