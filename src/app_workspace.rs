//! Workspace de múltiplas abas — integração com `App`.

use std::path::{Path, PathBuf};

use crate::app::App;
use crate::config::{
    encoding_to_config_str, tabulation_to_config_str, AbasConfig, SessaoTabEntry,
};
use crate::editor_split::EditorSplit;
use crate::modal::Modal;
use crate::modal::dialog::DialogButtonAction;
use crate::session::{
    content_hash, now_ms, purge_all, purge_orphans, purge_tab, purge_undo, read_content_tmp,
    read_meta, read_undo_stacks, write_content_tmp, write_meta, write_undo_stacks, SessionMeta,
};
use crate::workspace::{
    check_fs_drift, check_fs_drift_from_entry, create_tab_from_defaults, new_session_id,
    next_novo_counter, novo_display_name, snapshot_path, flush_editor_into_tab, FsDrift,
    PromptReason, Tab, Workspace,
};

#[derive(Debug, Clone)]
pub enum PendingFsCheck {
    ReloadExternal { tab_index: usize },
    FileMissing { tab_index: usize },
}

pub struct WorkspaceBootstrap {
    pub workspace: Workspace,
    pub editor: Editor,
    pub document: Document,
    pub pending_fs_checks: Vec<PendingFsCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AfterDirtyResolved {
    Quit,
    CloseAll,
    CloseTab,
    EvictThenOpen(PathBuf),
    EvictThenNew,
    SaveAll,
}

#[derive(Debug, Clone)]
pub struct DirtyFlow {
    pub reason: PromptReason,
    pub pending: Vec<usize>,
    pub after: AfterDirtyResolved,
}

impl App {
    pub(crate) fn sync_active_tab(&mut self) {
        if self.workspace.tabs.is_empty() {
            return;
        }
        let idx = self
            .workspace
            .active_index
            .min(self.workspace.tabs.len() - 1);
        let word_wrap = self.view.word_wrap;
        let tab = &mut self.workspace.tabs[idx];
        flush_editor_into_tab(&self.editor, &self.document, tab, word_wrap);
    }

    pub(crate) fn focus_tab(&mut self, index: usize) {
        if index >= self.workspace.tabs.len() || index == self.workspace.active_index {
            return;
        }
        self.sync_active_tab();
        if let Some(path) = self.workspace.tabs[index].filepath().map(|p| p.to_path_buf()) {
            let snapshot = self.workspace.tabs[index].fs_snapshot.clone();
            match check_fs_drift(&path, snapshot.as_ref()) {
                FsDrift::ModifiedExternally => {
                    let name = self.tab_display_name(index);
                    self.pending_focus_tab = Some(index);
                    self.modal = Modal::reload_external(&name, index);
                    return;
                }
                FsDrift::Deleted => {
                    let name = self.tab_display_name(index);
                    self.pending_focus_tab = Some(index);
                    self.modal = Modal::file_missing(&name, index);
                    return;
                }
                FsDrift::Ok => {}
            }
        }
        self.focus_tab_unchecked(index);
    }

    pub(crate) fn focus_tab_unchecked(&mut self, index: usize) {
        if index >= self.workspace.tabs.len() {
            return;
        }
        self.pending_focus_tab = None;
        self.workspace.active_index = index;
        let history = self.workspace.tabs[index].editor.take_history_stacks();
        let tab = &self.workspace.tabs[index];
        let content = tab.editor.content_string();
        let (line, col) = tab.editor.cursor_line_col();
        self.document = tab.document.clone();
        self.editor.replace_content(&content);
        self.editor.set_tabulation(self.document.tabulation);
        self.editor.set_word_wrap(self.view.word_wrap);
        self.editor.set_cursor(line.saturating_sub(1), col.saturating_sub(1));
        self.editor.import_history(history);
        let palette = self.theme.palette();
        self.editor.apply_theme(&palette);
        self.sync_split_after_focus_tab(index);
    }

    pub(crate) fn process_pending_fs_checks(&mut self) {
        if self.modal.is_active() || self.pending_fs_checks.is_empty() {
            return;
        }
        let check = self.pending_fs_checks[0].clone();
        match check {
            PendingFsCheck::ReloadExternal { tab_index } => {
                let name = self.tab_display_name(tab_index);
                self.modal = Modal::reload_external(&name, tab_index);
            }
            PendingFsCheck::FileMissing { tab_index } => {
                let name = self.tab_display_name(tab_index);
                self.modal = Modal::file_missing(&name, tab_index);
            }
        }
    }

    fn pop_pending_fs_check(&mut self) {
        if !self.pending_fs_checks.is_empty() {
            self.pending_fs_checks.remove(0);
        }
    }

    pub(crate) fn reload_tab_from_disk(&mut self, tab_index: usize) -> bool {
        let path = match self.workspace.tabs.get(tab_index).and_then(|t| t.filepath()) {
            Some(p) => p.to_path_buf(),
            None => return false,
        };
        if !path.is_file() {
            return false;
        }
        let session_id = self.workspace.tabs[tab_index].session_id.clone();
        let _ = purge_undo(&session_id);
        let encoding = self.workspace.tabs[tab_index].document.encoding;
        let Ok(lines) = crate::file_io::read_lines_encoded(&path, encoding) else {
            return false;
        };
        let content = lines.join("\n");
        let mut document = self.workspace.tabs[tab_index].document.clone();
        document.set_opened(content.clone(), path.clone());
        let palette = self.theme.palette();
        let mut editor = Editor::new(&palette);
        editor.set_lines(lines);
        editor.set_tabulation(document.tabulation);
        editor.set_word_wrap(self.view.word_wrap);
        let tab = &mut self.workspace.tabs[tab_index];
        tab.document = document;
        tab.editor = editor;
        tab.fs_snapshot = snapshot_path(&path).ok();
        true
    }

