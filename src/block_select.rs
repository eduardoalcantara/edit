use crate::cursors::CursorManager;

pub fn extract_block_text(lines: &[String], r0: usize, c0: usize, r1: usize, c1: usize) -> String {
    let mut out = Vec::new();
    for row in r0..=r1.min(lines.len().saturating_sub(1)) {
        let line = &lines[row];
        let end = c1.min(line.len());
        let start = c0.min(end);
        let mut slice = line[start..end].to_string();
        if end < c1 {
            slice.push_str(&" ".repeat(c1 - end));
        }
        out.push(slice);
    }
    out.join("\n")
}

pub fn block_highlight_rows(manager: &CursorManager) -> Vec<(usize, usize, usize)> {
    let Some((r0, c0, r1, c1)) = manager.block_range() else {
        return vec![];
    };
    (r0..=r1).map(|row| (row, c0, c1)).collect()
}

pub struct BlockSelect;

impl BlockSelect {
    pub fn on_press(manager: &mut CursorManager, row: usize, col: usize, alt: bool) -> bool {
        if alt {
            manager.start_block(row, col);
            true
        } else {
            false
        }
    }

    pub fn on_drag(manager: &mut CursorManager, row: usize, col: usize) {
        if manager.dragging_block {
            manager.update_block(row, col);
        }
    }

    pub fn on_release(manager: &mut CursorManager) {
        if manager.dragging_block {
            manager.finish_block();
        }
    }
}
