mod buttons;
pub mod convert_tab;
pub mod dialog;
pub mod file_browser;
pub mod find_replace;
pub mod form_controls;
pub mod go_to_line;
pub mod help;
pub mod help_content;
pub mod text_input;

use std::path::PathBuf;

pub use convert_tab::ConvertTabulationModal;
pub use dialog::{Dialog, DialogButton, DialogButtonAction, DialogKeyResult};
pub use file_browser::{FileBrowserModal, FileBrowserMode};
pub use find_replace::{FindReplaceCommand, FindReplaceModal, FindReplaceKeyResult};
pub use go_to_line::{GoToLineCommand, GoToLineModal, GoToLineKeyResult};
pub use text_input::{CharAccept, TextInput};
pub use help::{HelpKind, HelpModal};

use crate::encoding::FileEncoding;
use crate::workspace::PromptReason;

use buttons::{
    CONVERT, DISCARD_CLOSE, DISCARD_NEW, DISCARD_OPEN,
    OVERWRITE,
    FILE_MISSING, PATH_OPEN, PATH_RENAME, PATH_SAVE_AS, PURGE_UNDO, QUIT_UNSAVED, REINTERPRET,
    RELOAD_EXTERNAL, TAB_UNSAVED,
};

/// Intenção de domínio associada a um diálogo de confirmação.
#[derive(Debug, Clone)]
pub enum ConfirmKind {
    QuitUnsaved,
    DiscardForNew,
    DiscardForOpen,
    CloseDocument,
    OverwriteSave { path: PathBuf },
    ChangeEncoding { encoding: FileEncoding },
    ConvertEncoding { encoding: FileEncoding },
    TabUnsaved {
        tab_index: usize,
        reason: PromptReason,
    },
    PurgeUndoOnToggle,
    ReloadExternal { tab_index: usize },
    FileMissing { tab_index: usize },
}

#[derive(Debug, Clone)]
pub enum PathInputKind {
    Open,
    SaveAs,
    Rename,
}

#[derive(Debug, Clone)]
pub enum Modal {
    None,
    Confirm { dialog: Dialog, kind: ConfirmKind },
    PathInput {
        dialog: Dialog,
        prompt: String,
        input: TextInput,
        kind: PathInputKind,
    },
    Find(FindReplaceModal),
    ConvertTabulation(ConvertTabulationModal),
    GoToLine(GoToLineModal),
    FileBrowser(FileBrowserModal),
    Help(HelpModal),
}

impl Modal {
    pub fn is_active(&self) -> bool {
        !matches!(self, Modal::None)
    }

    pub fn dialog(&self) -> Option<&Dialog> {
        match self {
            Modal::Confirm { dialog, .. }
            | Modal::PathInput { dialog, .. } => Some(dialog),
            Modal::GoToLine(modal) => Some(&modal.dialog),
            Modal::Find(modal) => Some(&modal.dialog),
            Modal::ConvertTabulation(modal) => Some(&modal.dialog),
            Modal::FileBrowser(modal) => Some(&modal.dialog),
            Modal::Help(modal) => Some(&modal.dialog),
            Modal::None => None,
        }
    }

    pub fn dialog_mut(&mut self) -> Option<&mut Dialog> {
        match self {
            Modal::Confirm { dialog, .. }
            | Modal::PathInput { dialog, .. } => Some(dialog),
            Modal::GoToLine(modal) => Some(&mut modal.dialog),
            Modal::Find(modal) => Some(&mut modal.dialog),
            Modal::ConvertTabulation(modal) => Some(&mut modal.dialog),
            Modal::FileBrowser(modal) => Some(&mut modal.dialog),
            Modal::Help(modal) => Some(&mut modal.dialog),
            Modal::None => None,
        }
    }

    pub fn confirm(
        title: impl Into<String>,
        message: impl Into<String>,
        kind: ConfirmKind,
    ) -> Self {
        let buttons = confirm_buttons(&kind);
        Modal::Confirm {
            dialog: Dialog::message(title, message, buttons),
            kind,
        }
    }

    pub fn tab_unsaved(
        filename: &str,
        tab_index: usize,
        reason: PromptReason,
    ) -> Self {
        Modal::Confirm {
            dialog: Dialog::message(
                "Salvar alterações",
                format!("Salvar alterações em {filename}?"),
                &TAB_UNSAVED,
            ),
            kind: ConfirmKind::TabUnsaved { tab_index, reason },
        }
    }