    pub(crate) fn close_tab_by_index(&mut self, tab_index: usize) {
        if tab_index >= self.workspace.tabs.len() {
            return;
        }
        let removed = self.workspace.tabs.remove(tab_index);
        let _ = purge_tab(&removed.session_id);
        if self.workspace.tabs.is_empty() {
            let palette = self.theme.palette();
            let tab = create_tab_from_defaults(
                &palette,
                self.user_config().default_encoding(),
                self.user_config().default_tabulation(),
                self.view.word_wrap,
                new_session_id(),
                "Novo".to_string(),
            );
            self.workspace.tabs.push(tab);
        }
        self.workspace.active_index = self
            .workspace
            .active_index
            .min(self.workspace.tabs.len().saturating_sub(1));
        self.focus_tab_unchecked(self.workspace.active_index);
        self.on_tab_count_changed();
    }

    pub(crate) fn handle_reload_external(
        &mut self,
        tab_index: usize,
        action: DialogButtonAction,
        from_focus: bool,
    ) {
        match action {
            DialogButtonAction::Primary => {
                if self.reload_tab_from_disk(tab_index) {
                    if from_focus || tab_index == self.workspace.active_index {
                        self.focus_tab_unchecked(tab_index);
                    }
                    self.set_status("Arquivo recarregado do disco");
                } else {
                    self.set_status("Falha ao recarregar arquivo");
                }
            }
            DialogButtonAction::Secondary => {
                if from_focus {
                    self.focus_tab_unchecked(tab_index);
                }
            }
            DialogButtonAction::Cancel => {
                if from_focus {
                    self.pending_focus_tab = None;
                    self.set_status("Troca de aba cancelada");
                }
            }
        }
        self.pop_pending_fs_check();
    }

    pub(crate) fn handle_file_missing(
        &mut self,
        tab_index: usize,
        action: DialogButtonAction,
        from_focus: bool,
    ) {
        match action {
            DialogButtonAction::Primary => {
                if tab_index == self.workspace.active_index {
                    self.sync_active_tab();
                }
                self.close_tab_by_index(tab_index);
                self.set_status("Aba fechada — arquivo ausente");
            }
            DialogButtonAction::Secondary => {
                if let Some(tab) = self.workspace.tabs.get_mut(tab_index) {
                    tab.fs_snapshot = None;
                }
                if from_focus {
                    self.focus_tab_unchecked(tab_index);
                }
            }
            DialogButtonAction::Cancel => {
                if from_focus {
                    self.pending_focus_tab = None;
                }
            }
        }
        self.pop_pending_fs_check();
    }

    pub(crate) fn active_tab_is_pristine(&self) -> bool {
        !self.is_dirty()
            && self.editor.content_string() == crate::editor::EMPTY_DOCUMENT_TEXT
    }

    pub(crate) fn begin_dirty_flow(
        &mut self,
        indices: Vec<usize>,
        reason: PromptReason,
        after: AfterDirtyResolved,
    ) {
        if let Some(&index) = indices.first() {
            self.dirty_flow = Some(DirtyFlow {
                reason,
                pending: indices,
                after,
            });
            self.prompt_tab_unsaved(index, reason);
        } else {
            self.finish_after_dirty(after);
        }
    }

    fn prompt_tab_unsaved(&mut self, tab_index: usize, reason: PromptReason) {
        self.focus_tab(tab_index);
        let title = self.tab_display_name(tab_index);
        self.modal = Modal::tab_unsaved(&title, tab_index, reason);
    }

    pub(crate) fn tab_display_name(&self, index: usize) -> String {
        self.workspace.tabs[index].menu_label().trim_end_matches('*').to_string()
    }

    pub(crate) fn finish_after_dirty(&mut self, after: AfterDirtyResolved) {
        match after {
            AfterDirtyResolved::Quit => self.should_quit = true,
            AfterDirtyResolved::CloseAll => self.close_all_tabs_final(),
            AfterDirtyResolved::CloseTab => self.set_status("Aba fechada"),
            AfterDirtyResolved::EvictThenOpen(path) => self.open_path_after_evict(path),
            AfterDirtyResolved::EvictThenNew => self.create_new_tab_at_top(),
            AfterDirtyResolved::SaveAll => self.save_all_remaining(),
        }
    }

    pub(crate) fn advance_dirty_flow(&mut self, saved: bool) {
        let Some(flow) = self.dirty_flow.take() else {
            return;
        };
        let current = flow.pending.first().copied();
        let rest: Vec<usize> = flow.pending.into_iter().skip(1).collect();
        let next_index = rest.first().copied();

        if let Some(index) = current {
            if !saved {
                match flow.reason {
                    PromptReason::CloseTab | PromptReason::EvictTail => {
                        self.remove_tab_at(index);
                    }
                    PromptReason::Quit | PromptReason::CloseAll | PromptReason::OpenInPane => {}
                }
            } else if matches!(
                flow.reason,
                PromptReason::CloseTab | PromptReason::EvictTail
            ) {
                self.remove_tab_at(index);
            }
        }

        if rest.is_empty() {
            self.finish_after_dirty(flow.after);
        } else if let Some(next) = next_index {
            self.dirty_flow = Some(DirtyFlow {
                reason: flow.reason,
                pending: rest,
                after: flow.after,
            });
            self.prompt_tab_unsaved(next, flow.reason);
        }
    }

