use std::path::{Path, PathBuf};

use crate::editor::EMPTY_DOCUMENT_TEXT;
use crate::encoding::{FileEncoding, Tabulation};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub encoding: FileEncoding,
    pub tabulation: Tabulation,
    saved_content: String,
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            encoding: FileEncoding::default(),
            tabulation: Tabulation::default(),
            saved_content: EMPTY_DOCUMENT_TEXT.to_string(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn title(&self) -> String {
        match &self.path {
            Some(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Sem título")
                .to_string(),
            None => "Sem título".to_string(),
        }
    }

    pub fn is_dirty(&self, current: &str) -> bool {
        self.saved_content != current
    }

    pub fn mark_saved(&mut self, content: String, path: PathBuf) {
        self.saved_content = content;
        self.path = Some(path);
    }

    pub fn reset_with(&mut self, encoding: FileEncoding, tabulation: Tabulation) {
        self.path = None;
        self.saved_content = EMPTY_DOCUMENT_TEXT.to_string();
        self.encoding = encoding;
        self.tabulation = tabulation;
    }

    pub fn reset(&mut self) {
        self.reset_with(FileEncoding::default(), Tabulation::default());
    }

    pub fn set_opened(&mut self, content: String, path: PathBuf) {
        self.saved_content = content;
        self.path = Some(path);
    }

    pub fn restore_untitled(&mut self, content: String, encoding: FileEncoding, tabulation: Tabulation) {
        self.path = None;
        self.encoding = encoding;
        self.tabulation = tabulation;
        self.saved_content = EMPTY_DOCUMENT_TEXT.to_string();
        let _ = content; // dirty derivado pelo conteúdo atual vs baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_document_is_not_dirty() {
        let doc = Document::new();
        assert!(!doc.is_dirty(EMPTY_DOCUMENT_TEXT));
    }

    #[test]
    fn empty_opened_file_is_not_dirty() {
        let mut doc = Document::new();
        doc.set_opened(EMPTY_DOCUMENT_TEXT.to_string(), PathBuf::from("empty.txt"));
        assert!(!doc.is_dirty(EMPTY_DOCUMENT_TEXT));
    }
}
