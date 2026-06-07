mod buttons;
pub mod convert_tab;
pub mod dialog;

use std::path::PathBuf;

pub use convert_tab::ConvertTabulationModal;
pub use dialog::{Dialog, DialogButton, DialogButtonAction, DialogKeyResult};

use crate::encoding::FileEncoding;

use buttons::{
    CONVERT, DISCARD_CLOSE, DISCARD_NEW, DISCARD_OPEN, FIND, FIND_REPLACE, OVERWRITE,
    PATH_OPEN, PATH_SAVE_AS, QUIT_UNSAVED, REINTERPRET,
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
}

#[derive(Debug, Clone)]
pub enum PathInputKind {
    Open,
    SaveAs,
}

#[derive(Debug, Clone)]
pub enum Modal {
    None,
    Confirm { dialog: Dialog, kind: ConfirmKind },
    PathInput {
        dialog: Dialog,
        prompt: String,
        input: String,
        kind: PathInputKind,
    },
    Find {
        dialog: Dialog,
        pattern: String,
        replacement: String,
        replace_mode: bool,
    },
    ConvertTabulation(ConvertTabulationModal),
}

impl Modal {
    pub fn is_active(&self) -> bool {
        !matches!(self, Modal::None)
    }

    pub fn dialog(&self) -> Option<&Dialog> {
        match self {
            Modal::Confirm { dialog, .. }
            | Modal::PathInput { dialog, .. }
            | Modal::Find { dialog, .. } => Some(dialog),
            Modal::ConvertTabulation(modal) => Some(&modal.dialog),
            Modal::None => None,
        }
    }

    pub fn dialog_mut(&mut self) -> Option<&mut Dialog> {
        match self {
            Modal::Confirm { dialog, .. }
            | Modal::PathInput { dialog, .. }
            | Modal::Find { dialog, .. } => Some(dialog),
            Modal::ConvertTabulation(modal) => Some(&mut modal.dialog),
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

    pub fn path_input(title: impl Into<String>, prompt: impl Into<String>, kind: PathInputKind) -> Self {
        let buttons = match kind {
            PathInputKind::Open => &PATH_OPEN,
            PathInputKind::SaveAs => &PATH_SAVE_AS,
        };
        let prompt = prompt.into();
        Modal::PathInput {
            dialog: Dialog::form(title, String::new(), buttons),
            prompt: prompt.into(),
            input: String::new(),
            kind,
        }
    }

    pub fn find(title: impl Into<String>, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        Modal::Find {
            dialog: Dialog::form(
                title,
                format!("Texto:\n {pattern}▌"),
                &FIND,
            ),
            pattern,
            replacement: String::new(),
            replace_mode: false,
        }
    }

    pub fn find_replace(
        title: impl Into<String>,
        pattern: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        let pattern = pattern.into();
        let replacement = replacement.into();
        Modal::Find {
            dialog: Dialog::form(
                title,
                format!("Texto:\n {pattern}\n\nSubstituir:\n {replacement}▌"),
                &FIND_REPLACE,
            ),
            pattern,
            replacement,
            replace_mode: true,
        }
    }

    pub fn convert_tabulation(current: crate::encoding::Tabulation) -> Self {
        let mut modal = ConvertTabulationModal::new(current);
        modal.refresh_body();
        Modal::ConvertTabulation(modal)
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
                dialog.body = format!("{prompt}\n\n {input}▌");
            }
            Modal::Find {
                dialog,
                pattern,
                replacement,
                replace_mode,
            } => {
                dialog.body = if *replace_mode {
                    format!("Texto:\n {pattern}\n\nSubstituir:\n {replacement}▌")
                } else {
                    format!("Texto:\n {pattern}▌")
                };
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
    }
}