    pub(crate) fn cancel_dirty_flow(&mut self) {
        self.dirty_flow = None;
        self.pending_quit = false;
    }

    pub(crate) fn create_new_tab_at_top(&mut self) {
        if !self.workspace.tabs.is_empty() {
            self.sync_active_tab();
        }
        let session_id = new_session_id();
        let counter = next_novo_counter(&self.workspace.tabs);
        let name = novo_display_name(counter);
        let tab = create_tab_from_defaults(
            &self.theme.palette(),
            self.user_config().default_encoding(),
            self.user_config().default_tabulation(),
            self.view.word_wrap,
            session_id,
            name,
        );
        self.workspace.insert_tab_at_top(tab);
        self.focus_tab(0);
        self.refresh_menu();
        self.set_status("Novo documento");
    }

    pub(crate) fn ensure_pristine_singleton(&mut self) {
        if self.workspace.tabs.is_empty() {
            self.create_new_tab_at_top();
        }
    }

    pub(crate) fn remove_tab_at(&mut self, index: usize) {
        self.sync_active_tab();
        let removed = self.workspace.remove_tab_at(index);
        let _ = purge_tab(&removed.session_id);
        if let Some(path) = removed.document.path() {
            self.recent.push(path.to_path_buf());
            self.persist_user_config();
        }
        self.workspace.after_close_tab(index);
        self.on_tab_closed(index);
        if self.workspace.tabs.is_empty() {
            self.create_new_tab_at_top();
        } else {
            let focus = self.workspace.active_index.min(self.workspace.tabs.len() - 1);
            self.focus_tab(focus);
        }
    }

    pub(crate) fn close_active_tab_final(&mut self) {
        let index = self.workspace.active_index;
        self.remove_tab_at(index);
        self.set_status("Aba fechada");
    }

    pub(crate) fn close_all_tabs_final(&mut self) {
        self.sync_active_tab();
        while !self.workspace.tabs.is_empty() {
            let removed = self.workspace.remove_tab_at(0);
            let _ = purge_tab(&removed.session_id);
            if let Some(path) = removed.document.path() {
                self.recent.push(path.to_path_buf());
            }
        }
        self.persist_user_config();
        self.editor_split = EditorSplit::default();
        self.create_new_tab_at_top();
        self.set_status("Todas as abas fechadas");
    }

    pub(crate) fn evict_tail_if_needed(&mut self) -> bool {
        if !self.workspace.needs_eviction() {
            return true;
        }
        let tail = match self.workspace.tail_index() {
            Some(i) => i,
            None => return true,
        };
        if self.workspace.tabs[tail].is_dirty() {
            self.begin_dirty_flow(
                vec![tail],
                PromptReason::EvictTail,
                AfterDirtyResolved::EvictThenNew,
            );
            return false;
        }
        self.evict_tail_silent();
        true
    }

    pub(crate) fn evict_tail_silent(&mut self) {
        if let Some(path) = self.workspace.evict_tail_if_saved() {
            self.recent.push(path);
            self.persist_user_config();
        }
    }

    pub(crate) fn open_path_after_evict(&mut self, path: PathBuf) {
        self.evict_tail_silent();
        self.open_path_impl(path);
    }

    /// Garante aba para o caminho (abre se necessário) sem focar nem alterar painéis do split.
    pub(crate) fn ensure_tab_for_path(&mut self, path: PathBuf) -> Option<usize> {
        let path = crate::file_io::normalize_open_path(&path);
        if !path.is_file() {
            return None;
        }
        if let Some(index) = self.workspace.find_open_path(&path) {
            return Some(index);
        }

        if self.workspace.needs_eviction() {
            match self.workspace.prepare_open_path(&path) {
                crate::workspace::WorkspaceAction::PromptSaveRequired { .. } => {
                    return None;
                }
                _ => {}
            }
            self.evict_tail_silent();
        }

        match crate::file_io::read_lines_encoded(&path, self.document.encoding) {
            Ok(lines) => {
                let tab = self.tab_from_opened_path(&path, lines);
                self.workspace.insert_tab_at_top(tab);
                self.on_tab_count_changed();
                self.workspace.find_open_path(&path)
            }
            Err(_) => None,
        }
    }

