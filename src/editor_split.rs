//! Split horizontal do editor (duas abas visíveis).

use ratatui::layout::Rect;

use crate::reference_pane::ReferencePane;

pub const MIN_PANE_WIDTH: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    #[default]
    Off,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitPane {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorSplitLayout {
    pub left: Rect,
    pub right: Rect,
}

#[derive(Debug)]
pub struct EditorSplit {
    pub mode: SplitMode,
    pub left_tab: usize,
    pub right_tab: Option<usize>,
    pub focused_pane: SplitPane,
    pub reference: Option<ReferencePane>,
}

impl Default for EditorSplit {
    fn default() -> Self {
        Self {
            mode: SplitMode::Off,
            left_tab: 0,
            right_tab: None,
            focused_pane: SplitPane::Left,
            reference: None,
        }
    }
}

impl EditorSplit {
    pub fn is_active(&self) -> bool {
        self.mode == SplitMode::Horizontal
    }

    pub fn tab_for_pane(&self, pane: SplitPane) -> Option<usize> {
        match pane {
            SplitPane::Left => Some(self.left_tab),
            SplitPane::Right => {
                if self.reference.is_some() {
                    None
                } else {
                    self.right_tab
                }
            }
        }
    }

    pub fn has_reference(&self) -> bool {
        self.reference.is_some()
    }

    pub fn pane_tab_index(&self, pane: SplitPane) -> Option<usize> {
        match pane {
            SplitPane::Left => Some(self.left_tab),
            SplitPane::Right => self.right_tab,
        }
    }

    pub fn focused_tab(&self) -> usize {
        match self.focused_pane {
            SplitPane::Left => self.left_tab,
            SplitPane::Right => self
                .right_tab
                .or(self.reference.as_ref().and_then(|r| r.stashed_right_tab))
                .unwrap_or(self.left_tab),
        }
    }

    pub fn set_pane_tab(&mut self, pane: SplitPane, index: usize) {
        match pane {
            SplitPane::Left => self.left_tab = index,
            SplitPane::Right => self.right_tab = Some(index),
        }
        self.ensure_distinct_tabs();
    }

    pub fn ensure_distinct_tabs(&mut self) {
        if self.mode != SplitMode::Horizontal {
            return;
        }
        if self.right_tab == Some(self.left_tab) {
            self.right_tab = None;
        }
    }

    pub fn can_activate(&self, tab_count: usize) -> bool {
        tab_count >= 1
    }

    /// Com split desligado não existe painel direito — o foco interno deve ser sempre esquerdo.
    pub fn enforce_focus_invariant(&mut self) {
        if !self.is_active() {
            self.focused_pane = SplitPane::Left;
        }
    }
}

/// Divide a faixa do editor (acima do terminal) em dois painéis horizontais (~50/50).
pub fn split_editor_horizontally(shell: Rect, bottom_reserve: u16) -> EditorSplitLayout {
    let strip = Rect {
        x: shell.x,
        y: shell.y,
        width: shell.width,
        height: shell.height.saturating_sub(bottom_reserve),
    };
    split_shell_horizontally(strip)
}

/// Divide um retângulo em dois painéis horizontais (~50/50).
pub fn split_shell_horizontally(shell: Rect) -> EditorSplitLayout {
    if shell.width < MIN_PANE_WIDTH.saturating_mul(2) {
        return EditorSplitLayout {
            left: shell,
            right: Rect {
                x: shell.x.saturating_add(shell.width),
                y: shell.y,
                width: 0,
                height: shell.height,
            },
        };
    }
    let left_w = shell.width / 2;
    let right_w = shell.width.saturating_sub(left_w);
    EditorSplitLayout {
        left: Rect {
            x: shell.x,
            y: shell.y,
            width: left_w,
            height: shell.height,
        },
        right: Rect {
            x: shell.x.saturating_add(left_w),
            y: shell.y,
            width: right_w,
            height: shell.height,
        },
    }
}

pub fn pane_at_column(split: EditorSplitLayout, column: u16) -> Option<SplitPane> {
    if column >= split.left.x && column < split.left.x.saturating_add(split.left.width) {
        Some(SplitPane::Left)
    } else if split.right.width > 0
        && column >= split.right.x
        && column < split.right.x.saturating_add(split.right.width)
    {
        Some(SplitPane::Right)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_shell_divides_fifty_fifty() {
        let shell = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 20,
        };
        let layout = split_shell_horizontally(shell);
        assert_eq!(layout.left.width, 40);
        assert_eq!(layout.right.width, 40);
        assert_eq!(layout.left.x, 0);
        assert_eq!(layout.right.x, 40);
        assert_eq!(layout.left.height, 20);
        assert_eq!(layout.left.x + layout.left.width, layout.right.x);
    }

    #[test]
    fn split_editor_respects_terminal_reserve() {
        let shell = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        let layout = split_editor_horizontally(shell, 8);
        assert_eq!(layout.left.height, 16);
        assert_eq!(layout.right.height, 16);
    }

    #[test]
    fn pane_hit_test() {
        let layout = split_shell_horizontally(Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 10,
        });
        assert_eq!(pane_at_column(layout, 10), Some(SplitPane::Left));
        assert_eq!(pane_at_column(layout, 50), Some(SplitPane::Right));
    }

    #[test]
    fn distinct_tabs_clears_duplicate_right() {
        let mut split = EditorSplit::default();
        split.mode = SplitMode::Horizontal;
        split.left_tab = 1;
        split.right_tab = Some(1);
        split.ensure_distinct_tabs();
        assert_eq!(split.right_tab, None);
    }

    #[test]
    fn focus_invariant_resets_right_when_split_off() {
        let mut split = EditorSplit {
            focused_pane: SplitPane::Right,
            ..EditorSplit::default()
        };
        split.enforce_focus_invariant();
        assert_eq!(split.focused_pane, SplitPane::Left);
    }
}
