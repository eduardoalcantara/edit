use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cli;
use crate::clipboard::Clipboard;
use crate::app_workspace::{workspace_from_config, AfterDirtyResolved, DirtyFlow};
use crate::config::{config_from_view, EditConfig};
use crate::document::Document;
use crate::editor::Editor;
use crate::encoding::{FileEncoding, Tabulation};
use crate::editor::convert_tabulation_between;
use crate::events;
use crate::file_io;
use crate::memory::MemoryMonitor;
use crate::menus::{ActionId, MenuBar, MenuState};
use crate::modal::{ConfirmKind, DialogButtonAction, Modal, PathInputKind};
use crate::recent::RecentFiles;
use crate::theme::ThemeId;
use crate::ui;
use crate::session::has_any_undo_files;
use crate::view_state::{EditorBorder, EditorMargin, GuideColumn, ViewState};
use crate::workspace::{PromptReason, TabSortStrategy, Workspace};

pub struct App {
    pub editor: Editor,
    pub document: Document,
    pub theme: ThemeId,
    pub view: ViewState,
    pub recent: RecentFiles,
    pub clipboard: Clipboard,
    pub menu_bar: MenuBar,
    pub menu_state: MenuState,
    pub find_pattern: String,
    pub should_quit: bool,
    pub pending_quit: bool,
    pub status_message: String,
    pub mouse_enabled: bool,
    pub modal: Modal,
    pub is_ssh_session: bool,
    pub last_frame_width: u16,
    pub last_frame_height: u16,
    pub memory: MemoryMonitor,
    pub workspace: Workspace,
    pub dirty_flow: Option<DirtyFlow>,
    pub pending_dirty_save: Option<(usize, PromptReason)>,
    pub pending_save_all: bool,
    pub pending_open_path: Option<PathBuf>,
    user_config: EditConfig,
}

impl App {
    pub(crate) fn user_config(&self) -> &EditConfig {
        &self.user_config
    }

    pub fn new(mouse_enabled: bool) -> Self {
        let user_config = EditConfig::load();
        let view_snapshot = user_config.view_settings();
        let theme = view_snapshot.theme;
        let palette = theme.palette();
        let recent = RecentFiles::from_paths(user_config.recent_paths());
        let _encoding = user_config.default_encoding();
        let _tabulation = user_config.default_tabulation();
        let word_wrap = view_snapshot.word_wrap;
        let view = ViewState {
            zoom: view_snapshot.zoom,
            word_wrap,
            show_symbols: view_snapshot.show_symbols,
            show_spaces: view_snapshot.show_spaces,
            show_tabs: view_snapshot.show_tabs,
            show_eol: view_snapshot.show_eol,
            side_panel: view_snapshot.side_panel,
            terminal: view_snapshot.terminal,
            footer_visible: view_snapshot.footer_visible,
            show_memory: view_snapshot.show_memory,
            guide_column: view_snapshot.guide_column,
            margin: view_snapshot.margin,
            border: view_snapshot.border,
            theme: view_snapshot.theme,
        };
        let (workspace, editor, document) =
            workspace_from_config(&user_config, &palette, word_wrap);
        let mut editor = editor;
        let document = document;
        editor.set_tabulation(document.tabulation);
        editor.set_word_wrap(view.word_wrap);

        let mut app = Self {
            editor,
            document,
            theme,
            view,
            recent,
            clipboard: Clipboard::default(),
            menu_bar: MenuBar::build(
                &RecentFiles::default(),
                &ViewState::default(),
                FileEncoding::Utf8,
                Tabulation::Spaces4,
                &Clipboard::default(),
                &workspace,
            ),
            menu_state: MenuState::default(),
            find_pattern: String::new(),
            should_quit: false,
            pending_quit: false,
            status_message: "Pronto".to_string(),
            mouse_enabled,
            modal: Modal::None,
            is_ssh_session: std::env::var("SSH_CONNECTION").is_ok(),
            last_frame_width: 80,
            last_frame_height: 24,
            memory: MemoryMonitor::new(),
            workspace,
            dirty_flow: None,
            pending_dirty_save: None,
            pending_save_all: false,
            pending_open_path: None,
            user_config,
        };
        app.refresh_menu();
        app
    }

