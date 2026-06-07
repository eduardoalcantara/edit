use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 10;
pub const APP_DIR: &str = ".edit";
const LEGACY_APP_DIR: &str = ".editor-linux";
const RECENT_FILE: &str = "recent.json";

#[derive(Debug, Clone, Default)]
pub struct RecentFiles {
    paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn load() -> Self {
        migrate_legacy_data();
        let path = recent_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(list) = serde_parse(&content) {
                return Self { paths: list };
            }
        }
        Self::default()
    }

    pub fn push(&mut self, path: PathBuf) {
        self.paths.retain(|p| p != &path);
        self.paths.insert(0, path);
        self.paths.truncate(MAX_RECENT);
        let _ = self.save();
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    fn save(&self) -> io::Result<()> {
        let path = recent_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lines: Vec<String> = self.paths.iter().map(|p| p.display().to_string()).collect();
        let json = format!(
            "[{}]",
            lines
                .iter()
                .map(|s| format!("\"{}\"", escape_json(s)))
                .collect::<Vec<_>>()
                .join(",")
        );
        fs::write(path, json)
    }
}

pub fn app_dir() -> PathBuf {
    PathBuf::from(APP_DIR)
}

fn recent_path() -> PathBuf {
    app_dir().join(RECENT_FILE)
}

fn migrate_legacy_data() {
    let new_path = recent_path();
    if new_path.exists() {
        return;
    }

    let legacy_path = PathBuf::from(LEGACY_APP_DIR).join(RECENT_FILE);
    if !legacy_path.exists() {
        return;
    }

    if let Some(parent) = new_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if fs::rename(&legacy_path, &new_path).is_err() {
        let _ = fs::copy(&legacy_path, &new_path);
    }

    let _ = fs::remove_dir(PathBuf::from(LEGACY_APP_DIR));
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn serde_parse(content: &str) -> Option<Vec<PathBuf>> {
    let trimmed = content.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(vec![]);
    }
    let mut out = Vec::new();
    for part in split_json_strings(inner) {
        out.push(PathBuf::from(part));
    }
    Some(out)
}

fn split_json_strings(inner: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape = false;
    for ch in inner.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            if in_string {
                result.push(current.clone());
                current.clear();
            }
            in_string = !in_string;
            continue;
        }
        if in_string {
            current.push(ch);
        }
    }
    result
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static TEST_DIR_LOCK: Mutex<()> = Mutex::new(());

    struct TestDirGuard {
        _lock: MutexGuard<'static, ()>,
        created: Vec<PathBuf>,
    }

    impl TestDirGuard {
        fn new() -> Self {
            let lock = TEST_DIR_LOCK.lock().unwrap();
            Self {
                _lock: lock,
                created: Vec::new(),
            }
        }

        fn track(&mut self, path: PathBuf) {
            self.created.push(path);
        }
    }

    impl Drop for TestDirGuard {
        fn drop(&mut self) {
            for path in self.created.iter().rev() {
                let _ = fs::remove_file(path);
                if let Some(parent) = path.parent() {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
    }

    #[test]
    fn migrates_recent_json_from_legacy_dir() {
        let mut guard = TestDirGuard::new();

        let _ = fs::remove_file(recent_path());
        let _ = fs::remove_dir(app_dir());
        let _ = fs::remove_dir(PathBuf::from(LEGACY_APP_DIR));

        let legacy_dir = PathBuf::from(LEGACY_APP_DIR);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy_file = legacy_dir.join(RECENT_FILE);
        fs::write(&legacy_file, r#"["/tmp/exemplo.txt"]"#).unwrap();
        guard.track(legacy_file.clone());

        let recent = RecentFiles::load();
        assert_eq!(recent.paths(), &[PathBuf::from("/tmp/exemplo.txt")]);
        assert!(recent_path().exists());
        assert!(!legacy_file.exists());

        guard.track(recent_path());
        guard.track(app_dir());
    }
}
