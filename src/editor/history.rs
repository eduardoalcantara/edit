use ropey::Rope;

#[derive(Debug, Clone)]
struct HistoryEntry {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: usize,
    cursor_after: usize,
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
        self.redo_stack.clear();
        self.undo_stack.push(HistoryEntry {
            start,
            removed,
            inserted,
            cursor_before,
            cursor_after,
        });
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, text: &mut Rope, cursor: &mut usize) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };
        let end = entry.start + entry.inserted.chars().count();
        if entry.start <= text.len_chars() && end <= text.len_chars() {
            text.remove(entry.start..end);
        }
        text.insert(entry.start, &entry.removed);
        *cursor = entry.cursor_before;
        self.redo_stack.push(entry);
        true
    }

    pub fn redo(&mut self, text: &mut Rope, cursor: &mut usize) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        let end = entry.start + entry.removed.chars().count();
        if entry.start <= text.len_chars() && end <= text.len_chars() {
            text.remove(entry.start..end);
        }
        text.insert(entry.start, &entry.inserted);
        *cursor = entry.cursor_after;
        self.undo_stack.push(entry);
        true
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
