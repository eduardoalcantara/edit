#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    Normal,
    Block,
    Multi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct CursorManager {
    pub mode: CursorMode,
    pub primary: CursorPos,
    pub cursors: Vec<CursorPos>,
    pub block_start: Option<CursorPos>,
    pub block_end: Option<CursorPos>,
    pub dragging_block: bool,
}

impl Default for CursorManager {
    fn default() -> Self {
        Self {
            mode: CursorMode::Normal,
            primary: CursorPos { row: 0, col: 0 },
            cursors: vec![CursorPos { row: 0, col: 0 }],
            block_start: None,
            block_end: None,
            dragging_block: false,
        }
    }
}

impl CursorManager {
    pub fn sync_primary(&mut self, row: usize, col: usize) {
        self.primary = CursorPos { row, col };
        if self.mode == CursorMode::Normal {
            self.cursors = vec![self.primary];
        }
    }

    pub fn cancel_to_normal(&mut self) {
        self.mode = CursorMode::Normal;
        self.cursors = vec![self.primary];
        self.block_start = None;
        self.block_end = None;
        self.dragging_block = false;
    }

    pub fn start_block(&mut self, row: usize, col: usize) {
        self.mode = CursorMode::Block;
        self.block_start = Some(CursorPos { row, col });
        self.block_end = Some(CursorPos { row, col });
        self.dragging_block = true;
    }

    pub fn update_block(&mut self, row: usize, col: usize) {
        if self.mode == CursorMode::Block {
            self.block_end = Some(CursorPos { row, col });
        }
    }

    pub fn finish_block(&mut self) {
        if self.mode != CursorMode::Block {
            return;
        }
        self.dragging_block = false;
        if let (Some(start), Some(end)) = (self.block_start, self.block_end) {
            let (r0, r1) = if start.row <= end.row {
                (start.row, end.row)
            } else {
                (end.row, start.row)
            };
            let col = end.col.max(start.col);
            self.mode = CursorMode::Multi;
            self.cursors = (r0..=r1)
                .map(|row| CursorPos { row, col })
                .collect();
            if !self.cursors.is_empty() {
                self.primary = self.cursors[0];
            }
        }
    }

    pub fn add_cursor(&mut self, row: usize, col: usize) {
        let pos = CursorPos { row, col };
        if self.cursors.iter().any(|c| *c == pos) {
            return;
        }
        if self.mode == CursorMode::Normal {
            self.mode = CursorMode::Multi;
        }
        self.cursors.push(pos);
        self.cursors.sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));
        self.merge_colliding();
    }

    pub fn merge_colliding(&mut self) {
        self.cursors.sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));
        self.cursors.dedup();
        if self.cursors.len() <= 1 {
            if let Some(c) = self.cursors.first().copied() {
                self.primary = c;
            }
            if self.mode == CursorMode::Multi {
                self.mode = CursorMode::Normal;
            }
        }
    }

    pub fn block_range(&self) -> Option<(usize, usize, usize, usize)> {
        let start = self.block_start?;
        let end = self.block_end?;
        let r0 = start.row.min(end.row);
        let r1 = start.row.max(end.row);
        let c0 = start.col.min(end.col);
        let c1 = start.col.max(end.col);
        Some((r0, c0, r1, c1))
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            CursorMode::Normal => "Normal",
            CursorMode::Block => "Bloco",
            CursorMode::Multi => "Multi",
        }
    }
}