    pub(crate) fn open_path_impl(&mut self, path: PathBuf) {
        let path = crate::file_io::normalize_open_path(&path);
        if let Some(index) = self.workspace.find_open_path(&path) {
            if self.split_active() {
                self.assign_tab_to_focused_pane(index);
            }
            self.focus_tab(index);
            self.set_status(format!("Focado: {}", path.display()));
            return;
        }

        match self.workspace.prepare_open_path(&path) {
            crate::workspace::WorkspaceAction::FocusedExisting => {
                let index = self.workspace.active_index;
                if self.split_active() {
                    self.assign_tab_to_focused_pane(index);
                }
                self.focus_tab(index);
            }
            crate::workspace::WorkspaceAction::PromptSaveRequired { tab_index, reason } => {
                self.begin_dirty_flow(
                    vec![tab_index],
                    reason,
                    AfterDirtyResolved::EvictThenOpen(path),
                );
                return;
            }
            crate::workspace::WorkspaceAction::Ok => {}
        }

        self.evict_tail_silent();

        if self.split_active() {
            if let Some(idx) = self
                .editor_split
                .pane_tab_index(self.editor_split.focused_pane)
            {
                if idx < self.workspace.tabs.len() && self.workspace.tabs[idx].is_dirty() {
                    self.begin_dirty_flow(
                        vec![idx],
                        PromptReason::OpenInPane,
                        AfterDirtyResolved::EvictThenOpen(path),
                    );
                    return;
                }
            }
        }

        match crate::file_io::read_lines_encoded(&path, self.document.encoding) {
            Ok(lines) => {
                let tab = self.tab_from_opened_path(&path, lines);
                self.install_opened_tab(tab, &path);
            }
            Err(error) => {
                self.set_status(format!("Erro ao abrir: {error}"));
            }
        }
    }

    fn assign_tab_to_focused_pane(&mut self, index: usize) {
        if !self.split_active() {
            return;
        }
        self.editor_split
            .set_pane_tab(self.editor_split.focused_pane, index);
    }

    fn tab_from_opened_path(&self, path: &Path, lines: Vec<String>) -> Tab {
        let content = lines.join("\n");
        let session_id = new_session_id();
        let mut document = Document::new();
        document.encoding = self.document.encoding;
        document.tabulation = self.document.tabulation;
        document.set_opened(content, path.to_path_buf());
        let mut editor = Editor::new(&self.theme.palette());
        editor.set_lines(lines);
        editor.set_tabulation(document.tabulation);
        editor.set_word_wrap(self.view.word_wrap);
        let fs_snapshot = snapshot_path(path).ok();
        let mut tab = Tab::new_untitled(
            editor,
            document,
            session_id,
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
        );
        tab.fs_snapshot = fs_snapshot;
        tab
    }

    fn replace_tab_at(&mut self, index: usize, new_tab: Tab) {
        let old_session = self.workspace.tabs[index].session_id.clone();
        let _ = purge_tab(&old_session);
        self.workspace.tabs[index] = new_tab;
        self.focus_tab_unchecked(index);
        if self.split_active() {
            self.editor_split
                .set_pane_tab(self.editor_split.focused_pane, index);
        }
    }

    fn install_opened_tab(&mut self, tab: Tab, path: &Path) {
        if self.split_active() {
            let pane = self.editor_split.focused_pane;
            if let Some(index) = self.editor_split.pane_tab_index(pane) {
                self.replace_tab_at(index, tab);
            } else {
                let index = self.workspace.tabs.len();
                self.workspace.tabs.push(tab);
                self.editor_split.set_pane_tab(pane, index);
                self.focus_tab(index);
                self.on_tab_count_changed();
            }
        } else {
            self.workspace.insert_tab_at_top(tab);
            self.focus_tab_unchecked(0);
            self.on_tab_count_changed();
        }
        self.recent.remove_path(path);
        self.persist_user_config();
        self.set_status(format!("Aberto: {}", path.display()));
    }

    fn note_recent_file_on_open_only(&mut self) {
        self.persist_user_config();
    }

    pub(crate) fn build_abas_config(&self) -> AbasConfig {
        let mut abas = AbasConfig {
            fechar_tudo_ao_sair: self.workspace.fechar_tudo_ao_sair,
            salvar_desfazer_recentes: self.workspace.salvar_desfazer_recentes,
            indice_ativo: self.workspace.active_index,
            limite: self.workspace.max_tabs,
            sessao: Vec::new(),
        };
        if self.workspace.fechar_tudo_ao_sair {
            return abas;
        }
        for tab in &self.workspace.tabs {
            // `cursor_line_col` já é 1-based (ex.: linha 1 / col 1 = início do arquivo).
            let (line, col) = tab.editor.cursor_line_col();
            let entry = SessaoTabEntry {
                tab_id: tab.session_id.clone(),
                caminho: tab
                    .document
                    .path()
                    .map(|p| p.display().to_string()),
                nome_virtual: if tab.document.path().is_none() {
                    Some(tab.display_name.clone())
                } else {
                    None
                },
                temporario: tab.document.path().is_none(),
                cursor_linha: line,
                cursor_coluna: col,
                encoding: encoding_to_config_str(tab.document.encoding),
                tabulacao: tabulation_to_config_str(tab.document.tabulation),
                fs_mtime_ms: tab.fs_snapshot.as_ref().and_then(|s| {
                    s.modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_millis() as u64)
                }),
                fs_len: tab.fs_snapshot.as_ref().map(|s| s.len),
            };
            abas.sessao.push(entry);
        }
        abas
    }