    pub fn reload_external(filename: &str, tab_index: usize) -> Self {
        Modal::Confirm {
            dialog: Dialog::message(
                "Recarregar",
                format!("{filename} foi modificado externamente. Recarregar do disco?"),
                &RELOAD_EXTERNAL,
            ),
            kind: ConfirmKind::ReloadExternal { tab_index },
        }
    }

    pub fn file_missing(filename: &str, tab_index: usize) -> Self {
        Modal::Confirm {
            dialog: Dialog::message(
                "Arquivo ausente",
                format!("{filename} não foi encontrado. Fechar aba?"),
                &FILE_MISSING,
            ),
            kind: ConfirmKind::FileMissing { tab_index },
        }
    }

    pub fn purge_undo_toggle() -> Self {
        Modal::Confirm {
            dialog: Dialog::message(
                "Desfazer",
                "Apaga os passos de desfazer salvos ao lado do executável para liberar espaço.",
                &PURGE_UNDO,
            ),
            kind: ConfirmKind::PurgeUndoOnToggle,
        }
    }

    pub fn quit_unsaved(filename: &str) -> Self {
        Modal::Confirm {
            dialog: Dialog::message(
                "Sair",
                format!("Sair sem salvar o arquivo {filename}?"),
                &QUIT_UNSAVED,
            ),
            kind: ConfirmKind::QuitUnsaved,
        }
    }

    pub fn path_input(
        title: impl Into<String>,
        prompt: impl Into<String>,
        kind: PathInputKind,
        initial: impl Into<String>,
    ) -> Self {
        let initial = initial.into();
        let buttons = match kind {
            PathInputKind::Open => &PATH_OPEN,
            PathInputKind::SaveAs => &PATH_SAVE_AS,
            PathInputKind::Rename => &PATH_RENAME,
        };
        Modal::PathInput {
            dialog: Dialog::form(title, String::new(), buttons),
            prompt: prompt.into(),
            input: TextInput::new(initial),
            kind,
        }
    }

    pub fn go_to_line(line: usize, col: usize) -> Self {
        Modal::GoToLine(GoToLineModal::new(line, col))
    }

    pub fn find(title: impl Into<String>, pattern: impl Into<String>) -> Self {
        let _ = title;
        Modal::Find(FindReplaceModal::find(pattern))
    }

    pub fn find_replace(
        title: impl Into<String>,
        pattern: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        let _ = title;
        Modal::Find(FindReplaceModal::replace(pattern, replacement))
    }

    pub fn convert_tabulation(current: crate::encoding::Tabulation) -> Self {
        let mut modal = ConvertTabulationModal::new(current);
        modal.refresh_body();
        Modal::ConvertTabulation(modal)
    }

    pub fn file_browser(
        mode: FileBrowserMode,
        current_dir: PathBuf,
        name: impl Into<String>,
        filter: impl Into<String>,
        show_hidden: bool,
    ) -> Self {
        Modal::FileBrowser(FileBrowserModal::new(
            mode,
            current_dir,
            name.into(),
            filter.into(),
            show_hidden,
        ))
    }

    pub fn help(kind: HelpKind) -> Self {
        Modal::Help(HelpModal::new(kind))
    }

    pub fn refresh_body(&mut self) {
        match self {
            Modal::ConvertTabulation(modal) => modal.refresh_body(),
            Modal::PathInput {
                dialog,
                prompt,
                input,
                ..
            } => {
                dialog.body = format!("{prompt}\n\n{}", input.display_focused());
            }
            _ => {}
        }
    }
}

fn confirm_buttons(kind: &ConfirmKind) -> &'static [DialogButton] {
    match kind {
        ConfirmKind::QuitUnsaved => &QUIT_UNSAVED,
        ConfirmKind::DiscardForNew => &DISCARD_NEW,
        ConfirmKind::DiscardForOpen => &DISCARD_OPEN,
        ConfirmKind::CloseDocument => &DISCARD_CLOSE,
        ConfirmKind::OverwriteSave { .. } => &OVERWRITE,
        ConfirmKind::ChangeEncoding { .. } => &REINTERPRET,
        ConfirmKind::ConvertEncoding { .. } => &CONVERT,
        ConfirmKind::TabUnsaved { .. } => &TAB_UNSAVED,
        ConfirmKind::PurgeUndoOnToggle => &PURGE_UNDO,
        ConfirmKind::ReloadExternal { .. } => &RELOAD_EXTERNAL,
        ConfirmKind::FileMissing { .. } => &FILE_MISSING,
    }
}