    /// Abre arquivos passados na linha de comando (primeiro argumento = aba ativa no topo).
    pub fn open_cli_files(&mut self, files: &[PathBuf]) {
        if files.is_empty() {
            return;
        }

        self.sync_active_tab();
        if self.workspace.tabs.len() == 1 && self.workspace.tabs[0].filepath().is_none() {
            if self.active_tab_is_pristine() {
                let removed = self.workspace.remove_tab_at(0);
                let _ = crate::session::purge_tab(&removed.session_id);
            }
        }

        let mut opened = 0usize;
        for path in files.iter().rev() {
            let path = cli::canonicalize_open_path(path);
            if !path.is_file() {
                self.set_status(format!("Arquivo não encontrado: {}", path.display()));
                continue;
            }
            self.open_path_impl(path);
            opened += 1;
        }

        if opened == 0 && self.workspace.tabs.is_empty() {
            self.create_new_tab_at_top();
        } else if opened > 0 {
            self.set_status(format!("{opened} arquivo(s) aberto(s)"));
        }
        self.refresh_menu();
    }

    pub(crate) fn persist_user_config(&mut self) {
        self.sync_active_tab();
        self.user_config = config_from_view(
            self.recent.paths(),
            &self.view,
            self.document.encoding,
            self.document.tabulation,
            self.build_abas_config(),
        );
        let _ = self.user_config.save();
    }

    fn note_recent_file(&mut self, path: PathBuf) {
        self.recent.push(path);
        self.persist_user_config();
    }

    pub fn refresh_menu(&mut self) {
        self.menu_bar = MenuBar::build(
            &self.recent,
            &self.view,
            self.document.encoding,
            self.document.tabulation,
            &self.clipboard,
            &self.workspace,
        );
    }

    pub fn document_title(&self) -> String {
        let mut title = if self.document.path().is_some() {
            self.document.title()
        } else {
            self.workspace.active_tab().display_name.clone()
        };
        if self.is_dirty() {
            title.push('*');
        }
        title
    }

    pub fn is_dirty(&self) -> bool {
        self.document.is_dirty(&self.editor.content_string())
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            self.memory.refresh_if_due();
            self.refresh_menu();
            terminal.draw(|frame| ui::draw(frame, self))?;

            if events::poll(Duration::from_millis(50))? {
                let event = events::read()?;
                events::dispatch(self, event);
            }
        }

        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.persist_session_artifacts();
        self.persist_user_config();
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn apply_theme(&mut self, theme: ThemeId) {
        self.theme = theme;
        self.view.theme = theme;
        let palette = theme.palette();
        self.editor.apply_theme(&palette);
        self.persist_user_config();
    }

    pub fn request_new_document(&mut self) {
        self.sync_active_tab();
        if self.active_tab_is_pristine() {
            return;
        }
        if let Some(index) = self.workspace.find_first_pristine_untitled() {
            self.focus_tab(index);
            self.set_status("Documento novo");
            return;
        }
        if self.workspace.needs_eviction() {
            let tail = self.workspace.tail_index().expect("tail");
            if self.workspace.tabs[tail].is_dirty() {
                self.begin_dirty_flow(
                    vec![tail],
                    PromptReason::EvictTail,
                    AfterDirtyResolved::EvictThenNew,
                );
                return;
            }
            self.evict_tail_silent();
        }
        self.create_new_tab_at_top();
    }

    pub fn request_open(&mut self) {
        if self.is_dirty() {
            self.modal = Modal::confirm(
                "Abrir arquivo",
                "Existem alterações não salvas. Descartar e abrir outro arquivo?",
                ConfirmKind::DiscardForOpen,
            );
            return;
        }
        self.modal = Modal::path_input("Abrir arquivo", "Caminho:", PathInputKind::Open);
    }

    pub fn request_save(&mut self) {
        if let Some(path) = self.document.path().map(Path::to_path_buf) {
            self.save_to_path(path, false);
        } else {
            self.modal = Modal::path_input("Salvar como", "Caminho:", PathInputKind::SaveAs);
        }
    }

    pub fn request_save_as(&mut self) {
        self.modal = Modal::path_input("Salvar como", "Caminho:", PathInputKind::SaveAs);
    }