    pub(crate) fn persist_session_artifacts(&mut self) {
        self.sync_active_tab();
        if self.workspace.fechar_tudo_ao_sair {
            let _ = purge_all();
            return;
        }

        for tab in &self.workspace.tabs {
            let content = tab.editor.content_string();
            let _ = write_content_tmp(&tab.session_id, &content);
            if self.workspace.salvar_desfazer_recentes {
                let (line, col) = tab.editor.cursor_line_col();
                let meta = SessionMeta {
                    content_hash: content_hash(&content),
                    cursor_linha: line,
                    cursor_coluna: col,
                    encoding: encoding_to_config_str(tab.document.encoding),
                    fs_mtime_ms: tab.fs_snapshot.as_ref().and_then(|s| {
                        crate::session::system_time_to_ms(s.modified)
                    }),
                    fs_len: tab.fs_snapshot.as_ref().map(|s| s.len),
                    saved_at_ms: now_ms(),
                };
                let _ = write_meta(&tab.session_id, &meta);
                let stacks = tab.editor.export_history_for_persist();
                let _ = write_undo_stacks(&tab.session_id, &stacks);
            } else {
                let _ = purge_undo(&tab.session_id);
            }
        }

        let ids: Vec<String> = self
            .workspace
            .tabs
            .iter()
            .map(|t| t.session_id.clone())
            .collect();
        let _ = purge_orphans(&ids);
    }

    pub(crate) fn save_all_dirty(&mut self) {
        self.sync_active_tab();
        let dirty: Vec<usize> = self.workspace.dirty_indices_menu_order();
        if dirty.is_empty() {
            self.set_status("Nada a salvar");
            return;
        }
        self.begin_dirty_flow(dirty, PromptReason::CloseAll, AfterDirtyResolved::SaveAll);
    }

    pub(crate) fn save_all_remaining(&mut self) {
        self.sync_active_tab();
        for index in self.workspace.dirty_indices_menu_order() {
            self.focus_tab(index);
            if self.document.path().is_some() {
                if let Some(path) = self.document.path().map(Path::to_path_buf) {
                    self.save_to_path_internal(path, false);
                }
            } else {
                self.open_file_browser(crate::modal::FileBrowserMode::SaveAs);
                self.pending_save_all = true;
                return;
            }
        }
        self.set_status("Todos salvos");
    }

    pub(crate) fn focus_tab_relative(&mut self, delta: isize) {
        let len = self.workspace.tabs.len();
        if len <= 1 {
            return;
        }
        let current = self.workspace.active_index as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        self.focus_tab(next);
    }

    pub(crate) fn focus_tab_by_menu_number(&mut self, n: usize) {
        if n == 0 {
            if self.workspace.tabs.len() >= 10 {
                self.focus_tab(9);
            }
        } else if n <= self.workspace.tabs.len() {
            self.focus_tab(n - 1);
        }
    }

    pub(crate) fn handle_tab_unsaved_modal(
        &mut self,
        tab_index: usize,
        reason: PromptReason,
        action: DialogButtonAction,
    ) {
        match action {
            DialogButtonAction::Primary => {
                if self.document.path().is_some() {
                    if let Some(path) = self.document.path().map(Path::to_path_buf) {
                        self.save_to_path_internal(path, false);
                    }
                    if !self.is_dirty() {
                        self.advance_dirty_flow(true);
                    } else {
                        self.dirty_flow = Some(DirtyFlow {
                            reason,
                            pending: vec![tab_index],
                            after: self
                                .dirty_flow
                                .take()
                                .map(|f| f.after)
                                .unwrap_or(AfterDirtyResolved::Quit),
                        });
                    }
                } else {
                    self.open_file_browser(crate::modal::FileBrowserMode::SaveAs);
                    self.pending_dirty_save = Some((tab_index, reason));
                }
            }
            DialogButtonAction::Secondary => self.advance_dirty_flow(false),
            DialogButtonAction::Cancel => {
                self.cancel_dirty_flow();
                self.set_status("Ação cancelada");
            }
        }
    }
}

use crate::document::Document;
use crate::editor::Editor;

/// Conteúdo da aba na sessão: `content.tmp` se bater com `meta.json`, senão fallback.
fn session_editor_content(tab_id: &str, fallback: &str) -> String {
    let Ok(Some(tmp)) = read_content_tmp(tab_id) else {
        return fallback.to_string();
    };
    let Ok(Some(meta)) = read_meta(tab_id) else {
        return fallback.to_string();
    };
    if content_hash(&tmp) == meta.content_hash {
        tmp
    } else {
        fallback.to_string()
    }
}

fn restore_tab_undo(
    tab: &mut Tab,
    tab_id: &str,
    content: &str,
    fs_mtime_ms: Option<u64>,
    fs_len: Option<u64>,
    path: Option<&Path>,
    salvar_desfazer: bool,
) {
    if !salvar_desfazer {
        let _ = purge_undo(tab_id);
        return;
    }
    let hash = content_hash(content);
    let Some(meta) = read_meta(tab_id).ok().flatten() else {
        let _ = purge_undo(tab_id);
        return;
    };
    if meta.content_hash != hash {
        let _ = purge_undo(tab_id);
        return;
    }
    if let Some(path) = path {
        if check_fs_drift_from_entry(path, fs_mtime_ms, fs_len) != FsDrift::Ok {
            let _ = purge_undo(tab_id);
            return;
        }
    }
    if let Ok(Some(stacks)) = read_undo_stacks(tab_id) {
        tab.editor.import_history(stacks);
    }
}

