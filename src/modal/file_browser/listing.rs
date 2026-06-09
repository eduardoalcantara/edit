//! Leitura de diretório, filtro glob e arquivos ocultos.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEntryKind {
    Parent,
    Dir,
    File,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: FileEntryKind,
    pub meta: FileMetadata,
}

pub fn list_directory(
    dir: &Path,
    filter: &str,
    show_hidden: bool,
) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    if dir.parent().is_some() {
        entries.push(FileEntry {
            name: "..".to_string(),
            path: dir.parent().unwrap_or(dir).to_path_buf(),
            kind: FileEntryKind::Parent,
            meta: FileMetadata {
                size: None,
                modified: None,
            },
        });
    }

    let read = fs::read_dir(dir).map_err(|e| format!("Não foi possível ler {}: {e}", dir.display()))?;

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for item in read {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        let name = item.file_name().to_string_lossy().into_owned();
        if name.is_empty() {
            continue;
        }
        let meta = item.metadata().ok();
        if !show_hidden && is_hidden(&name, meta.as_ref()) {
            continue;
        }
        let file_type = item.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            dirs.push(FileEntry {
                name,
                path,
                kind: FileEntryKind::Dir,
                meta: metadata_from(meta.as_ref()),
            });
        } else if file_type.is_file() {
            if !matches_glob(filter, &name) {
                continue;
            }
            files.push(FileEntry {
                name,
                path,
                kind: FileEntryKind::File,
                meta: metadata_from(meta.as_ref()),
            });
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries.extend(dirs);
    entries.extend(files);
    Ok(entries)
}

fn metadata_from(meta: Option<&fs::Metadata>) -> FileMetadata {
    let Some(meta) = meta else {
        return FileMetadata {
            size: None,
            modified: None,
        };
    };
    FileMetadata {
        size: if meta.is_file() { Some(meta.len()) } else { None },
        modified: meta.modified().ok(),
    }
}

fn is_hidden(name: &str, meta: Option<&fs::Metadata>) -> bool {
    if name.starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    {
        if let Some(meta) = meta {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
            let attrs = meta.file_attributes();
            if attrs & FILE_ATTRIBUTE_HIDDEN != 0 || attrs & FILE_ATTRIBUTE_SYSTEM != 0 {
                return true;
            }
        }
    }
    let _ = meta;
    false
}

pub fn matches_glob(pattern: &str, name: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() || pattern == "*.*" || pattern == "*" {
        return true;
    }
    glob_match(pattern.as_bytes(), name.as_bytes())
}

fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    glob_match_impl(pattern, text, 0, 0)
}

fn glob_match_impl(pattern: &[u8], text: &[u8], pi: usize, ti: usize) -> bool {
    if pi == pattern.len() {
        return ti == text.len();
    }
    if pattern[pi] == b'*' {
        if pi + 1 == pattern.len() {
            return true;
        }
        for i in ti..=text.len() {
            if glob_match_impl(pattern, text, pi + 1, i) {
                return true;
            }
        }
        return false;
    }
    if ti >= text.len() {
        return false;
    }
    if pattern[pi] == b'?' || pattern[pi] == text[ti] {
        return glob_match_impl(pattern, text, pi + 1, ti + 1);
    }
    false
}

pub fn format_metadata_line(entry: &FileEntry) -> String {
    match entry.kind {
        FileEntryKind::Parent => String::new(),
        FileEntryKind::Dir => format!(
            "{}   -   {}",
            entry.name,
            format_modified(entry.meta.modified)
        ),
        FileEntryKind::File => format!(
            "{}   {}   {}",
            entry.name,
            entry.meta.size.unwrap_or(0),
            format_modified(entry.meta.modified)
        ),
    }
}

pub fn format_modified(time: Option<SystemTime>) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let Some(time) = time else {
        return "-".to_string();
    };
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "-".to_string();
    };
    let secs = duration.as_secs() as i64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let year = 1970 + days / 365;
    let month = ((days % 365) / 30).clamp(0, 11) + 1;
    let day = (days % 30).clamp(0, 29) + 1;
    let ampm = if hour < 12 { "am" } else { "pm" };
    let h12 = if hour % 12 == 0 { 12 } else { hour % 12 };
    let _ = Duration::ZERO;
    format!("{day} {month}/{year}  {h12}:{min:02}{ampm}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("edit-fb-{prefix}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn applies_glob_filter() {
        assert!(matches_glob("*.rs", "main.rs"));
        assert!(!matches_glob("*.rs", "main.txt"));
        assert!(matches_glob("*.*", "readme.md"));
        assert!(matches_glob("README*", "README.md"));
    }

    #[test]
    fn hidden_dotfiles_skipped_when_disabled() {
        let dir = temp_dir("hidden");
        fs::write(dir.join("visible.txt"), b"x").unwrap();
        fs::write(dir.join(".hidden"), b"x").unwrap();
        let entries = list_directory(&dir, "*.*", false).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"visible.txt"));
        assert!(!names.iter().any(|n| *n == ".hidden"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parent_entry_first_when_parent_exists() {
        let dir = temp_dir("parent");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        let entries = list_directory(&sub, "*.*", true).unwrap();
        assert_eq!(entries.first().map(|e| e.kind), Some(FileEntryKind::Parent));
        let _ = fs::remove_dir_all(&dir);
    }
}
