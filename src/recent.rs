use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, Default)]
pub struct RecentFiles {
    paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut recent = Self { paths };
        recent.paths.truncate(MAX_RECENT);
        recent
    }

    pub fn push(&mut self, path: PathBuf) {
        self.paths.retain(|p| p != &path);
        self.paths.insert(0, path);
        self.paths.truncate(MAX_RECENT);
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string()
}
