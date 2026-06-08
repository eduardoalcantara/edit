//! Resolução de caminho a partir do campo Name e diretório atual.

use std::path::{Path, PathBuf};

pub fn infer_filter_from_path(path: Option<&Path>) -> String {
    path.and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|ext| format!("*.{ext}"))
        .unwrap_or_else(|| "*.*".to_string())
}

pub fn suggest_file_name(path: Option<&Path>, untitled: &str) -> String {
    path.and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| untitled.to_string())
}

pub fn initial_directory(
    document_path: Option<&Path>,
    last_dir: Option<&Path>,
    cwd: &Path,
) -> PathBuf {
    if let Some(path) = document_path {
        if let Some(parent) = path.parent() {
            if parent.exists() {
                return parent.to_path_buf();
            }
        }
    }
    if let Some(last) = last_dir {
        if last.is_dir() {
            return last.to_path_buf();
        }
    }
    cwd.to_path_buf()
}

/// Resolve `name` contra `current_dir`. Suporta path absoluto e `D:` no Windows.
pub fn resolve_name(current_dir: &Path, name: &str) -> PathBuf {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return current_dir.to_path_buf();
    }

    #[cfg(windows)]
    {
        if trimmed.len() == 2 {
            let bytes = trimmed.as_bytes();
            if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                return PathBuf::from(format!("{}:\\", bytes[0] as char));
            }
        }
    }

    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

pub fn resolve_open_target(current_dir: &Path, name: &str, selected: Option<&Path>) -> PathBuf {
    if let Some(path) = selected {
        return path.to_path_buf();
    }
    resolve_name(current_dir, name)
}

pub fn resolve_save_target(current_dir: &Path, name: &str) -> PathBuf {
    resolve_name(current_dir, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_and_absolute() {
        let cwd = PathBuf::from("/home/user/docs");
        assert_eq!(
            resolve_name(&cwd, "file.txt"),
            PathBuf::from("/home/user/docs/file.txt")
        );
        assert_eq!(
            resolve_name(&cwd, "/tmp/x"),
            PathBuf::from("/tmp/x")
        );
    }

    #[test]
    fn infer_filter_from_extension() {
        assert_eq!(
            infer_filter_from_path(Some(Path::new("main.rs"))),
            "*.rs"
        );
        assert_eq!(infer_filter_from_path(None), "*.*");
    }
}
