use ropey::Rope;
use serde::{Deserialize, Serialize};

use crate::editor::selection::BlockDeletePatch;

pub const PERSIST_UNDO_MIN: usize = 5;
pub const PERSIST_UNDO_MAX: usize = 20;

#[derive(Debug, Clone)]
struct LinearPatch {
    start: usize,
    removed: String,
    inserted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableLinearPatch {
    start: usize,
    removed: String,
    inserted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableBlockPatch {
    row: usize,
    char_col: usize,
    removed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableHistoryEntry {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: usize,
    cursor_after: usize,
    block_delete: Option<Vec<SerializableBlockPatch>>,
    linear_patches: Option<Vec<SerializableLinearPatch>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryStacks {
    pub undo: Vec<SerializableHistoryEntry>,
    pub redo: Vec<SerializableHistoryEntry>,
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

    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn export_stacks(&self) -> HistoryStacks {
        HistoryStacks {
            undo: self
                .undo_stack
                .iter()
                .map(entry_to_serializable)
                .collect(),
            redo: self
                .redo_stack
                .iter()
                .map(entry_to_serializable)
                .collect(),
        }
    }

    pub fn export_for_persist(&self) -> HistoryStacks {
        let mut stacks = self.export_stacks();
        if stacks.undo.len() > PERSIST_UNDO_MAX {
            let drop = stacks.undo.len() - PERSIST_UNDO_MAX;
            stacks.undo.drain(0..drop);
        }
        if stacks.redo.len() > PERSIST_UNDO_MAX {
            let drop = stacks.redo.len() - PERSIST_UNDO_MAX;
            stacks.redo.drain(0..drop);
        }
        stacks
    }

    pub fn import_stacks(&mut self, stacks: HistoryStacks) {
        self.undo_stack = stacks.undo.into_iter().map(entry_from_serializable).collect();
        self.redo_stack = stacks.redo.into_iter().map(entry_from_serializable).collect();
    }

    pub fn take_stacks(&mut self) -> HistoryStacks {
        let stacks = self.export_stacks();
        self.clear();
        stacks
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

fn entry_to_serializable(entry: &HistoryEntry) -> SerializableHistoryEntry {
    SerializableHistoryEntry {
        start: entry.start,
        removed: entry.removed.clone(),
        inserted: entry.inserted.clone(),
        cursor_before: entry.cursor_before,
        cursor_after: entry.cursor_after,
        block_delete: entry.block_delete.as_ref().map(|patches| {
            patches
                .iter()
                .map(|p| SerializableBlockPatch {
                    row: p.row,
                    char_col: p.char_col,
                    removed: p.removed.clone(),
                })
                .collect()
        }),
        linear_patches: entry.linear_patches.as_ref().map(|patches| {
            patches
                .iter()
                .map(|p| SerializableLinearPatch {
                    start: p.start,
                    removed: p.removed.clone(),
                    inserted: p.inserted.clone(),
                })
                .collect()
        }),
    }
}

fn entry_from_serializable(entry: SerializableHistoryEntry) -> HistoryEntry {
    HistoryEntry {
        start: entry.start,
        removed: entry.removed,
        inserted: entry.inserted,
        cursor_before: entry.cursor_before,
        cursor_after: entry.cursor_after,
        block_delete: entry.block_delete.map(|patches| {
            patches
                .into_iter()
                .map(|p| BlockDeletePatch {
                    row: p.row,
                    char_col: p.char_col,
                    removed: p.removed,
                })
                .collect()
        }),
        linear_patches: entry.linear_patches.map(|patches| {
            patches
                .into_iter()
                .map(|p| LinearPatch {
                    start: p.start,
                    removed: p.removed,
                    inserted: p.inserted,
                })
                .collect()
        }),
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

    #[test]
    fn export_import_round_trip() {
        let mut history = EditHistory::new();
        history.record_change(0, String::new(), "x".into(), 0, 1);
        let stacks = history.export_stacks();
        let mut other = EditHistory::new();
        other.import_stacks(stacks);
        assert_eq!(other.undo_depth(), 1);
    }
}
