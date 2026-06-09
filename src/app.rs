use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::cli;
use crate::clipboard::Clipboard;
use crate::app_workspace::{
    workspace_from_config, AfterDirtyResolved, DirtyFlow, PendingFsCheck, WorkspaceBootstrap,
};
use crate::session::{purge_all, purge_orphans};
use crate::config::{config_from_view, EditConfig};
use crate::editor_split::EditorSplit;
use crate::document::Document;
use crate::editor::Editor;
use crate::encoding::{FileEncoding, Tabulation};
use crate::editor::convert_tabulation_between;
use crate::events;
use crate::file_io;
use crate::memory::MemoryMonitor;
use crate::menus::{ActionId, MenuBar, MenuState};
use crate::modal::{
    ConfirmKind, DialogButtonAction, FileBrowserMode, FindReplaceCommand, GoToLineCommand,
    HelpKind, Modal, PathInputKind,
};
use crate::recent::RecentFiles;
use crate::theme::ThemeId;
use crate::ui;
use crate::session::has_any_undo_files;
use crate::terminal::TerminalWorkspace;
use crate::view_state::{EditorBorder, EditorMargin, GuideColumn, InputFocus, ViewState};
use crate::workspace::{PromptReason, TabSortStrategy, Workspace};

pub struct App {
    pub editor: Editor,
    pub document: Document,
    pub theme: ThemeId,
    pub view: ViewState,
    pub input_focus: InputFocus,
    pub recent: RecentFiles,
    pub clipboard: Clipboard,
    pub menu_bar: MenuBar,
    pub menu_state: MenuState,
    pub find_pattern: String,
    pub should_quit: bool,
    pub pending_quit: bool,
    pub status_message: String,
    /// Ajuda temporária ao passar o mouse sobre um grupo do rodapé.
    pub footer_hover_help: Option<String>,
    pub mouse_enabled: bool,
    pub modal: Modal,
    pub is_ssh_session: bool,
    pub last_frame_width: u16,
    pub last_frame_height: u16,
    pub memory: MemoryMonitor,
    pub workspace: Workspace,
    pub editor_split: EditorSplit,
    pub terminal: TerminalWorkspace,
    pub dirty_flow: Option<DirtyFlow>,
    pub pending_dirty_save: Option<(usize, PromptReason)>,
    pub pending_save_all: bool,
    pub pending_open_path: Option<PathBuf>,
    pub(crate) pending_fs_checks: Vec<PendingFsCheck>,
    pub(crate) pending_focus_tab: Option<usize>,
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
            word_wrap,
            show_symbols: view_snapshot.show_symbols,
            show_spaces: view_snapshot.show_spaces,
            show_tabs: view_snapshot.show_tabs,
            show_eol: view_snapshot.show_eol,
            terminal: view_snapshot.terminal,
            terminal_panel_rows: view_snapshot.terminal_panel_rows,
            terminal_color_scheme: view_snapshot.terminal_color_scheme,
            footer_visible: view_snapshot.footer_visible,
            show_memory: view_snapshot.show_memory,
            show_line_numbers: view_snapshot.show_line_numbers,
            guide_column: view_snapshot.guide_column,
            margin: view_snapshot.margin,
            border: view_snapshot.border,
            theme: view_snapshot.theme,
        };
        let abas = &user_config.arquivo.abas;
        if abas.fechar_tudo_ao_sair {
            let _ = purge_all();
        } else {
            let ids: Vec<String> = abas.sessao.iter().map(|e| e.tab_id.clone()).collect();
            if !ids.is_empty() {
                let _ = purge_orphans(&ids);
            }
        }
        let WorkspaceBootstrap {
            workspace,
            editor,
            document,
            pending_fs_checks,
        } = workspace_from_config(&user_config, &palette, word_wrap);
        let mut editor_split = user_config.editor_split();
        if editor_split.is_active() && !editor_split.can_activate(workspace.tabs.len()) {
            editor_split = EditorSplit::default();
        }
        let mut editor = editor;
        let document = document;
        editor.set_tabulation(document.tabulation);
        editor.set_word_wrap(view.word_wrap);

        let mut app = Self {
            editor,
            document,
            theme,
            view,
            input_focus: InputFocus::Editor,
            recent,
            clipboard: Clipboard::default(),
            menu_bar: MenuBar::build(
                &RecentFiles::default(),
                &ViewState::default(),
                FileEncoding::Utf8,
                Tabulation::Spaces4,
                &Clipboard::default(),
                &workspace,
                editor_split.is_active(),
            ),
            menu_state: MenuState::default(),
            find_pattern: String::new(),
            should_quit: false,
            pending_quit: false,
            status_message: "Pronto".to_string(),
            footer_hover_help: None,
            mouse_enabled,
            modal: Modal::None,
            is_ssh_session: std::env::var("SSH_CONNECTION").is_ok(),
            last_frame_width: 80,
            last_frame_height: 24,
            memory: MemoryMonitor::new(),
            workspace,
            editor_split,
            terminal: TerminalWorkspace::default(),
            dirty_flow: None,
            pending_dirty_save: None,
            pending_save_all: false,
            pending_open_path: None,
            pending_fs_checks,
            pending_focus_tab: None,
            user_config,
        };
        app.refresh_menu();
        if app.view.terminal {
            app.ensure_terminal_session();
        }
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
        let ultimo = self.user_config.arquivo.ultimo_diretorio.clone();
        let ocultos = self.user_config.arquivo.mostrar_ocultos;
        let filtro = self.user_config.arquivo.filtro_abrir.clone();
        self.user_config = config_from_view(
            self.recent.paths(),
            &self.view,
            self.document.encoding,
            self.document.tabulation,
            self.build_abas_config(),
            &self.editor_split,
        );
        self.user_config.arquivo.ultimo_diretorio = ultimo;
        self.user_config.arquivo.mostrar_ocultos = ocultos;
        self.user_config.arquivo.filtro_abrir = filtro;
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
            self.split_active(),
        );
    }

    pub fn document_title(&self) -> String {
        self.tab_pane_title(Some(self.workspace.active_index))
    }

    pub fn tab_pane_title(&self, tab_index: Option<usize>) -> String {
        let Some(index) = tab_index else {
            return "Selecione aba…".to_string();
        };
        if index >= self.workspace.tabs.len() {
            return "Selecione aba…".to_string();
        }
        let tab = &self.workspace.tabs[index];
        let mut title = if tab.document.path().is_some() {
            tab.document.title()
        } else {
            tab.display_name.clone()
        };
        if tab.is_dirty() {
            title.push('*');
        }
        title
    }

    pub fn is_dirty(&self) -> bool {
        self.document.is_dirty(&self.editor.content_string())
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            self.process_pending_fs_checks();
            self.memory.refresh_if_due();
            self.terminal.drain_all();
            self.process_terminal_session_exits();
            self.sync_terminal_pty_size();
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
        self.terminal.shutdown();
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
        self.refresh_menu();
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
        self.open_file_browser(FileBrowserMode::Open);
    }

    pub fn request_save(&mut self) {
        if let Some(path) = self.document.path().map(Path::to_path_buf) {
            self.save_to_path(path, false);
        } else {
            self.open_file_browser(FileBrowserMode::SaveAs);
        }
    }

    pub fn request_save_as(&mut self) {
        self.open_file_browser(FileBrowserMode::SaveAs);
    }

    pub(crate) fn open_file_browser(&mut self, mode: FileBrowserMode) {
        use crate::modal::file_browser::{
            infer_filter_from_path, initial_directory, suggest_file_name,
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let last = self
            .user_config
            .arquivo
            .ultimo_diretorio
            .as_deref()
            .map(Path::new);
        let dir = initial_directory(self.document.path(), last, &cwd);
        let name = suggest_file_name(self.document.path(), "Novo.txt");
        let filter = match mode {
            FileBrowserMode::Open => {
                if self.user_config.arquivo.filtro_abrir.is_empty() {
                    infer_filter_from_path(self.document.path())
                } else {
                    self.user_config.arquivo.filtro_abrir.clone()
                }
            }
            FileBrowserMode::Save | FileBrowserMode::SaveAs => {
                infer_filter_from_path(Some(Path::new(&name)))
            }
        };
        let show_hidden = self.user_config.arquivo.mostrar_ocultos;
        self.modal = Modal::file_browser(mode, dir, name, filter, show_hidden);
    }

    pub fn open_help_features(&mut self) {
        self.modal = Modal::help(HelpKind::Features);
    }

    pub fn request_rename(&mut self) {
        let Some(path) = self.document.path().map(Path::to_path_buf) else {
            self.set_status("Salve o documento antes de renomear");
            return;
        };
        let default = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        self.modal = Modal::path_input("Renomear", "Novo nome:", PathInputKind::Rename, default);
    }

    pub fn toggle_terminal_panel(&mut self) {
        self.view.terminal = !self.view.terminal;
        if !self.view.terminal {
            self.input_focus = InputFocus::Editor;
            self.terminal.sidebar_hover = None;
        } else {
            self.ensure_terminal_session();
        }
        self.persist_user_config();
        self.set_status(if self.view.terminal {
            "Terminal: visível"
        } else {
            "Terminal: oculto"
        });
    }

    /// `Ctrl+E` — foco no editor, mesmo com terminal visível.
    pub fn focus_editor(&mut self) {
        self.input_focus = InputFocus::Editor;
        self.set_status("Editor: foco");
    }

    /// `Ctrl+T` / `Ctrl+'` — do editor: abre ou foca terminal; do terminal: fecha painel.
    pub fn chord_terminal_toggle(&mut self) {
        if self.view.terminal && self.input_focus == InputFocus::Terminal {
            self.view.terminal = false;
            self.input_focus = InputFocus::Editor;
            self.terminal.sidebar_hover = None;
            self.persist_user_config();
            self.set_status("Terminal: oculto");
            return;
        }
        if !self.view.terminal {
            self.view.terminal = true;
            self.persist_user_config();
        }
        self.ensure_terminal_session();
        self.sync_terminal_pty_size();
        self.input_focus = InputFocus::Terminal;
        self.set_status("Terminal: foco");
    }

    pub fn request_go_to_line(&mut self) {
        let (ln, col) = self.editor.cursor_line_col();
        self.modal = Modal::go_to_line(ln, col);
    }

    pub fn grow_terminal_panel(&mut self) {
        use crate::terminal::{clamp_terminal_panel_rows, TERMINAL_PANEL_ROWS_MAX};
        let next = self.view.terminal_panel_rows.saturating_add(1);
        if next > TERMINAL_PANEL_ROWS_MAX {
            self.set_status("Terminal: altura máxima");
            return;
        }
        self.view.terminal_panel_rows = clamp_terminal_panel_rows(next);
        self.persist_user_config();
        self.sync_terminal_pty_size();
        self.set_status(format!(
            "Terminal: {} linhas",
            self.view.terminal_panel_rows
        ));
    }

    pub fn shrink_terminal_panel(&mut self) {
        use crate::terminal::{clamp_terminal_panel_rows, TERMINAL_PANEL_ROWS_MIN};
        if self.view.terminal_panel_rows <= TERMINAL_PANEL_ROWS_MIN {
            self.set_status("Terminal: altura mínima");
            return;
        }
        self.view.terminal_panel_rows =
            clamp_terminal_panel_rows(self.view.terminal_panel_rows.saturating_sub(1));
        self.persist_user_config();
        self.sync_terminal_pty_size();
        self.set_status(format!(
            "Terminal: {} linhas",
            self.view.terminal_panel_rows
        ));
    }

    pub fn toggle_terminal_color_scheme(&mut self) {
        self.view.terminal_color_scheme = self.view.terminal_color_scheme.toggle();
        self.persist_user_config();
        self.set_status(format!(
            "Terminal: {}",
            self.view.terminal_color_scheme.status_label()
        ));
    }

    /// Envia seleção do editor (ou linha atual) ao PTY ativo e foca o terminal.
    pub fn send_editor_text_to_terminal(&mut self) {
        let text = self.editor.text_for_terminal_insert();
        if !self.view.terminal {
            self.view.terminal = true;
            self.persist_user_config();
        }
        self.ensure_terminal_session();
        self.sync_terminal_pty_size();
        if !text.is_empty() {
            self.terminal.write_active(text.as_bytes());
        }
        self.input_focus = InputFocus::Terminal;
        self.set_status(if text.is_empty() {
            "Terminal: foco".to_string()
        } else {
            format!("Terminal: {} caractere(s) enviado(s)", text.chars().count())
        });
    }

    pub fn toggle_input_focus(&mut self) {
        if !self.view.terminal {
            self.view.terminal = true;
            self.ensure_terminal_session();
            self.persist_user_config();
            self.input_focus = InputFocus::Terminal;
            self.set_status("Terminal: foco");
            return;
        }
        self.input_focus = match self.input_focus {
            InputFocus::Editor => InputFocus::Terminal,
            InputFocus::Terminal => InputFocus::Editor,
        };
        self.set_status(match self.input_focus {
            InputFocus::Editor => "Editor: foco",
            InputFocus::Terminal => "Terminal: foco",
        });
    }

    fn terminal_pty_size(&self) -> (u16, u16) {
        let layout = ui::UiLayout::compute(
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: self.last_frame_width,
                height: self.last_frame_height,
            },
            self,
        );
        layout
            .terminal
            .map(|panel| (panel.output.width.max(1), panel.output.height.max(1)))
            .unwrap_or((80, 24))
    }

    pub(crate) fn sync_terminal_pty_size(&mut self) {
        if !self.view.terminal {
            return;
        }
        let (cols, rows) = self.terminal_pty_size();
        self.terminal.resize_active(cols, rows);
    }

    pub(crate) fn ensure_terminal_session(&mut self) {
        if !self.view.terminal {
            return;
        }
        let cwd = crate::terminal::default_spawn_cwd(self.document.path());
        let (cols, rows) = self.terminal_pty_size();
        self.terminal.ensure_session(cwd, cols, rows);
        self.sync_terminal_pty_size();
    }

    /// Fecha sessões cujo shell terminou (`exit` / `quit`). Se era a última, oculta o painel.
    pub(crate) fn process_terminal_session_exits(&mut self) {
        let exited = self.terminal.exited_session_indices();
        if exited.is_empty() {
            return;
        }
        for index in exited {
            self.terminal.close_session(index);
        }
        self.terminal.clear_selection();
        if self.terminal.sessions.is_empty() {
            self.view.terminal = false;
            self.terminal.sidebar_hover = None;
            if self.input_focus == InputFocus::Terminal {
                self.input_focus = InputFocus::Editor;
            }
            self.set_status("Terminal: sessão encerrada");
        } else {
            self.sync_terminal_pty_size();
            self.set_status("Terminal: sessão encerrada");
        }
    }

    pub(crate) fn rename_file_to(&mut self, name_input: &str) {
        let Some(old) = self.document.path().map(Path::to_path_buf) else {
            self.set_status("Nenhum arquivo para renomear");
            return;
        };
        let trimmed = name_input.trim();
        if trimmed.is_empty() {
            self.set_status("Nome inválido");
            return;
        }
        let new_path = if Path::new(trimmed).is_absolute() {
            PathBuf::from(trimmed)
        } else if let Some(parent) = old.parent() {
            parent.join(trimmed)
        } else {
            PathBuf::from(trimmed)
        };
        if file_io::same_file_path(&old, &new_path) {
            self.set_status("Nome inalterado");
            return;
        }
        if new_path.exists() {
            self.set_status("Arquivo destino já existe");
            return;
        }
        match std::fs::rename(&old, &new_path) {
            Ok(()) => {
                self.sync_active_tab();
                let normalized = file_io::normalize_open_path(&new_path);
                let content = self.editor.content_string();
                self.document.mark_saved(content, normalized.clone());
                let idx = self.workspace.active_index;
                if let Some(tab) = self.workspace.tabs.get_mut(idx) {
                    tab.document = self.document.clone();
                    tab.display_name = normalized
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();
                    tab.fs_snapshot =
                        crate::workspace::snapshot_path(&normalized).ok();
                }
                self.recent.remove_path(&old);
                self.recent.push(normalized.clone());
                self.persist_user_config();
                self.refresh_menu();
                self.set_status(format!("Renomeado: {}", normalized.display()));
            }
            Err(error) => {
                self.set_status(format!("Erro ao renomear: {error}"));
            }
        }
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
        let is_current = self
            .document
            .path()
            .is_some_and(|current| crate::file_io::same_file_path(current, path.as_path()));
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
            (ConfirmKind::ReloadExternal { tab_index }, action) => {
                let from_focus = self.pending_focus_tab == Some(tab_index);
                self.handle_reload_external(tab_index, action, from_focus);
            }
            (ConfirmKind::FileMissing { tab_index }, action) => {
                let from_focus = self.pending_focus_tab == Some(tab_index);
                self.handle_file_missing(tab_index, action, from_focus);
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
                    self.open_file_browser(FileBrowserMode::SaveAs);
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
                    self.open_file_browser(FileBrowserMode::Open);
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

    pub fn submit_file_browser(&mut self) {
        use crate::modal::file_browser::FileEntryKind;

        let Modal::FileBrowser(modal) = &mut self.modal else {
            return;
        };

        if modal.mode == FileBrowserMode::Open {
            if let Some(entry) = modal.entries.get(modal.list_cursor).cloned() {
                if matches!(entry.kind, FileEntryKind::Parent | FileEntryKind::Dir) {
                    modal.enter_directory(entry.path);
                    return;
                }
            }
        }

        let mode = modal.mode;
        let path = modal.resolved_path();

        let has_file_selection = modal
            .entries
            .get(modal.list_cursor)
            .is_some_and(|e| e.kind == FileEntryKind::File);

        if modal.name_input.text().trim().is_empty() && !has_file_selection {
            self.set_status("Informe um nome de arquivo");
            return;
        }

        if mode == FileBrowserMode::Open && path.is_dir() {
            modal.enter_directory(path);
            return;
        }

        if mode == FileBrowserMode::Open && !path.is_file() {
            self.set_status("Arquivo não encontrado");
            return;
        }

        let show_hidden = modal.show_hidden;
        let filter = modal.filter_input.text().to_string();
        let dir = modal.current_dir.clone();
        self.modal = Modal::None;

        self.user_config.arquivo.ultimo_diretorio = Some(dir.display().to_string());
        self.user_config.arquivo.mostrar_ocultos = show_hidden;
        if mode == FileBrowserMode::Open {
            self.user_config.arquivo.filtro_abrir = filter;
        }

        match mode {
            FileBrowserMode::Open => {
                self.open_path(file_io::normalize_open_path(&path));
            }
            FileBrowserMode::Save | FileBrowserMode::SaveAs => {
                self.save_to_path(path, false);
            }
        }
    }

    pub fn submit_path_input(&mut self) {
        let Some((path_str, kind)) = self.take_path_input() else {
            return;
        };

        if matches!(kind, PathInputKind::Rename) {
            self.rename_file_to(&path_str);
            self.modal = Modal::None;
            return;
        }

        let path = PathBuf::from(path_str.trim());
        if path.as_os_str().is_empty() {
            self.set_status("Caminho inválido");
            self.modal = Modal::None;
            return;
        }

        match kind {
            PathInputKind::Open => self.open_path(path),
            PathInputKind::SaveAs => self.save_to_path(path, false),
            PathInputKind::Rename => {}
        }
        self.modal = Modal::None;
    }

    pub fn has_active_search(&self) -> bool {
        !self.find_pattern.is_empty() || !self.editor.search_pattern().is_empty()
    }

    pub fn clear_search(&mut self) {
        self.find_pattern.clear();
        self.editor.set_search_pattern("");
        if let Modal::Find(modal) = &mut self.modal {
            modal.pattern.clear();
            modal.replacement.clear();
            modal.focus = crate::modal::find_replace::FindReplaceFocus::Pattern;
        }
        self.set_status("Busca limpa");
    }

    pub fn apply_find_command(&mut self, cmd: FindReplaceCommand) {
        if matches!(cmd, FindReplaceCommand::Clear) {
            self.clear_search();
            return;
        }
        if matches!(cmd, FindReplaceCommand::Close) {
            self.modal = Modal::None;
            return;
        }

        let Modal::Find(modal) = &self.modal else {
            return;
        };
        let pattern = modal.pattern.text().to_string();
        let replacement = modal.replacement.text().to_string();

        if pattern.is_empty() {
            self.set_status("Padrão vazio");
            return;
        }

        self.find_pattern = pattern;
        self.editor.set_search_pattern(&self.find_pattern);

        let status = match cmd {
            FindReplaceCommand::FindNext => {
                if self.editor.find_next() {
                    "Próximo"
                } else {
                    "Fim da busca"
                }
            }
            FindReplaceCommand::FindPrev => {
                if self.editor.find_prev() {
                    "Anterior"
                } else {
                    "Início da busca"
                }
            }
            FindReplaceCommand::FindFirst => {
                if self.editor.find_first() {
                    "Primeiro"
                } else {
                    "Não encontrado"
                }
            }
            FindReplaceCommand::FindLast => {
                if self.editor.find_last() {
                    "Último"
                } else {
                    "Não encontrado"
                }
            }
            FindReplaceCommand::ReplaceNext => {
                if self.editor.find_next() && self.editor.replace_one(&replacement) {
                    "Substituído"
                } else {
                    "Nenhuma ocorrência"
                }
            }
            FindReplaceCommand::ReplaceAll => {
                let count = self.editor.replace_all(&replacement);
                if count > 0 {
                    return self.set_status(format!("{count} substituída(s)"));
                }
                "Nenhuma ocorrência"
            }
            FindReplaceCommand::Clear | FindReplaceCommand::Close => unreachable!(),
        };
        self.set_status(status);
    }

    pub fn apply_go_to_line(&mut self) {
        let Modal::GoToLine(modal) = &self.modal else {
            return;
        };
        let line_text = modal.line.text().to_string();
        let col_text = modal.col.text().to_string();
        let line_num = line_text.trim().parse::<usize>().unwrap_or(0);
        if line_num == 0 {
            self.set_status("Número de linha inválido");
            return;
        }
        let line = line_num.saturating_sub(1);
        let col = if col_text.trim().is_empty() {
            let (_, c) = self.editor.cursor_line_col();
            c.saturating_sub(1)
        } else {
            col_text
                .trim()
                .parse::<usize>()
                .unwrap_or(1)
                .saturating_sub(1)
        };
        self.editor.set_cursor(line, col);
        self.modal = Modal::None;
        self.set_status(format!("Linha {line_num}"));
    }

    pub fn apply_go_to_line_command(&mut self, cmd: GoToLineCommand) {
        match cmd {
            GoToLineCommand::Go => self.apply_go_to_line(),
            GoToLineCommand::Close => self.modal = Modal::None,
        }
    }

    pub fn request_find_next(&mut self) {
        if self.find_pattern.is_empty() {
            self.modal = Modal::find("Buscar", &self.find_pattern);
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
            Modal::PathInput { input, kind, .. } => Some((input.text().to_string(), kind)),
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
            ActionId::Rename => self.request_rename(),
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
            ActionId::GoToLine => self.request_go_to_line(),
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
            ActionId::ToggleTerminal => self.toggle_terminal_panel(),
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
            ActionId::LineNumbersToggle => {
                self.view.show_line_numbers = !self.view.show_line_numbers;
                self.set_status(if self.view.show_line_numbers {
                    "Números de linha: ativados"
                } else {
                    "Números de linha: desativados"
                });
            }
            ActionId::WordWrapToggle => {
                self.view.word_wrap = !self.view.word_wrap;
                self.editor.set_word_wrap(self.view.word_wrap);
                self.set_status(if self.view.word_wrap {
                    "Quebra de linha: ativada"
                } else {
                    "Quebra de linha: desativada"
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
            ActionId::ToggleSplitEditor => self.toggle_editor_split(),
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
                    "Fechar tudo ao sair: ligado (não restaura abas)"
                } else {
                    "Fechar tudo ao sair: desligado (mantém abas)"
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
            ActionId::HelpFeatures => self.modal = Modal::help(HelpKind::Features),
            ActionId::HelpShortcuts => self.modal = Modal::help(HelpKind::Shortcuts),
            ActionId::HelpAbout => self.modal = Modal::help(HelpKind::About),
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
            | ActionId::ToggleTerminal
            | ActionId::ToggleFooter
            | ActionId::ShowMemoryToggle
            | ActionId::LineNumbersToggle
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
            | ActionId::ToggleSplitEditor
    )
}