    pub fn request_close(&mut self) {
        self.sync_active_tab();
        if self.is_dirty() {
            self.begin_dirty_flow(
                vec![self.workspace.active_index],
                PromptReason::CloseTab,
                AfterDirtyResolved::CloseTab,
            );
            return;
        }
        self.close_active_tab_final();
    }

    pub fn request_close_all(&mut self) {
        self.sync_active_tab();
        let dirty = self.workspace.dirty_indices_menu_order();
        if dirty.is_empty() {
            self.close_all_tabs_final();
            return;
        }
        self.begin_dirty_flow(dirty, PromptReason::CloseAll, AfterDirtyResolved::CloseAll);
    }

    pub fn request_quit(&mut self) {
        self.sync_active_tab();
        let dirty = self.workspace.dirty_indices_menu_order();
        if dirty.is_empty() {
            self.should_quit = true;
            return;
        }
        self.begin_dirty_flow(dirty, PromptReason::Quit, AfterDirtyResolved::Quit);
    }

    fn save_to_path(&mut self, path: PathBuf, confirmed: bool) {
        self.save_to_path_internal(path, confirmed);
    }

    pub(crate) fn save_to_path_internal(&mut self, path: PathBuf, confirmed: bool) {
        let is_current = self.document.path().is_some_and(|current| current == path.as_path());
        if !confirmed && file_io::path_exists(&path) && !is_current {
            self.modal = Modal::confirm(
                "Sobrescrever arquivo",
                format!("O arquivo '{}' já existe. Sobrescrever?", path.display()),
                ConfirmKind::OverwriteSave { path },
            );
            return;
        }

        let lines = self.editor.lines();
        let content = self.editor.content_string();
        match file_io::write_lines_encoded(&path, &lines, self.document.encoding) {
            Ok(()) => {
                self.document.mark_saved(content, path.clone());
                self.note_recent_file(path.clone());
                self.set_status(format!("Salvo: {}", path.display()));
                if self.pending_quit {
                    self.pending_quit = false;
                    self.should_quit = true;
                }
                if self.pending_dirty_save.is_some() || self.dirty_flow.is_some() {
                    if !self.is_dirty() {
                        if let Some((index, reason)) = self.pending_dirty_save.take() {
                            self.handle_tab_unsaved_modal(
                                index,
                                reason,
                                DialogButtonAction::Primary,
                            );
                        } else if self.dirty_flow.is_some() {
                            self.advance_dirty_flow(true);
                        }
                    }
                }
                if self.pending_save_all && !self.is_dirty() {
                    self.save_all_remaining();
                }
            }
            Err(error) => {
                self.set_status(format!("Erro ao salvar: {error}"));
            }
        }
    }

    pub fn open_path(&mut self, path: PathBuf) {
        self.open_path_impl(path);
    }

    pub fn confirm_modal(&mut self) {
        let (kind, action) = match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::Confirm { dialog, kind, .. } => {
                let action = dialog
                    .selected_action()
                    .unwrap_or(DialogButtonAction::Cancel);
                (kind, action)
            }
            other => {
                self.modal = other;
                return;
            }
        };

