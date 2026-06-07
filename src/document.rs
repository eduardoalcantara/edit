use std::path::{Path, PathBuf};

use crate::encoding::{FileEncoding, Tabulation};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub encoding: FileEncoding,
    pub tabulation: Tabulation,
    saved_lines: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            encoding: FileEncoding::default(),
            tabulation: Tabulation::default(),
            saved_lines: vec![String::new()],
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

    pub fn is_dirty(&self, current: &[String]) -> bool {
        self.saved_lines != current
    }

    pub fn mark_saved(&mut self, lines: Vec<String>, path: PathBuf) {
        self.saved_lines = lines;
        self.path = Some(path);
    }

    pub fn reset(&mut self) {
        self.path = None;
        self.saved_lines = vec![String::new()];
        self.encoding = FileEncoding::default();
        self.tabulation = Tabulation::default();
    }

    pub fn set_opened(&mut self, lines: Vec<String>, path: PathBuf) {
        self.saved_lines = lines.clone();
        self.path = Some(path);
    }
}
