//! Split horizontal do editor — ativação, foco e atalhos.

use crate::app::App;
use crate::editor_split::{SplitMode, SplitPane};
use crate::view_state::InputFocus;

impl App {
    pub fn split_active(&self) -> bool {
        self.editor_split.is_active()
    }

    pub fn toggle_editor_split(&mut self) {
        if self.editor_split.is_active() {
            self.disable_editor_split();
        } else {
            self.enable_editor_split();
        }
    }

    pub fn enable_editor_split(&mut self) {
        if !self.editor_split.can_activate(self.workspace.tabs.len()) {
            self.set_status("Abra outra aba para dividir o editor");
            return;
        }
        if self.editor_split.is_active() {
            return;
        }
        self.sync_active_tab();
        let active = self.workspace.active_index;
        self.editor_split.mode = SplitMode::Horizontal;
        self.editor_split.left_tab = active;
        self.editor_split.right_tab = self.pick_secondary_tab(active);
        self.editor_split.focused_pane = SplitPane::Left;
        self.editor_split.ensure_distinct_tabs();
        self.focus_editor_pane(SplitPane::Left);
        self.set_status("Editor dividido (Ctrl+1 único | Ctrl+2 direita)");
        self.persist_user_config();
    }

    pub fn disable_editor_split(&mut self) {
        if !self.editor_split.is_active() {
            return;
        }
        let keep = self.editor_split.focused_tab();
        self.editor_split.mode = SplitMode::Off;
        self.editor_split.right_tab = None;
        self.focus_tab(keep);
        self.set_status("Editor único");
        self.persist_user_config();
    }

    pub fn focus_editor_pane(&mut self, pane: SplitPane) {
        if !self.editor_split.is_active() {
            self.focus_editor();
            return;
        }
        self.sync_active_tab();
        let index = match pane {
            SplitPane::Left => self.editor_split.left_tab,
            SplitPane::Right => self
                .editor_split
                .right_tab
                .unwrap_or(self.editor_split.left_tab),
        };
        self.editor_split.focused_pane = pane;
        self.focus_tab(index);
        self.input_focus = InputFocus::Editor;
        self.persist_user_config();
    }

    /// `Ctrl+1`: editor único ou foco no painel esquerdo.
    pub fn chord_editor_single_or_left(&mut self) {
        if self.editor_split.is_active() {
            if self.editor_split.focused_pane == SplitPane::Right {
                self.focus_editor_pane(SplitPane::Left);
            } else {
                self.disable_editor_split();
            }
        } else {
            self.focus_editor();
        }
    }

    /// `Ctrl+2`: divide o editor ou foca o painel direito.
    pub fn chord_editor_split_or_right(&mut self) {
        if self.editor_split.is_active() {
            self.focus_editor_pane(SplitPane::Right);
        } else {
            self.enable_editor_split();
            if self.editor_split.is_active() {
                self.focus_editor_pane(SplitPane::Right);
            }
        }
    }

    pub(crate) fn on_tab_count_changed(&mut self) {
        if self.workspace.tabs.len() < 2 && self.editor_split.is_active() {
            self.disable_editor_split();
            return;
        }
        if self.editor_split.is_active() {
            let len = self.workspace.tabs.len();
            if self.editor_split.left_tab >= len {
                self.editor_split.left_tab = len.saturating_sub(1);
            }
            if let Some(r) = self.editor_split.right_tab {
                if r >= len {
                    self.editor_split.right_tab =
                        self.pick_secondary_tab(self.editor_split.left_tab);
                }
            }
            self.editor_split.ensure_distinct_tabs();
        }
    }

    pub(crate) fn on_tab_closed(&mut self, closed_index: usize) {
        if !self.editor_split.is_active() {
            return;
        }
        let adjust = |idx: usize| -> Option<usize> {
            if idx == closed_index {
                None
            } else if idx > closed_index {
                Some(idx - 1)
            } else {
                Some(idx)
            }
        };
        if let Some(l) = adjust(self.editor_split.left_tab) {
            self.editor_split.left_tab = l;
        } else {
            self.editor_split.left_tab = self
                .workspace
                .active_index
                .min(self.workspace.tabs.len().saturating_sub(1));
        }
        if let Some(r) = self.editor_split.right_tab {
            self.editor_split.right_tab = match adjust(r) {
                Some(nr) => Some(nr),
                None => self.pick_secondary_tab(self.editor_split.left_tab),
            };
        }
        self.editor_split.ensure_distinct_tabs();
        self.on_tab_count_changed();
    }

    pub(crate) fn sync_split_after_focus_tab(&mut self, index: usize) {
        if !self.editor_split.is_active() {
            return;
        }
        self.editor_split.set_pane_tab(self.editor_split.focused_pane, index);
    }

    pub(crate) fn remap_editor_split_indices(&mut self) {
        if !self.editor_split.is_active() {
            return;
        }
        let left_id = self
            .workspace
            .tabs
            .get(self.editor_split.left_tab)
            .map(|t| t.session_id.clone());
        let right_id = self.editor_split.right_tab.and_then(|i| {
            self.workspace
                .tabs
                .get(i)
                .map(|t| t.session_id.clone())
        });
        if let Some(id) = left_id {
            if let Some(i) = self
                .workspace
                .tabs
                .iter()
                .position(|t| t.session_id == id)
            {
                self.editor_split.left_tab = i;
            }
        }
        if let Some(id) = right_id {
            if let Some(i) = self
                .workspace
                .tabs
                .iter()
                .position(|t| t.session_id == id)
            {
                self.editor_split.right_tab = Some(i);
            }
        }
        self.editor_split.ensure_distinct_tabs();
    }

    fn pick_secondary_tab(&self, primary: usize) -> Option<usize> {
        let len = self.workspace.tabs.len();
        if len < 2 {
            return None;
        }
        let next = (primary + 1) % len;
        if next != primary {
            Some(next)
        } else {
            Some(0)
        }
    }
}
