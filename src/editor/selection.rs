use ropey::Rope;

use crate::editor::cursor::{char_idx_to_line_col, line_col_to_char_idx, Cursor, SelectionMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSelectionState {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub dragging: bool,
}

impl BlockSelectionState {
    pub fn normalized(&self) -> (usize, usize, usize, usize) {
        let r0 = self.start_line.min(self.end_line);
        let r1 = self.start_line.max(self.end_line);
        let c0 = self.start_col.min(self.end_col);
        let c1 = self.start_col.max(self.end_col);
        (r0, c0, r1, c1)
    }
}

pub fn extract_block_text(text: &Rope, r0: usize, c0: usize, r1: usize, c1: usize) -> String {
    let mut out = Vec::new();
    let lines = text.len_lines().max(1);
    for row in r0..=r1.min(lines.saturating_sub(1)) {
        let line = text.line(row);
        let end = c1.min(line.len_chars());
        let start = c0.min(end);
        let mut slice = line.slice(start..end).to_string();
        if end < c1 {
            slice.push_str(&" ".repeat(c1 - end));
        }
        out.push(slice);
    }
    out.join("\n")
}

pub fn extract_linear_text(text: &Rope, start: usize, end: usize) -> String {
    let (a, b) = if start <= end { (start, end) } else { (end, start) };
    if a == b {
        return String::new();
    }
    text.slice(a..b).to_string()
}

pub fn delete_block(text: &mut Rope, block: &BlockSelectionState) {
    let (r0, c0, r1, c1) = block.normalized();
    for row in (r0..=r1).rev() {
        let line_start = text.line_to_char(row);
        let line = text.line(row);
        let end = c1.min(line.len_chars());
        let start = c0.min(end);
        let del_start = line_start + start;
        let del_end = line_start + end;
        if del_start < del_end {
            text.remove(del_start..del_end);
        }
    }
}

pub fn selection_label(
    mode: SelectionMode,
    block: Option<&BlockSelectionState>,
    cursors: &[Cursor],
    primary: &Cursor,
    text: &Rope,
) -> String {
    match mode {
        SelectionMode::Block => {
            if let Some(b) = block {
                let (r0, c0, r1, c1) = b.normalized();
                return format!("bloco ({r0},{c0})-({r1},{c1})");
            }
        }
        SelectionMode::Multi => return format!("multi:{}", cursors.len()),
        SelectionMode::Normal => {
            if let Some(anchor) = primary.anchor {
                if anchor != primary.char_idx {
                    let (r1, c1) = char_idx_to_line_col(text, primary.char_idx);
                    let (r0, c0) = char_idx_to_line_col(text, anchor);
                    return format!("({r0},{c0})-({r1},{c1})");
                }
            }
        }
    }
    "0".to_string()
}

pub fn merge_cursors(cursors: &mut Vec<Cursor>) {
    cursors.sort_by_key(|c| c.char_idx);
    cursors.dedup_by_key(|c| c.char_idx);
}

pub fn add_cursor_at(text: &Rope, cursors: &mut Vec<Cursor>, line: usize, col: usize) {
    let idx = line_col_to_char_idx(text, line, col);
    if cursors.iter().any(|c| c.char_idx == idx) {
        return;
    }
    cursors.push(Cursor {
        char_idx: idx,
        virtual_col: col,
        anchor: None,
    });
    merge_cursors(cursors);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn extract_block_pads_short_lines() {
        let text = Rope::from_str("ab\nx");
        let block = extract_block_text(&text, 0, 1, 1, 2);
        assert_eq!(block, "b\n ");
    }

    #[test]
    fn merge_removes_duplicate_cursors() {
        let mut cursors = vec![
            Cursor::new(5),
            Cursor::new(5),
            Cursor::new(10),
        ];
        merge_cursors(&mut cursors);
        assert_eq!(cursors.len(), 2);
    }
}