        match (kind, action) {
            (ConfirmKind::TabUnsaved { tab_index, reason }, action) => {
                self.handle_tab_unsaved_modal(tab_index, reason, action);
            }
            (ConfirmKind::PurgeUndoOnToggle, DialogButtonAction::Primary) => {
                let _ = crate::session::purge_all_undo();
                self.workspace.salvar_desfazer_recentes = false;
                self.persist_user_config();
                self.set_status("Desfazer persistido apagado");
            }
            (ConfirmKind::PurgeUndoOnToggle, DialogButtonAction::Secondary) => {
                self.workspace.salvar_desfazer_recentes = false;
                self.persist_user_config();
                self.set_status("Persistência de desfazer desligada");
            }
            (ConfirmKind::PurgeUndoOnToggle, DialogButtonAction::Cancel) => {
                self.set_status("Toggle mantido");
            }
            (ConfirmKind::QuitUnsaved, DialogButtonAction::Primary) => {
                self.pending_quit = true;
                if self.document.path().is_some() {
                    self.request_save();
                    if !self.is_dirty() {
                        self.pending_quit = false;
                        self.should_quit = true;
                    }
                } else {
                    self.modal =
                        Modal::path_input("Salvar como", "Caminho:", PathInputKind::SaveAs);
                }
            }
            (ConfirmKind::QuitUnsaved, DialogButtonAction::Secondary) => self.should_quit = true,
            (_, DialogButtonAction::Cancel) => self.set_status("Ação cancelada"),
            (ConfirmKind::DiscardForNew, DialogButtonAction::Primary) => self.request_new_document(),
            (ConfirmKind::CloseDocument, DialogButtonAction::Primary) => self.request_close(),
            (ConfirmKind::DiscardForOpen, DialogButtonAction::Primary) => {
                if let Some(path) = self.pending_open_path.take() {
                    self.open_path(path);
                } else {
                    self.modal = Modal::path_input("Abrir arquivo", "Caminho:", PathInputKind::Open);
                }
            }
            (ConfirmKind::OverwriteSave { path }, DialogButtonAction::Primary) => {
                self.save_to_path(path, true)
            }
            (ConfirmKind::ChangeEncoding { encoding }, DialogButtonAction::Primary) => {
                self.apply_encoding_change(encoding)
            }
            (ConfirmKind::ConvertEncoding { encoding }, DialogButtonAction::Primary) => {
                self.convert_encoding(encoding)
            }
            _ => self.set_status("Ação cancelada"),
        }
    }

    pub fn submit_convert_tabulation(&mut self) {
        let Modal::ConvertTabulation(modal) = std::mem::replace(&mut self.modal, Modal::None) else {
            return;
        };
        let from = modal.from_tab();
        let to = modal.to_tab();
        let before = self.editor.content_string();
        let after = convert_tabulation_between(&before, from, to);
        if after != before {
            self.editor.replace_content(&after);
            self.set_status(format!(
                "Convertido: {} → {}",
                from.convert_option_label(),
                to.convert_option_label()
            ));
        } else {
            self.set_status("Nada a converter");
        }
        self.set_tab(to);
    }

    pub fn cancel_modal(&mut self) {
        self.modal = Modal::None;
        self.set_status("Ação cancelada");
    }

    pub fn submit_path_input(&mut self) {
        let Some((path, kind)) = self.take_path_input() else {
            return;
        };

        let path = PathBuf::from(path.trim());
        if path.as_os_str().is_empty() {
            self.set_status("Caminho inválido");
            self.modal = Modal::None;
            return;
        }

        match kind {
            PathInputKind::Open => self.open_path(path),
            PathInputKind::SaveAs => self.save_to_path(path, false),
        }
        self.modal = Modal::None;
    }

    pub fn submit_find(&mut self) {
        let Some((pattern, replacement, replace_mode)) = self.take_find_input() else {
            return;
        };
        self.find_pattern = pattern.clone();
        self.editor.set_search_pattern(&pattern);
        if pattern.is_empty() {
            self.set_status("Padrão vazio");
            self.modal = Modal::None;
            return;
        }
        if replace_mode {
            if self.editor.replace_one(&replacement) {
                self.set_status("Substituído");
            } else {
                self.set_status("Nenhuma ocorrência");
            }
        } else if self.editor.find_next() {
            self.set_status(format!("Busca: {pattern}"));
        } else {
            self.set_status("Não encontrado");
        }
        self.modal = Modal::None;
    }

    pub fn find_next(&mut self) {
        if self.find_pattern.is_empty() {
            self.set_status("Defina um padrão com Ctrl+F");
            return;
        }
        self.editor.set_search_pattern(&self.find_pattern);
        if self.editor.find_next() {
            self.set_status("Próximo");
        } else {
            self.set_status("Fim da busca");
        }
    }

    pub fn find_prev(&mut self) {
        if self.find_pattern.is_empty() {
            return;
        }
        self.editor.set_search_pattern(&self.find_pattern);
        if self.editor.find_prev() {
            self.set_status("Anterior");
        }
    }

    fn take_path_input(&mut self) -> Option<(String, PathInputKind)> {
        match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::PathInput { input, kind, .. } => Some((input, kind)),
            other => {
                self.modal = other;
                None
            }
        }
    }

    fn take_find_input(&mut self) -> Option<(String, String, bool)> {
        match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::Find {
                pattern,
                replacement,
                replace_mode,
                ..
            } => Some((pattern, replacement, replace_mode)),
            other => {
                self.modal = other;
                None
            }
        }
    }

    pub fn toggle_edit_mode(&mut self) {
        let palette = self.theme.palette();
        self.editor.toggle_mode(&palette);
        self.set_status(format!("Modo: {}", self.editor.mode().label()));
    }

    fn apply_encoding_change(&mut self, encoding: FileEncoding) {
        if self.document.path().is_some() {
            self.reinterpret_encoding(encoding);
        } else {
            self.document.encoding = encoding;
            self.set_status(format!("Codificação: {}", encoding.label()));
        }
        self.persist_user_config();
    }

    fn reinterpret_encoding(&mut self, encoding: FileEncoding) {
        if let Some(path) = self.document.path().map(|p| p.to_path_buf()) {
            self.document.encoding = encoding;
            self.open_path(path);
            self.set_status(format!("Reinterpretado como {}", encoding.label()));
        } else {
            self.document.encoding = encoding;
            self.set_status(format!("Codificação: {}", encoding.label()));
        }
    }

    fn convert_encoding(&mut self, encoding: FileEncoding) {
        self.document.encoding = encoding;
        self.set_status(format!("Salvar converterá para {}", encoding.label()));
        self.persist_user_config();
    }

    fn set_tab(&mut self, tab: Tabulation) {
        self.document.tabulation = tab;
        self.editor.set_tabulation(tab);
        self.set_status(format!("Tab: {}", tab.label()));
        self.persist_user_config();
    }

    fn set_encoding(&mut self, encoding: FileEncoding) {
        if encoding == self.document.encoding {
            return;
        }
        let mut message = if self.document.path().is_some() {
            format!(
                "Alterar codificação para {}?\n\n\
                 O arquivo será reaberto com a nova codificação.",
                encoding.label()
            )
        } else {
            format!(
                "Alterar codificação do documento para {}?",
                encoding.label()
            )
        };
        if self.is_dirty() {
            message.push_str("\n\nAlterações não salvas podem ser afetadas.");
        }
        self.modal = Modal::confirm(
            "Codificação",
            message,
            ConfirmKind::ChangeEncoding { encoding },
        );
    }

    pub fn dispatch_action(&mut self, action: ActionId) {
        let persist = is_persistent_setting(action);
        match action {
            ActionId::Quit => self.request_quit(),
            ActionId::New => self.request_new_document(),
            ActionId::Open => self.request_open(),
            ActionId::Save => self.request_save(),
            ActionId::SaveAs => self.request_save_as(),
            ActionId::SaveAll => self.save_all_dirty(),
            ActionId::Close => self.request_close(),
            ActionId::CloseAll => self.request_close_all(),
            ActionId::Undo => {
                self.editor.undo();
                self.set_status("Desfazer");
            }
            ActionId::Redo => {
                self.editor.redo();
                self.set_status("Refazer");
            }
            ActionId::Cut => {
                if self.editor.cut_selection(&mut self.clipboard) {
                    self.set_status("Recortado");
                }
            }
            ActionId::Copy => {
                if self.editor.copy_selection(&mut self.clipboard) {
                    self.set_status("Copiado");
                }
            }
            ActionId::Paste => {
                if let Some(text) = self.clipboard.paste_text() {
                    self.editor.paste(&text);
                    self.set_status("Colado");
                }
            }
            ActionId::PasteClip(i) => {
                if let Some(text) = self.clipboard.get(i).map(str::to_string) {
                    self.editor.paste(&text);
                    self.set_status("Colado do histórico");
                }
            }
            ActionId::SelectAll => self.editor.select_all(),
            ActionId::Find => {
                self.modal = Modal::find("Buscar", &self.find_pattern);
            }
            ActionId::Replace => {
                self.modal = Modal::find_replace("Substituir", &self.find_pattern, "");
            }
            ActionId::ThemeDark => {
                self.apply_theme(ThemeId::Dark);
                self.set_status("Tema: Escuro");
            }
            ActionId::ThemeLight => {
                self.apply_theme(ThemeId::Light);
                self.set_status("Tema: Claro");
            }
            ActionId::ThemeClassicBlue => {
                self.apply_theme(ThemeId::ClassicBlue);
                self.set_status("Tema: Azul Clássico");
            }
            ActionId::ThemeMatrix => {
                self.apply_theme(ThemeId::Matrix);
                self.set_status("Tema Matrix");
            }
            ActionId::ToggleSidePanel => {
                self.view.side_panel = !self.view.side_panel;
                self.set_status(if self.view.side_panel {
                    "Painel: visível (placeholder)"
                } else {
                    "Painel: oculto"
                });
            }
            ActionId::ToggleTerminal => {
                self.view.terminal = !self.view.terminal;
                self.set_status(if self.view.terminal {
                    "Terminal: visível (placeholder)"
                } else {
                    "Terminal: oculto"
                });
            }
            ActionId::ToggleFooter => {
                self.view.footer_visible = !self.view.footer_visible;
            }
            ActionId::ShowMemoryToggle => {
                self.view.show_memory = !self.view.show_memory;
                if self.view.show_memory {
                    self.memory.refresh_if_due();
                }
                self.set_status(if self.view.show_memory {
                    "Consumo de memória: ativado"
                } else {
                    "Consumo de memória: desativado"
                });
            }
            ActionId::ZoomIn => {
                self.view.zoom = self.view.zoom.saturating_add(1).min(3);
                self.set_status(format!("Zoom: {}", self.view.zoom));
            }
            ActionId::ZoomOut => {
                self.view.zoom = self.view.zoom.saturating_sub(1).max(1);
                self.set_status(format!("Zoom: {}", self.view.zoom));
            }
            ActionId::ZoomReset => {
                self.view.zoom = 1;
                self.set_status("Zoom: reset");
            }
            ActionId::WordWrapToggle => {
                self.view.word_wrap = !self.view.word_wrap;
                self.editor.set_word_wrap(self.view.word_wrap);
                self.set_status(if self.view.word_wrap {
                    "Word wrap: on"
                } else {
                    "Word wrap: off"
                });
            }
            ActionId::ShowSymbols => self.view.show_symbols = !self.view.show_symbols,
            ActionId::ShowSpaces => self.view.show_spaces = !self.view.show_spaces,
            ActionId::ShowTabs => self.view.show_tabs = !self.view.show_tabs,
            ActionId::ShowEol => self.view.show_eol = !self.view.show_eol,
            ActionId::ShowAll => {
                let on = !(self.view.show_symbols && self.view.show_spaces);
                self.view.show_all(on);
            }
            ActionId::Column80 => self.view.guide_column = GuideColumn::Col80,
            ActionId::Column120 => self.view.guide_column = GuideColumn::Col120,
            ActionId::Column160 => self.view.guide_column = GuideColumn::Col160,
            ActionId::ColumnUnlimited => self.view.guide_column = GuideColumn::Unlimited,
            ActionId::MarginNone => {
                self.view.margin = EditorMargin::None;
                self.set_status("Margem: sem margem");
            }
            ActionId::MarginOneLine => {
                self.view.margin = EditorMargin::OneLine;
                self.set_status("Margem: uma linha");
            }
            ActionId::MarginTwoLines => {
                self.view.margin = EditorMargin::TwoLines;
                self.set_status("Margem: duas linhas");
            }
            ActionId::BorderToggle => {
                self.view.border = match self.view.border {
                    EditorBorder::Visible => EditorBorder::Hidden,
                    EditorBorder::Hidden => EditorBorder::Visible,
                };
                self.set_status(match self.view.border {
                    EditorBorder::Visible => "Borda: visível",
                    EditorBorder::Hidden => "Borda: invisível",
                });
            }
            ActionId::EncodingUtf8 => self.set_encoding(FileEncoding::Utf8),
            ActionId::EncodingUtf8NoBom => self.set_encoding(FileEncoding::Utf8NoBom),
            ActionId::EncodingUtf16Le => self.set_encoding(FileEncoding::Utf16Le),
            ActionId::EncodingUtf16Be => self.set_encoding(FileEncoding::Utf16Be),
            ActionId::EncodingIso88591 => self.set_encoding(FileEncoding::Iso88591),
            ActionId::EncodingAnsi => self.set_encoding(FileEncoding::Ansi),
            ActionId::TabSpaces2 => self.set_tab(Tabulation::Spaces2),
            ActionId::TabSpaces4 => self.set_tab(Tabulation::Spaces4),
            ActionId::TabSpaces8 => self.set_tab(Tabulation::Spaces8),
            ActionId::TabLiteral => self.set_tab(Tabulation::TabLiteral),
            ActionId::ConvertTabulation => {
                self.modal = Modal::convert_tabulation(self.document.tabulation);
            }
            ActionId::OpenRecent(i) => {
                if let Some(path) = self.recent.paths().get(i).cloned() {
                    self.sync_active_tab();
                    if self.is_dirty() {
                        self.pending_open_path = Some(path);
                        self.modal = Modal::confirm(
                            "Abrir recente",
                            "Descartar alterações e abrir arquivo recente?",
                            ConfirmKind::DiscardForOpen,
                        );
                    } else {
                        self.open_path(path);
                    }
                }
            }
            ActionId::FocusTab(i) => self.focus_tab(i),
            ActionId::ToggleCloseAllOnExit => {
                self.workspace.fechar_tudo_ao_sair = !self.workspace.fechar_tudo_ao_sair;
                self.persist_user_config();
                self.set_status(if self.workspace.fechar_tudo_ao_sair {
                    "Fechar tudo ao sair: ligado"
                } else {
                    "Fechar tudo ao sair: desligado"
                });
            }
            ActionId::TogglePersistUndo => self.toggle_persist_undo(),
            ActionId::SortFileName => {
                self.sync_active_tab();
                self.workspace.sort_tabs(TabSortStrategy::FileName);
                self.focus_tab(self.workspace.active_index);
            }
            ActionId::SortFilePath => {
                self.sync_active_tab();
                self.workspace.sort_tabs(TabSortStrategy::FilePath);
                self.focus_tab(self.workspace.active_index);
            }
            ActionId::SortOpenedFirst => {
                self.sync_active_tab();
                self.workspace.sort_tabs(TabSortStrategy::OpenedFirst);
                self.focus_tab(self.workspace.active_index);
            }
            ActionId::SortOpenedLast => {
                self.sync_active_tab();
                self.workspace.sort_tabs(TabSortStrategy::OpenedLast);
                self.focus_tab(self.workspace.active_index);
            }
            ActionId::SortStatus => {
                self.sync_active_tab();
                self.workspace.sort_tabs(TabSortStrategy::Status);
                self.focus_tab(self.workspace.active_index);
            }
            ActionId::Recent | ActionId::PastePrevious | ActionId::NoOp => {}
        }
        if persist {
            self.persist_user_config();
        }
    }

    fn toggle_persist_undo(&mut self) {
        if self.workspace.salvar_desfazer_recentes {
            if has_any_undo_files().unwrap_or(false) {
                self.modal = Modal::purge_undo_toggle();
                return;
            }
            self.workspace.salvar_desfazer_recentes = false;
            self.persist_user_config();
            self.set_status("Salvar desfazer recentes: desligado");
        } else {
            self.workspace.salvar_desfazer_recentes = true;
            self.persist_user_config();
            self.set_status("Salvar desfazer recentes: ligado");
        }
    }
}

fn is_persistent_setting(action: ActionId) -> bool {
    matches!(
        action,
        ActionId::ToggleCloseAllOnExit
            | ActionId::TogglePersistUndo
            | ActionId::ToggleSidePanel
            | ActionId::ToggleTerminal
            | ActionId::ToggleFooter
            | ActionId::ShowMemoryToggle
            | ActionId::ZoomIn
            | ActionId::ZoomOut
            | ActionId::ZoomReset
            | ActionId::WordWrapToggle
            | ActionId::ShowSymbols
            | ActionId::ShowSpaces
            | ActionId::ShowTabs
            | ActionId::ShowEol
            | ActionId::ShowAll
            | ActionId::Column80
            | ActionId::Column120
            | ActionId::Column160
            | ActionId::ColumnUnlimited
            | ActionId::MarginNone
            | ActionId::MarginOneLine
            | ActionId::MarginTwoLines
            | ActionId::BorderToggle
    )
}
