use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsDrift {
    Ok,
    ModifiedExternally,
    Deleted,
}

pub fn check_fs_drift(path: &Path, snapshot: Option<&FileSnapshot>) -> FsDrift {
    if !path.is_file() {
        return FsDrift::Deleted;
    }
    let Some(snapshot) = snapshot else {
        return FsDrift::Ok;
    };
    let Ok(current) = snapshot_path(path) else {
        return FsDrift::Deleted;
    };
    if current.len != snapshot.len {
        return FsDrift::ModifiedExternally;
    }
    let snap_ms = snapshot
        .modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis());
    let cur_ms = current
        .modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis());
    match (snap_ms, cur_ms) {
        (Some(a), Some(b)) if a != b => FsDrift::ModifiedExternally,
        _ => FsDrift::Ok,
    }
}

pub fn check_fs_drift_from_entry(
    path: &Path,
    fs_mtime_ms: Option<u64>,
    fs_len: Option<u64>,
) -> FsDrift {
    if !path.is_file() {
        return FsDrift::Deleted;
    }
    let Ok(meta) = std::fs::metadata(path) else {
        return FsDrift::Deleted;
    };
    if let Some(len) = fs_len {
        if meta.len() != len {
            return FsDrift::ModifiedExternally;
        }
    }
    if let Some(expected_ms) = fs_mtime_ms {
        if let Ok(modified) = meta.modified() {
            if let Ok(dur) = modified.duration_since(UNIX_EPOCH) {
                if dur.as_millis() as u64 != expected_ms {
                    return FsDrift::ModifiedExternally;
                }
            }
        }
    }
    FsDrift::Ok
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn check_fs_drift_detects_deleted_file() {
        let path = std::env::temp_dir().join(format!("edit-drift-{}.txt", std::process::id()));
        std::fs::write(&path, b"x").unwrap();
        let snap = snapshot_path(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(check_fs_drift(&path, Some(&snap)), FsDrift::Deleted);
    }

    #[test]
    fn check_fs_drift_detects_size_change() {
        let path = std::env::temp_dir().join(format!("edit-size-{}.txt", std::process::id()));
        std::fs::write(&path, b"short").unwrap();
        let snap = snapshot_path(&path).unwrap();
        std::fs::write(&path, b"much longer content").unwrap();
        assert_eq!(
            check_fs_drift(&path, Some(&snap)),
            FsDrift::ModifiedExternally
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn check_fs_drift_from_entry_matches_metadata() {
        let path = std::env::temp_dir().join(format!("edit-meta-{}.txt", std::process::id()));
        std::fs::write(&path, b"abc").unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        let mtime = meta.modified().unwrap();
        let ms = mtime.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        assert_eq!(
            check_fs_drift_from_entry(&path, Some(ms), Some(meta.len())),
            FsDrift::Ok
        );
        std::thread::sleep(Duration::from_millis(50));
        let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"!").unwrap();
        drop(file);
        assert_eq!(
            check_fs_drift_from_entry(&path, Some(ms), Some(meta.len())),
            FsDrift::ModifiedExternally
        );
        let _ = std::fs::remove_file(&path);
    }
}
