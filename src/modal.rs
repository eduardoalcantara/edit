use std::path::PathBuf;

use crate::encoding::FileEncoding;

#[derive(Debug, Clone)]
pub enum ConfirmKind {
    QuitUnsaved,
    DiscardForNew,
    DiscardForOpen,
    CloseDocument,
    OverwriteSave { path: PathBuf },
    ReinterpretEncoding { encoding: FileEncoding },
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
    Confirm {
        title: String,
        message: String,
        kind: ConfirmKind,
        selected: usize,
    },
    PathInput {
        title: String,
        prompt: String,
        input: String,
        kind: PathInputKind,
    },
    Find {
        title: String,
        pattern: String,
        replacement: String,
        replace_mode: bool,
    },
}

impl Modal {
    pub fn is_active(&self) -> bool {
        !matches!(self, Modal::None)
    }

    pub fn confirm(title: impl Into<String>, message: impl Into<String>, kind: ConfirmKind) -> Self {
        Modal::Confirm {
            title: title.into(),
            message: message.into(),
            kind,
            selected: 0,
        }
    }

    pub fn path_input(title: impl Into<String>, prompt: impl Into<String>, kind: PathInputKind) -> Self {
        Modal::PathInput {
            title: title.into(),
            prompt: prompt.into(),
            input: String::new(),
            kind,
        }
    }

    pub fn find(title: impl Into<String>, pattern: impl Into<String>) -> Self {
        Modal::Find {
            title: title.into(),
            pattern: pattern.into(),
            replacement: String::new(),
            replace_mode: false,
        }
    }

    pub fn find_replace(title: impl Into<String>, pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Modal::Find {
            title: title.into(),
            pattern: pattern.into(),
            replacement: replacement.into(),
            replace_mode: true,
        }
    }
}
