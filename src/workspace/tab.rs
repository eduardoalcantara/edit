use std::path::Path;
use std::time::{Instant, SystemTime};

use crate::document::Document;
use crate::editor::{Editor, EMPTY_DOCUMENT_TEXT};
use crate::encoding::{FileEncoding, Tabulation};
use crate::theme::ThemePalette;

#[derive(Debug, Clone)]
pub struct FileSnapshot {
    pub modified: SystemTime,
    pub len: u64,
}

pub struct Tab {
    pub editor: Editor,
    pub document: Document,
    pub session_id: String,
    pub display_name: String,
    pub opened_at: Instant,
    pub fs_snapshot: Option<FileSnapshot>,
    pub is_temp_file: bool,
}

impl Tab {
    pub fn new_untitled(
        editor: Editor,
        document: Document,
        session_id: String,
        display_name: String,
    ) -> Self {
        Self {
            editor,
            document,
            session_id,
            display_name,
            opened_at: Instant::now(),
            fs_snapshot: None,
            is_temp_file: false,
        }
    }

    pub fn is_pristine(&self) -> bool {
        self.document
            .is_dirty(&self.editor.content_string())
            == false
            && self.editor.content_string() == EMPTY_DOCUMENT_TEXT
    }

    pub fn is_dirty(&self) -> bool {
        self.document.is_dirty(&self.editor.content_string())
    }

    pub fn filepath(&self) -> Option<&Path> {
        self.document.path()
    }

    pub fn menu_label(&self) -> String {
        let mut label = if let Some(path) = self.document.path() {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string()
        } else {
            self.display_name.clone()
        };
        if self.is_dirty() {
            label.push('*');
        }
        label
    }
}

pub fn snapshot_path(path: &Path) -> std::io::Result<FileSnapshot> {
    let meta = std::fs::metadata(path)?;
    Ok(FileSnapshot {
        modified: meta.modified()?,
        len: meta.len(),
    })
}

pub fn novo_display_name(counter: u32) -> String {
    if counter == 0 {
        "Novo".to_string()
    } else {
        format!("Novo{counter}")
    }
}

pub fn next_novo_counter(existing: &[Tab]) -> u32 {
    let mut max = 0u32;
    for tab in existing {
        if tab.document.path().is_some() {
            continue;
        }
        let name = tab.display_name.as_str();
        if name == "Novo" {
            max = max.max(1);
        } else if let Some(suffix) = name.strip_prefix("Novo") {
            if suffix.is_empty() {
                max = max.max(1);
            } else if let Ok(n) = suffix.parse::<u32>() {
                max = max.max(n + 1);
            }
        }
    }
    max
}

pub fn create_tab_from_defaults(
    palette: &ThemePalette,
    encoding: FileEncoding,
    tabulation: Tabulation,
    word_wrap: bool,
    session_id: String,
    display_name: String,
) -> Tab {
    let mut document = Document::new();
    document.encoding = encoding;
    document.tabulation = tabulation;
    let mut editor = Editor::new(palette);
    editor.set_tabulation(tabulation);
    editor.set_word_wrap(word_wrap);
    Tab::new_untitled(editor, document, session_id, display_name)
}
