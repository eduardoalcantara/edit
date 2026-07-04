//! Painel de referência no split — abrir, fechar e foco.

use ratatui::layout::Rect;

use crate::app::App;
use crate::editor_split::{SplitMode, SplitPane};
use crate::reference_pane::{new_reference_editor, ReferenceKind, ReferencePane};
use crate::view_state::InputFocus;

impl App {
    pub fn has_reference_pane(&self) -> bool {
        self.editor_split.reference.is_some()
    }

    pub fn reference_pane_active(&self) -> bool {
        self.has_reference_pane() && self.editor_split.focused_pane == SplitPane::Right
    }

    pub fn reference_close_label(&self) -> String {
        if self.view.use_paren_mnemonics {
            "(F)echar".to_string()
        } else {
            "Fechar".to_string()
        }
    }

    pub fn open_reference_pane(&mut self, kind: ReferenceKind) {
        self.sync_active_tab();

        let stash = if let Some(ref reference) = self.editor_split.reference {
            reference.stashed_right_tab
        } else {
            self.editor_split.right_tab
        };

        if !self.editor_split.is_active() {
            self.editor_split.mode = SplitMode::Horizontal;
            self.editor_split.left_tab = self.workspace.active_index;
        }

        if self.reference_pane_active() {
            self.unfocus_reference_pane();
        }

        let palette = self.theme.palette();
        let editor = new_reference_editor(kind, &palette);
        self.editor_split.reference = Some(ReferencePane {
            kind,
            editor,
            stashed_right_tab: stash,
        });
        self.editor_split.right_tab = None;
        self.focus_reference_pane();
        self.set_status(format!("Referência: {} (Esc ou Fechar para voltar)", kind.title()));
    }

    pub fn close_reference_pane(&mut self) {
        let was_reference_focus = self.reference_pane_active();
        if was_reference_focus {
            self.unfocus_reference_pane();
        }
        let Some(reference) = self.editor_split.reference.take() else {
            return;
        };

        self.editor_split.right_tab = reference.stashed_right_tab;
        self.reference_close_hit = None;

        self.editor_split.focused_pane = SplitPane::Left;
        let keep = self
            .editor_split
            .left_tab
            .min(self.workspace.tabs.len().saturating_sub(1));

        if self.editor_split.right_tab.is_none() && self.workspace.tabs.len() < 2 {
            self.editor_split.mode = SplitMode::Off;
        }
        self.editor_split.enforce_focus_invariant();

        self.focus_tab_unchecked(keep);

        if self.editor_split.right_tab.is_some() {
            self.focus_editor_pane(SplitPane::Right);
        }

        self.set_status("Referência fechada");
    }

    pub fn open_help_features(&mut self) {
        self.open_reference_pane(ReferenceKind::HelpFeatures);
    }

    pub fn open_help_shortcuts(&mut self) {
        self.open_reference_pane(ReferenceKind::HelpShortcuts);
    }

    pub fn open_ascii_table(&mut self) {
        self.open_reference_pane(ReferenceKind::AsciiTable);
    }

    pub(crate) fn focus_reference_pane(&mut self) {
        if self.editor_split.reference.is_none() {
            return;
        }
        self.editor_split.focused_pane = SplitPane::Right;
        std::mem::swap(
            &mut self.editor,
            &mut self.editor_split.reference.as_mut().unwrap().editor,
        );
        self.input_focus = InputFocus::Editor;
    }

    pub(crate) fn unfocus_reference_pane(&mut self) {
        if self.editor_split.reference.is_none() {
            return;
        }
        std::mem::swap(
            &mut self.editor,
            &mut self.editor_split.reference.as_mut().unwrap().editor,
        );
    }

    /// Devolve foco ao editor de arquivo antes de fechar/salvar/sync de abas.
    pub(crate) fn prepare_file_tab_operations(&mut self) {
        if self.workspace.tabs.is_empty() {
            return;
        }

        if self.reference_pane_active() {
            self.unfocus_reference_pane();
        }

        // Só corrige foco "direita" quando o split está desligado (estado inválido pós-ajuda).
        let stale_right_focus = !self.split_active()
            && self.editor_split.focused_pane == SplitPane::Right;

        if stale_right_focus {
            let left = self
                .editor_split
                .left_tab
                .min(self.workspace.tabs.len() - 1);
            self.editor_split.focused_pane = SplitPane::Left;
            self.editor_split.left_tab = left;
            self.focus_tab_unchecked(left);
            self.input_focus = InputFocus::Editor;
        }
    }

    /// Índice da aba ligada ao painel do editor que tem o teclado (split ou único).
    pub(crate) fn focused_editor_tab_index(&self) -> Option<usize> {
        if self.reference_pane_active() {
            return None;
        }
        if self.split_active() {
            match self.editor_split.focused_pane {
                SplitPane::Left => Some(self.editor_split.left_tab),
                SplitPane::Right => self.editor_split.right_tab,
            }
        } else {
            Some(self.workspace.active_index)
        }
    }

    /// Descarta alterações não salvas da aba em foco (reverte ao último conteúdo salvo).
    pub(crate) fn discard_focused_tab_changes(&mut self) {
        self.prepare_file_tab_operations();
        self.sync_active_tab();
        let idx = self
            .focused_editor_tab_index()
            .unwrap_or(self.workspace.active_index);
        if idx >= self.workspace.tabs.len() {
            return;
        }
        let saved = self.workspace.tabs[idx]
            .document
            .saved_snapshot()
            .to_string();
        self.workspace.tabs[idx].editor.replace_content(&saved);
        if self.workspace.active_index == idx {
            self.editor.replace_content(&saved);
        }
    }

    pub(crate) fn effective_right_tab(&self) -> Option<usize> {
        self.editor_split
            .reference
            .as_ref()
            .and_then(|r| r.stashed_right_tab)
            .or(self.editor_split.right_tab)
    }

    pub(crate) fn set_reference_close_hit(&mut self, rect: Option<Rect>) {
        self.reference_close_hit = rect;
    }

    pub(crate) fn reference_title(&self) -> Option<String> {
        self.editor_split
            .reference
            .as_ref()
            .map(|r| r.kind.title().to_string())
    }
}
