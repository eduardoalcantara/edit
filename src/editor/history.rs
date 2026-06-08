use ropey::Rope;

use crate::editor::selection::BlockDeletePatch;

#[derive(Debug, Clone)]
struct LinearPatch {
    start: usize,
    removed: String,
    inserted: String,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: usize,
    cursor_after: usize,
    block_delete: Option<Vec<BlockDeletePatch>>,
    linear_patches: Option<Vec<LinearPatch>>,
}

#[derive(Debug, Default)]
pub struct EditHistory {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    max_depth: usize,
}

impl EditHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100,
        }
    }

    pub fn record_change(
        &mut self,
        start: usize,
        removed: String,
        inserted: String,
        cursor_before: usize,
        cursor_after: usize,
    ) {
        if removed.is_empty() && inserted.is_empty() {
            return;
        }
        self.push_entry(HistoryEntry {
            start,
            removed,
            inserted,
            cursor_before,
            cursor_after,
            block_delete: None,
            linear_patches: None,
        });
    }

    pub fn record_multi_linear(
        &mut self,
        patches: Vec<(usize, String, String)>,
        cursor_before: usize,
        cursor_after: usize,
    ) {
        if patches.is_empty() {
            return;
        }
        self.push_entry(HistoryEntry {
            start: 0,
            removed: String::new(),
            inserted: String::new(),
            cursor_before,
            cursor_after,
            block_delete: None,
            linear_patches: Some(
                patches
                    .into_iter()
                    .map(|(start, removed, inserted)| LinearPatch {
                        start,
                        removed,
                        inserted,
                    })
                    .collect(),
            ),
        });
    }

    pub fn record_block_delete(
        &mut self,
        patches: Vec<BlockDeletePatch>,
        cursor_before: usize,
        cursor_after: usize,
    ) {
        if patches.is_empty() {
            return;
        }
        self.push_entry(HistoryEntry {
            start: 0,
            removed: String::new(),
            inserted: String::new(),
            cursor_before,
            cursor_after,
            block_delete: Some(patches),
            linear_patches: None,
        });
    }

    fn push_entry(&mut self, entry: HistoryEntry) {
        self.redo_stack.clear();
        self.undo_stack.push(entry);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, text: &mut Rope, cursor: &mut usize) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };
        if let Some(patches) = &entry.block_delete {
            for patch in patches {
                let line_start = text.line_to_char(patch.row);
                text.insert(line_start + patch.char_col, &patch.removed);
            }
        } else if let Some(patches) = &entry.linear_patches {
            for patch in patches.iter().rev() {
                apply_undo_patch(text, patch);
            }
        } else {
            apply_undo_patch(
                text,
                &LinearPatch {
                    start: entry.start,
                    removed: entry.removed.clone(),
                    inserted: entry.inserted.clone(),
                },
            );
        }
        *cursor = entry.cursor_before;
        self.redo_stack.push(entry);
        true
    }

    pub fn redo(&mut self, text: &mut Rope, cursor: &mut usize) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        if let Some(patches) = &entry.block_delete {
            for patch in patches.iter().rev() {
                let line_start = text.line_to_char(patch.row);
                let len = patch.removed.chars().count();
                text.remove(line_start + patch.char_col..line_start + patch.char_col + len);
            }
        } else if let Some(patches) = &entry.linear_patches {
            for patch in patches {
                apply_redo_patch(text, patch);
            }
        } else {
            apply_redo_patch(
                text,
                &LinearPatch {
                    start: entry.start,
                    removed: entry.removed.clone(),
                    inserted: entry.inserted.clone(),
                },
            );
        }
        *cursor = entry.cursor_after;
        self.undo_stack.push(entry);
        true
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

fn apply_undo_patch(text: &mut Rope, patch: &LinearPatch) {
    let end = patch.start + patch.inserted.chars().count();
    if patch.start <= text.len_chars() && end <= text.len_chars() {
        text.remove(patch.start..end);
    }
    text.insert(patch.start, &patch.removed);
}

fn apply_redo_patch(text: &mut Rope, patch: &LinearPatch) {
    let end = patch.start + patch.removed.chars().count();
    if patch.start <= text.len_chars() && end <= text.len_chars() {
        text.remove(patch.start..end);
    }
    text.insert(patch.start, &patch.inserted);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::selection::BlockDeletePatch;

    #[test]
    fn undo_linear_delete_restores_text() {
        let mut text = Rope::from_str(" world");
        let mut history = EditHistory::new();
        let mut cursor = 0;
        history.record_change(0, "hello".into(), String::new(), 5, 0);
        assert!(history.undo(&mut text, &mut cursor));
        assert_eq!(text.to_string(), "hello world");
        assert_eq!(cursor, 5);
    }

    #[test]
    fn undo_block_delete_restores_each_line() {
        let mut text = Rope::from_str("ad\nwz");
        let mut history = EditHistory::new();
        let mut cursor = 1;
        history.record_block_delete(
            vec![
                BlockDeletePatch {
                    row: 0,
                    char_col: 1,
                    removed: "bc".into(),
                },
                BlockDeletePatch {
                    row: 1,
                    char_col: 1,
                    removed: "xy".into(),
                },
            ],
            1,
            1,
        );
        assert!(history.undo(&mut text, &mut cursor));
        assert_eq!(text.to_string(), "abcd\nwxyz");
        assert_eq!(cursor, 1);
    }

    #[test]
    fn undo_multi_linear_restores_each_patch() {
        let mut text = Rope::from_str("ace");
        let mut history = EditHistory::new();
        let mut cursor = 0;
        history.record_multi_linear(
            vec![
                (3, "d".into(), String::new()),
                (1, "b".into(), String::new()),
            ],
            4,
            2,
        );
        assert!(history.undo(&mut text, &mut cursor));
        assert_eq!(text.to_string(), "abcde");
        assert_eq!(cursor, 4);
    }
}
