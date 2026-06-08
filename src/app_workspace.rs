//! Workspace de múltiplas abas — integração com `App`.

use std::path::{Path, PathBuf};

use crate::app::App;
use crate::config::{
    encoding_to_config_str, tabulation_to_config_str, AbasConfig, SessaoTabEntry,
};
use crate::modal::Modal;
use crate::modal::dialog::DialogButtonAction;
use crate::session::{purge_orphans, purge_tab, read_content_tmp, write_content_tmp};
use crate::workspace::{
    create_tab_from_defaults, new_session_id, next_novo_counter, novo_display_name, snapshot_path,
    swap_active_with_tab, PromptReason, Tab, Workspace,
};

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
        let idx = self.workspace.active_index;
        let tab = &mut self.workspace.tabs[idx];
        swap_active_with_tab(&mut self.editor, &mut self.document, tab);
    }

    pub(crate) fn focus_tab(&mut self, index: usize) {
        if index >= self.workspace.tabs.len() {
            return;
        }
        if index != self.workspace.active_index {
            self.sync_active_tab();
            self.workspace.active_index = index;
            let tab = &mut self.workspace.tabs[index];
            swap_active_with_tab(&mut self.editor, &mut self.document, tab);
        }
        self.editor.set_tabulation(self.document.tabulation);
        self.editor.set_word_wrap(self.view.word_wrap);
        let palette = self.theme.palette();
        self.editor.apply_theme(&palette);
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
            AfterDirtyResolved::CloseTab => self.close_active_tab_final(),
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
                    PromptReason::Quit | PromptReason::CloseAll => {}
                }
            } else if matches!(flow.reason, PromptReason::CloseTab) {
                self.remove_tab_at(index);
            } else if matches!(flow.reason, PromptReason::EvictTail) {
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
        self.sync_active_tab();
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

    pub(crate) fn open_path_impl(&mut self, path: PathBuf) {
        if let Some(index) = self.workspace.find_open_path(&path) {
            self.focus_tab(index);
            self.set_status(format!("Focado: {}", path.display()));
            return;
        }

        match self.workspace.prepare_open_path(&path) {
            crate::workspace::WorkspaceAction::FocusedExisting => {
                self.focus_tab(self.workspace.active_index);
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

        match crate::file_io::read_lines_encoded(&path, self.document.encoding) {
            Ok(lines) => {
                let content = lines.join("\n");
                let session_id = new_session_id();
                let mut document = Document::new();
                document.encoding = self.document.encoding;
                document.tabulation = self.document.tabulation;
                document.set_opened(content.clone(), path.clone());
                let mut editor = Editor::new(&self.theme.palette());
                editor.set_lines(lines);
                editor.set_tabulation(document.tabulation);
                editor.set_word_wrap(self.view.word_wrap);
                let fs_snapshot = snapshot_path(&path).ok();
                let tab = Tab::new_untitled(
                    editor,
                    document,
                    session_id,
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string(),
                );
                let mut tab = tab;
                tab.fs_snapshot = fs_snapshot;
                self.workspace.insert_tab_at_top(tab);
                self.focus_tab(0);
                self.recent.remove_path(&path);
                self.persist_user_config();
                self.set_status(format!("Aberto: {}", path.display()));
            }
            Err(error) => {
                self.set_status(format!("Erro ao abrir: {error}"));
            }
        }
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
        if !self.workspace.fechar_tudo_ao_sair {
            return abas;
        }
        for tab in &self.workspace.tabs {
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
                cursor_linha: line.saturating_add(1),
                cursor_coluna: col.saturating_add(1),
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
        if !self.workspace.fechar_tudo_ao_sair {
            let ids: Vec<String> = self
                .workspace
                .tabs
                .iter()
                .map(|t| t.session_id.clone())
                .collect();
            let _ = purge_orphans(&ids);
            return;
        }

        for tab in &self.workspace.tabs {
            if tab.document.path().is_none() {
                let content = tab.editor.content_string();
                let _ = write_content_tmp(&tab.session_id, &content);
            }
        }

        if !self.workspace.salvar_desfazer_recentes {
            let _ = crate::session::purge_all_undo();
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
                self.modal = Modal::path_input("Salvar como", "Caminho:", crate::modal::PathInputKind::SaveAs);
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
                    self.modal = Modal::path_input(
                        "Salvar como",
                        "Caminho:",
                        crate::modal::PathInputKind::SaveAs,
                    );
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

pub fn workspace_from_config(
    user_config: &crate::config::EditConfig,
    palette: &crate::theme::ThemePalette,
    word_wrap: bool,
) -> (Workspace, Editor, Document) {
    let abas = &user_config.arquivo.abas;
    let mut workspace = Workspace {
        tabs: Vec::new(),
        active_index: 0,
        max_tabs: abas.limite.clamp(1, 10),
        fechar_tudo_ao_sair: abas.fechar_tudo_ao_sair,
        salvar_desfazer_recentes: abas.salvar_desfazer_recentes,
    };

    if abas.fechar_tudo_ao_sair && !abas.sessao.is_empty() {
        for entry in &abas.sessao {
            if let Some(path_str) = &entry.caminho {
                let path = PathBuf::from(path_str);
                if path.is_file() {
                    if let Ok(lines) =
                        crate::file_io::read_lines_encoded(&path, user_config.default_encoding())
                    {
                        let content = lines.join("\n");
                        let mut document = Document::new();
                        document.encoding = user_config.default_encoding();
                        document.tabulation = user_config.default_tabulation();
                        document.set_opened(content, path.clone());
                        let mut editor = Editor::new(palette);
                        editor.set_lines(lines);
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
                        workspace.tabs.push(tab);
                        continue;
                    }
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
                let tab = Tab::new_untitled(editor, document, entry.tab_id.clone(), name);
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
    let mut editor = Editor::new(palette);
    let mut document = Document::new();
    document.encoding = user_config.default_encoding();
    document.tabulation = user_config.default_tabulation();
    swap_active_with_tab(
        &mut editor,
        &mut document,
        &mut workspace.tabs[active_idx],
    );
    (workspace, editor, document)
}
