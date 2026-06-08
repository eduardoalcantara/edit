use std::path::{Path, PathBuf};

use crate::document::Document;
use crate::editor::Editor;
use crate::encoding::{FileEncoding, Tabulation};
use crate::theme::ThemePalette;

use super::tab::{create_tab_from_defaults, next_novo_counter, novo_display_name, Tab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabSortStrategy {
    FileName,
    FilePath,
    OpenedFirst,
    OpenedLast,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptReason {
    CloseTab,
    EvictTail,
    Quit,
    CloseAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceAction {
    Ok,
    PromptSaveRequired { tab_index: usize, reason: PromptReason },
    FocusedExisting,
}

pub struct Workspace {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    pub max_tabs: usize,
    pub fechar_tudo_ao_sair: bool,
    pub salvar_desfazer_recentes: bool,
}

impl Workspace {
    pub fn with_single_tab(tab: Tab) -> Self {
        Self {
            tabs: vec![tab],
            active_index: 0,
            max_tabs: 10,
            fechar_tudo_ao_sair: false,
            salvar_desfazer_recentes: true,
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn find_open_path(&self, path: &Path) -> Option<usize> {
        self.tabs.iter().position(|t| {
            t.document
                .path()
                .is_some_and(|p| crate::file_io::same_file_path(p, path))
        })
    }

    pub fn find_first_pristine_untitled(&self) -> Option<usize> {
        self.tabs
            .iter()
            .position(|t| t.document.path().is_none() && t.is_pristine())
    }

    pub fn dirty_indices_menu_order(&self) -> Vec<usize> {
        (0..self.tabs.len()).filter(|&i| self.tabs[i].is_dirty()).collect()
    }

    pub fn insert_tab_at_top(&mut self, tab: Tab) {
        self.tabs.insert(0, tab);
        self.active_index = 0;
    }

    pub fn remove_tab_at(&mut self, index: usize) -> Tab {
        self.tabs.remove(index)
    }

    pub fn focus_index(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub fn tail_index(&self) -> Option<usize> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(self.tabs.len() - 1)
        }
    }

    pub fn needs_eviction(&self) -> bool {
        self.tabs.len() >= self.max_tabs
    }

    pub fn prepare_open_path(&mut self, path: &Path) -> WorkspaceAction {
        if let Some(index) = self.find_open_path(path) {
            self.active_index = index;
            return WorkspaceAction::FocusedExisting;
        }
        if self.needs_eviction() {
            let tail = self.tail_index().expect("tail");
            if self.tabs[tail].is_dirty() {
                return WorkspaceAction::PromptSaveRequired {
                    tab_index: tail,
                    reason: PromptReason::EvictTail,
                };
            }
        }
        WorkspaceAction::Ok
    }

    pub fn evict_tail_if_saved(&mut self) -> Option<PathBuf> {
        if !self.needs_eviction() {
            return None;
        }
        let tail = self.tail_index()?;
        if self.tabs[tail].is_dirty() {
            return None;
        }
        let removed = self.remove_tab_at(tail);
        if self.active_index >= self.tabs.len() && self.active_index > 0 {
            self.active_index = self.tabs.len() - 1;
        }
        removed.document.path().map(|p| p.to_path_buf())
    }

    pub fn prepare_new_tab(
        &self,
        active_pristine: bool,
    ) -> Result<Option<usize>, ()> {
        if active_pristine {
            return Ok(None);
        }
        if let Some(index) = self.find_first_pristine_untitled() {
            return Ok(Some(index));
        }
        Ok(None)
    }

    pub fn spawn_untitled_tab(
        &mut self,
        palette: &ThemePalette,
        encoding: FileEncoding,
        tabulation: Tabulation,
        word_wrap: bool,
        session_id: String,
    ) -> bool {
        if self.needs_eviction() {
            return false;
        }
        let counter = next_novo_counter(&self.tabs);
        let name = novo_display_name(counter);
        let tab = create_tab_from_defaults(
            palette,
            encoding,
            tabulation,
            word_wrap,
            session_id,
            name,
        );
        self.insert_tab_at_top(tab);
        true
    }

    pub fn after_close_tab(&mut self, closed_index: usize) {
        if self.tabs.is_empty() {
            return;
        }
        if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len().saturating_sub(1);
        } else if closed_index <= self.active_index && self.active_index > 0 {
            // focus stays on same slot which shifted; prefer index 0
            self.active_index = self.active_index.min(self.tabs.len() - 1);
        }
    }

    pub fn sort_tabs(&mut self, strategy: TabSortStrategy) {
        let active_id = self.tabs[self.active_index].session_id.clone();
        match strategy {
            TabSortStrategy::FileName => {
                self.tabs.sort_by(|a, b| a.menu_label().cmp(&b.menu_label()));
            }
            TabSortStrategy::FilePath => {
                self.tabs.sort_by(|a, b| {
                    match (a.document.path(), b.document.path()) {
                        (Some(pa), Some(pb)) => pa.cmp(pb),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.display_name.cmp(&b.display_name),
                    }
                });
            }
            TabSortStrategy::OpenedFirst => {
                self.tabs.sort_by_key(|t| t.opened_at);
            }
            TabSortStrategy::OpenedLast => {
                self.tabs.sort_by_key(|t| std::cmp::Reverse(t.opened_at));
            }
            TabSortStrategy::Status => {
                self.tabs.sort_by(|a, b| {
                    b.is_dirty()
                        .cmp(&a.is_dirty())
                        .then_with(|| a.menu_label().cmp(&b.menu_label()))
                });
            }
        }
        if let Some(i) = self.tabs.iter().position(|t| t.session_id == active_id) {
            self.active_index = i;
        }
    }
}

/// Copia editor/documento ativos da `App` para a aba, sem alterar o que o usuário vê.
pub fn flush_editor_into_tab(
    app_editor: &Editor,
    app_document: &Document,
    tab: &mut Tab,
    word_wrap: bool,
) {
    tab.document = app_document.clone();
    tab.editor.replace_content(&app_editor.content_string());
    tab.editor.set_tabulation(app_document.tabulation);
    tab.editor.set_word_wrap(word_wrap);
    let (line, col) = app_editor.cursor_line_col();
    tab.editor.set_cursor(line.saturating_sub(1), col.saturating_sub(1));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeId;

    fn palette() -> ThemePalette {
        ThemeId::ClassicBlue.palette()
    }

    fn empty_tab(id: &str, name: &str) -> Tab {
        create_tab_from_defaults(
            &palette(),
            FileEncoding::Utf8,
            Tabulation::Spaces4,
            false,
            id.to_string(),
            name.to_string(),
        )
    }

    #[test]
    fn evict_tail_when_at_capacity() {
        let mut ws = Workspace::with_single_tab(empty_tab("a", "Novo"));
        for i in 1..10 {
            ws.tabs.push(empty_tab(&format!("t{i}"), &format!("f{i}.txt")));
        }
        assert_eq!(ws.tabs.len(), 10);
        assert!(ws.needs_eviction());
        let path = ws.evict_tail_if_saved();
        assert!(path.is_none());
        assert_eq!(ws.tabs.len(), 9);
    }

    #[test]
    fn sort_preserves_active_tab() {
        let mut ws = Workspace::with_single_tab(empty_tab("b", "b.txt"));
        ws.tabs.push(empty_tab("a", "a.txt"));
        ws.active_index = 0;
        ws.sort_tabs(TabSortStrategy::FileName);
        assert_eq!(ws.tabs[ws.active_index].session_id, "b");
    }

    #[test]
    fn find_pristine_novo() {
        let mut ws = Workspace::with_single_tab(empty_tab("a", "Novo"));
        ws.tabs[0]
            .editor
            .engine_mut()
            .load_text("x");
        ws.tabs.push(empty_tab("b", "Novo1"));
        assert_eq!(ws.find_first_pristine_untitled(), Some(1));
    }
}
