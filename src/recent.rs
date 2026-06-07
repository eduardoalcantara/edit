use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 10;
const RECENT_DIR: &str = ".editor-linux";
const RECENT_FILE: &str = "recent.json";

#[derive(Debug, Clone, Default)]
pub struct RecentFiles {
    paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn load() -> Self {
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

fn recent_path() -> PathBuf {
    PathBuf::from(RECENT_DIR).join(RECENT_FILE)
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
