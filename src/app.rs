use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::clipboard::Clipboard;
use crate::document::Document;
use crate::editor::Editor;
use crate::encoding::{FileEncoding, Tabulation};
use crate::events;
use crate::file_io;
use crate::menus::{ActionId, MenuBar, MenuState};
use crate::modal::{ConfirmKind, ConfirmLayout, Modal, PathInputKind};
use crate::recent::RecentFiles;
use crate::theme::ThemeId;
use crate::ui;
use crate::view_state::{EditorBorder, EditorMargin, GuideColumn, ViewState};

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
}

impl App {
    pub fn new(mouse_enabled: bool) -> Self {
        let theme = ThemeId::ClassicBlue;
        let palette = theme.palette();
        let recent = RecentFiles::load();
        let view = ViewState {
            theme,
            ..ViewState::default()
        };
        let mut app = Self {
            editor: Editor::new(&palette),
            document: Document::new(),
            theme,
            view,
            recent,
            clipboard: Clipboard::default(),
            menu_bar: MenuBar::build(&RecentFiles::default(), &ViewState::default(), FileEncoding::Utf8, Tabulation::Spaces4, &Clipboard::default()),
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
        };
        app.refresh_menu();
        app
    }

    pub fn refresh_menu(&mut self) {
        self.menu_bar = MenuBar::build(
            &self.recent,
            &self.view,
            self.document.encoding,
            self.document.tabulation,
            &self.clipboard,
        );
    }

    pub fn document_title(&self) -> String {
        let mut title = self.document.title();
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
            self.refresh_menu();
            terminal.draw(|frame| ui::draw(frame, self))?;

            if events::poll(Duration::from_millis(50))? {
                let event = events::read()?;
                events::dispatch(self, event);
            }
        }

        Ok(())
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn apply_theme(&mut self, theme: ThemeId) {
        self.theme = theme;
        self.view.theme = theme;
        let palette = theme.palette();
        self.editor.apply_theme(&palette);
    }

    pub fn request_new_document(&mut self) {
        if self.is_dirty() {
            self.modal = Modal::confirm(
                "Novo documento",
                "Existem alterações não salvas. Descartar e criar novo documento?",
                ConfirmKind::DiscardForNew,
            );
            return;
        }
        self.new_document();
    }

    pub fn new_document(&mut self) {
        self.editor.clear();
        self.document.reset();
        self.editor.set_tabulation(self.document.tabulation);
        self.set_status("Novo documento");
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
        if self.is_dirty() {
            self.modal = Modal::confirm(
                "Fechar documento",
                "Existem alterações não salvas. Descartar alterações?",
                ConfirmKind::CloseDocument,
            );
            return;
        }
        self.new_document();
    }

    pub fn request_quit(&mut self) {
        if self.is_dirty() {
            self.modal = Modal::quit_unsaved(&self.document_title());
            return;
        }
        self.should_quit = true;
    }

    fn save_to_path(&mut self, path: PathBuf, confirmed: bool) {
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
                self.recent.push(path.clone());
                self.set_status(format!("Salvo: {}", path.display()));
                if self.pending_quit {
                    self.pending_quit = false;
                    self.should_quit = true;
                }
            }
            Err(error) => {
                self.set_status(format!("Erro ao salvar: {error}"));
            }
        }
    }

    pub fn open_path(&mut self, path: PathBuf) {
        match file_io::read_lines_encoded(&path, self.document.encoding) {
            Ok(lines) => {
                self.editor.set_lines(lines);
                self.document
                    .set_opened(self.editor.content_string(), path.clone());
                self.recent.push(path.clone());
                self.editor.set_tabulation(self.document.tabulation);
                self.set_status(format!("Aberto: {}", path.display()));
            }
            Err(error) => {
                self.set_status(format!("Erro ao abrir: {error}"));
            }
        }
    }

    pub fn confirm_modal(&mut self) {
        let (kind, selected, layout) = match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::Confirm {
                kind,
                selected,
                layout,
                ..
            } => (kind, selected, layout),
            other => {
                self.modal = other;
                return;
            }
        };

        match kind {
            ConfirmKind::QuitUnsaved => match selected {
                0 => {
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
                1 => self.should_quit = true,
                _ => self.set_status("Ação cancelada"),
            },
            _ if layout == ConfirmLayout::OkCancel && selected == 1 => {
                self.set_status("Ação cancelada");
            }
            ConfirmKind::DiscardForNew | ConfirmKind::CloseDocument => self.new_document(),
            ConfirmKind::DiscardForOpen => {
                self.modal = Modal::path_input("Abrir arquivo", "Caminho:", PathInputKind::Open);
            }
            ConfirmKind::OverwriteSave { path } => self.save_to_path(path, true),
            ConfirmKind::ReinterpretEncoding { encoding } => self.reinterpret_encoding(encoding),
            ConfirmKind::ConvertEncoding { encoding } => self.convert_encoding(encoding),
        }
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
    }

    pub fn dispatch_action(&mut self, action: ActionId) {
        match action {
            ActionId::Quit => self.request_quit(),
            ActionId::New => self.request_new_document(),
            ActionId::Open => self.request_open(),
            ActionId::Save => self.request_save(),
            ActionId::SaveAs => self.request_save_as(),
            ActionId::Close => self.request_close(),
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
            ActionId::WordWrapOn => {
                self.view.word_wrap = true;
                self.editor.set_word_wrap(true);
                self.set_status("Word wrap: on");
            }
            ActionId::WordWrapOff => {
                self.view.word_wrap = false;
                self.editor.set_word_wrap(false);
                self.set_status("Word wrap: off");
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
            ActionId::BorderVisible => {
                self.view.border = EditorBorder::Visible;
                self.set_status("Borda: visível");
            }
            ActionId::BorderHidden => {
                self.view.border = EditorBorder::Hidden;
                self.set_status("Borda: invisível");
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
            ActionId::OpenRecent(i) => {
                if let Some(path) = self.recent.paths().get(i).cloned() {
                    if self.is_dirty() {
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
            ActionId::Recent | ActionId::PastePrevious | ActionId::NoOp => {}
        }
    }

    fn set_encoding(&mut self, encoding: FileEncoding) {
        if self.document.path().is_some() && self.is_dirty() {
            self.modal = Modal::confirm(
                "Codificação",
                format!(
                    "Reinterpretar arquivo como {}? Alterações não salvas podem ser afetadas.",
                    encoding.label()
                ),
                ConfirmKind::ReinterpretEncoding { encoding },
            );
        } else if self.document.path().is_some() {
            self.reinterpret_encoding(encoding);
        } else {
            self.document.encoding = encoding;
            self.set_status(format!("Codificação: {}", encoding.label()));
        }
    }

    fn set_tab(&mut self, tab: Tabulation) {
        self.document.tabulation = tab;
        self.editor.set_tabulation(tab);
        self.set_status(format!("Tab: {}", tab.label()));
    }
}