pub fn workspace_from_config(
    user_config: &crate::config::EditConfig,
    palette: &crate::theme::ThemePalette,
    word_wrap: bool,
    restore_session: bool,
) -> WorkspaceBootstrap {
    let abas = &user_config.arquivo.abas;
    let mut pending_fs_checks = Vec::new();
    let mut workspace = Workspace {
        tabs: Vec::new(),
        active_index: 0,
        max_tabs: abas.limite.clamp(1, 10),
        fechar_tudo_ao_sair: abas.fechar_tudo_ao_sair,
        salvar_desfazer_recentes: abas.salvar_desfazer_recentes,
    };

    if restore_session && !abas.fechar_tudo_ao_sair && !abas.sessao.is_empty() {
        for entry in &abas.sessao {
            if let Some(path_str) = &entry.caminho {
                let path = crate::file_io::normalize_open_path(Path::new(path_str));
                if workspace.find_open_path(&path).is_some() {
                    continue;
                }
                if path.is_file() {
                    if let Ok(lines) =
                        crate::file_io::read_lines_encoded(&path, user_config.default_encoding())
                    {
                        let tab_index = workspace.tabs.len();
                        let drift =
                            check_fs_drift_from_entry(&path, entry.fs_mtime_ms, entry.fs_len);
                        if drift == FsDrift::ModifiedExternally {
                            pending_fs_checks.push(PendingFsCheck::ReloadExternal { tab_index });
                        }
                        let disk_content = crate::encoding::normalize_newlines(&lines.join("\n"));
                        let editor_content = crate::encoding::normalize_newlines(
                            &session_editor_content(&entry.tab_id, &disk_content),
                        );
                        let mut document = Document::new();
                        document.encoding = user_config.default_encoding();
                        document.tabulation = user_config.default_tabulation();
                        document.set_opened(disk_content.clone(), path.clone());
                        let mut editor = Editor::new(palette);
                        editor.replace_content(&editor_content);
                        editor.set_tabulation(document.tabulation);
                        editor.set_word_wrap(word_wrap);
                        editor.set_cursor(
                            entry.cursor_linha.saturating_sub(1),
                            entry.cursor_coluna.saturating_sub(1),
                        );
                        let fs_snapshot = snapshot_path(&path).ok();
                        let mut tab = Tab::new_untitled(
                            editor,
                            document,
                            entry.tab_id.clone(),
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("?")
                                .to_string(),
                        );
                        tab.fs_snapshot = fs_snapshot;
                        restore_tab_undo(
                            &mut tab,
                            &entry.tab_id,
                            &editor_content,
                            entry.fs_mtime_ms,
                            entry.fs_len,
                            Some(path.as_path()),
                            abas.salvar_desfazer_recentes,
                        );
                        workspace.tabs.push(tab);
                        continue;
                    }
                } else if entry.caminho.is_some() {
                    continue;
                }
            } else if entry.temporario {
                let content = read_content_tmp(&entry.tab_id)
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| crate::editor::EMPTY_DOCUMENT_TEXT.to_string());
                let mut document = Document::new();
                document.encoding = user_config.default_encoding();
                document.tabulation = user_config.default_tabulation();
                document.restore_untitled(
                    content.clone(),
                    user_config.default_encoding(),
                    user_config.default_tabulation(),
                );
                let mut editor = Editor::new(palette);
                editor.replace_content(&content);
                editor.set_tabulation(document.tabulation);
                editor.set_word_wrap(word_wrap);
                editor.set_cursor(
                    entry.cursor_linha.saturating_sub(1),
                    entry.cursor_coluna.saturating_sub(1),
                );
                let name = entry
                    .nome_virtual
                    .clone()
                    .unwrap_or_else(|| "Novo".to_string());
                let mut tab = Tab::new_untitled(editor, document, entry.tab_id.clone(), name);
                restore_tab_undo(
                    &mut tab,
                    &entry.tab_id,
                    &content,
                    entry.fs_mtime_ms,
                    entry.fs_len,
                    None,
                    abas.salvar_desfazer_recentes,
                );
                workspace.tabs.push(tab);
            }
        }
    }

    if !workspace.tabs.is_empty() {
        workspace.active_index = abas
            .indice_ativo
            .min(workspace.tabs.len().saturating_sub(1));
    }

    if workspace.tabs.is_empty() {
        let session_id = new_session_id();
        let tab = create_tab_from_defaults(
            palette,
            user_config.default_encoding(),
            user_config.default_tabulation(),
            word_wrap,
            session_id,
            "Novo".to_string(),
        );
        workspace.tabs.push(tab);
        workspace.active_index = 0;
    }

    workspace.active_index = workspace
        .active_index
        .min(workspace.tabs.len().saturating_sub(1));
    let active_idx = workspace.active_index;
    let document = workspace.tabs[active_idx].document.clone();
    let content = workspace.tabs[active_idx].editor.content_string();
    let (line, col) = workspace.tabs[active_idx].editor.cursor_line_col();
    let history = workspace.tabs[active_idx].editor.take_history_stacks();
    let mut editor = Editor::new(palette);
    editor.replace_content(&content);
    editor.set_tabulation(document.tabulation);
    editor.set_word_wrap(word_wrap);
    editor.import_history(history);
    editor.set_cursor(line.saturating_sub(1), col.saturating_sub(1));
    WorkspaceBootstrap {
        workspace,
        editor,
        document,
        pending_fs_checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::config::{
        clear_config_path_override, set_config_path_for_tests, AbasConfig, EditConfig,
        SessaoTabEntry,
    };
    use crate::theme::ThemeId;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    static APP_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct IsolatedConfigGuard {
        _lock: MutexGuard<'static, ()>,
        path: PathBuf,
    }

    impl IsolatedConfigGuard {
        fn new(name: &str) -> Self {
            let lock = APP_TEST_LOCK.lock().unwrap();
            let path = std::env::temp_dir().join(format!(
                "edit-app-test-{name}-{}.json",
                std::process::id()
            ));
            let _ = std::fs::remove_file(&path);
            set_config_path_for_tests(path.clone());
            EditConfig::default().save().unwrap();
            Self { _lock: lock, path }
        }
    }

    impl Drop for IsolatedConfigGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
            clear_config_path_override();
        }
    }

    fn sample_config(fechar_tudo_ao_sair: bool, sessao: Vec<SessaoTabEntry>) -> EditConfig {
        let mut config = EditConfig::default();
        config.arquivo.abas = AbasConfig {
            fechar_tudo_ao_sair,
            salvar_desfazer_recentes: true,
            indice_ativo: 0,
            limite: 10,
            sessao,
        };
        config
    }

    #[test]
    fn restore_virtual_name_when_fechar_tudo_desligado() {
        let config = sample_config(
            false,
            vec![SessaoTabEntry {
                tab_id: "t1".into(),
                caminho: None,
                nome_virtual: Some("Salvo".into()),
                temporario: true,
                cursor_linha: 1,
                cursor_coluna: 1,
                encoding: "utf-8".into(),
                tabulacao: "4".into(),
                fs_mtime_ms: None,
                fs_len: None,
            }],
        );
        let palette = ThemeId::ClassicBlue.palette();
        let boot = workspace_from_config(&config, &palette, false, true);
        assert_eq!(boot.workspace.tabs[0].display_name, "Salvo");
    }

    #[test]
    fn sync_active_tab_preserves_app_editor_content() {
        let _guard = IsolatedConfigGuard::new("sync-active");
        let mut app = App::new(false, true);
        app.editor.replace_content("# README\n\nConteúdo editado");
        app.sync_active_tab();
        assert_eq!(
            app.editor.content_string(),
            "# README\n\nConteúdo editado"
        );
        assert_eq!(
            app.workspace.tabs[0].editor.content_string(),
            "# README\n\nConteúdo editado"
        );
    }

    #[test]
    fn saved_cursor_position_roundtrips_without_drift() {
        let _guard = IsolatedConfigGuard::new("cursor-roundtrip");
        let mut app = App::new(false, true);
        app.editor.set_cursor(0, 0);
        app.sync_active_tab();

        let abas = app.build_abas_config();
        assert_eq!(abas.sessao[0].cursor_linha, 1);
        assert_eq!(abas.sessao[0].cursor_coluna, 1);

        let config = sample_config(false, abas.sessao);
        let palette = ThemeId::ClassicBlue.palette();
        let boot = workspace_from_config(&config, &palette, false, true);
        assert_eq!(boot.workspace.tabs[0].editor.cursor_line_col(), (1, 1));
    }

    #[test]
    fn history_survives_tab_switch_via_app() {
        let _guard = IsolatedConfigGuard::new("history-switch");
        let mut app = App::new(false, true);
        app.editor.paste("z");
        app.sync_active_tab();
        let depth = app.editor.history_depth();
        assert!(depth > 0);
        let session_b = new_session_id();
        let tab_b = create_tab_from_defaults(
            &ThemeId::ClassicBlue.palette(),
            app.user_config().default_encoding(),
            app.user_config().default_tabulation(),
            false,
            session_b,
            "Segunda".to_string(),
        );
        app.workspace.tabs.push(tab_b);
        app.focus_tab(1);
        app.focus_tab(0);
        assert_eq!(app.editor.history_depth(), depth);
        app.editor.undo();
        assert_eq!(app.editor.content_string(), "");
    }

    #[test]
    fn undo_survives_session_restart_for_path_tab() {
        use crate::session::{
            clear_session_root_override, set_session_root, write_content_tmp, write_meta,
            write_undo_stacks, SessionMeta,
        };
        use crate::session::test_lock;
        use std::sync::MutexGuard;

        struct Guard {
            _lock: MutexGuard<'static, ()>,
            root: std::path::PathBuf,
        }
        impl Guard {
            fn new() -> Self {
                let lock = test_lock();
                let root = std::env::temp_dir().join(format!(
                    "edit-undo-restart-{}",
                    std::process::id()
                ));
                let _ = std::fs::remove_dir_all(&root);
                set_session_root(root.clone());
                Self { _lock: lock, root }
            }
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.root);
                clear_session_root_override();
            }
        }
        let _guard = Guard::new();

        let path = std::env::temp_dir().join(format!("edit-undo-file-{}.txt", std::process::id()));
        std::fs::write(&path, "hello").unwrap();

        let tab_id = "tab-undo-restart".to_string();
        let edited = "hell";
        let _ = write_content_tmp(&tab_id, edited);
        let meta = SessionMeta {
            content_hash: content_hash(edited),
            cursor_linha: 1,
            cursor_coluna: 5,
            encoding: "utf-8".into(),
            fs_mtime_ms: None,
            fs_len: None,
            saved_at_ms: now_ms(),
        };
        let _ = write_meta(&tab_id, &meta);
        let mut history = crate::editor::history::EditHistory::new();
        history.record_change(4, "o".into(), String::new(), 5, 4);
        let stacks = history.export_stacks();
        let _ = write_undo_stacks(&tab_id, &stacks);

        let config = sample_config(
            false,
            vec![SessaoTabEntry {
                tab_id: tab_id.clone(),
                caminho: Some(path.display().to_string()),
                nome_virtual: None,
                temporario: false,
                cursor_linha: 1,
                cursor_coluna: 5,
                encoding: "utf-8".into(),
                tabulacao: "4".into(),
                fs_mtime_ms: std::fs::metadata(&path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_millis() as u64)
                    }),
                fs_len: Some(5),
            }],
        );
        let palette = ThemeId::ClassicBlue.palette();
        let boot = workspace_from_config(&config, &palette, false, true);
        assert_eq!(boot.editor.content_string(), "hell");
        assert!(boot.editor.history_depth() > 0);
        let mut editor = boot.editor;
        editor.undo();
        assert_eq!(editor.content_string(), "hello");
        let _ = std::fs::remove_file(path);
    }

    fn skip_restore_when_fechar_tudo_ligado() {
        let config = sample_config(
            true,
            vec![SessaoTabEntry {
                tab_id: "t1".into(),
                caminho: None,
                nome_virtual: Some("Salvo".into()),
                temporario: true,
                cursor_linha: 1,
                cursor_coluna: 1,
                encoding: "utf-8".into(),
                tabulacao: "4".into(),
                fs_mtime_ms: None,
                fs_len: None,
            }],
        );
        let palette = ThemeId::ClassicBlue.palette();
        let boot = workspace_from_config(&config, &palette, false, true);
        assert_eq!(boot.workspace.tabs.len(), 1);
        assert_eq!(boot.workspace.tabs[0].display_name, "Novo");
    }

    #[test]
    fn cli_open_loads_file_content_into_editor() {
        let _guard = IsolatedConfigGuard::new("cli-open");
        let path =
            std::env::temp_dir().join(format!("edit-cli-open-{}.txt", std::process::id()));
        std::fs::write(&path, "conteudo cli").unwrap();

        let mut app = App::new(false, true);
        app.open_cli_files(&[path.clone()]);
        assert_eq!(app.editor_text_for_test(), "conteudo cli");
        assert!(app.workspace.tabs[0].filepath().is_some());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn restore_session_flag_controls_workspace_bootstrap() {
        let config = sample_config(
            false,
            vec![SessaoTabEntry {
                tab_id: "t-old".into(),
                caminho: None,
                nome_virtual: Some("Antigo".into()),
                temporario: true,
                cursor_linha: 1,
                cursor_coluna: 1,
                encoding: "utf-8".into(),
                tabulacao: "4".into(),
                fs_mtime_ms: None,
                fs_len: None,
            }],
        );
        let palette = ThemeId::ClassicBlue.palette();
        let boot = workspace_from_config(&config, &palette, false, true);
        assert_eq!(boot.workspace.tabs[0].display_name, "Antigo");
        let boot_off = workspace_from_config(&config, &palette, false, false);
        assert_eq!(boot_off.workspace.tabs[0].display_name, "Novo");
    }

    #[test]
    fn cli_two_files_open_in_split_editors() {
        let _guard = IsolatedConfigGuard::new("cli-split");
        let dir = std::env::temp_dir().join(format!("edit-cli-split-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        std::fs::write(&a, "alfa").unwrap();
        std::fs::write(&b, "beta").unwrap();

        let mut app = App::new(false, false);
        app.open_cli_files(&[a.clone(), b.clone()]);
        assert!(app.editor_split.is_active());
        let idx_a = app.workspace.find_open_path(&a).unwrap();
        let idx_b = app.workspace.find_open_path(&b).unwrap();
        assert_eq!(app.editor_split.left_tab, idx_a);
        assert_eq!(app.editor_split.right_tab, Some(idx_b));
        assert_ne!(idx_a, idx_b);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cli_reuses_existing_tab_without_duplicate() {
        let _guard = IsolatedConfigGuard::new("cli-dup");
        let dir = std::env::temp_dir().join(format!("edit-cli-dup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("dup.txt");
        std::fs::write(&file, "x").unwrap();

        let mut app = App::new(false, false);
        app.open_cli_files(&[file.clone()]);
        let count_after_first = app.workspace.tabs.len();
        app.open_cli_files(&[file.clone()]);
        assert_eq!(app.workspace.tabs.len(), count_after_first);
        assert_eq!(
            app.workspace
                .tabs
                .iter()
                .filter(|t| t
                    .filepath()
                    .is_some_and(|p| crate::file_io::same_file_path(p, &file)))
                .count(),
            1
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cli_keeps_existing_workspace_tabs_when_opening_file() {
        let _guard = IsolatedConfigGuard::new("cli-keep");
        let path =
            std::env::temp_dir().join(format!("edit-cli-keep-{}.txt", std::process::id()));
        std::fs::write(&path, "arquivo cli").unwrap();

        let mut app = App::new(false, false);
        let tab_b = create_tab_from_defaults(
            &ThemeId::ClassicBlue.palette(),
            app.user_config().default_encoding(),
            app.user_config().default_tabulation(),
            false,
            new_session_id(),
            "Outra".to_string(),
        );
        app.workspace.tabs.push(tab_b);

        app.open_cli_files(&[path.clone()]);
        assert_eq!(app.workspace.tabs.len(), 3);
        assert_eq!(app.workspace.active_index, 0);
        assert!(app.workspace.find_open_path(&path).is_some());
        assert!(
            app.workspace
                .tabs
                .iter()
                .any(|t| t.display_name == "Outra")
        );
        assert_eq!(app.editor_text_for_test(), "arquivo cli");
        let _ = std::fs::remove_file(path);
    }
}
