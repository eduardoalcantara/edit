//! Persistência de sessão em `.edit-session/` ao lado do executável.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::editor::history::{HistoryStacks, PERSIST_UNDO_MIN};

const SESSION_DIR: &str = ".edit-session";
const TABS_DIR: &str = "tabs";

static SESSION_ROOT_OVERRIDE: std::sync::Mutex<Option<PathBuf>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
static SESSION_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    SESSION_TEST_LOCK.lock().expect("session test lock")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMeta {
    pub content_hash: String,
    pub cursor_linha: usize,
    pub cursor_coluna: usize,
    pub encoding: String,
    pub fs_mtime_ms: Option<u64>,
    pub fs_len: Option<u64>,
    pub saved_at_ms: u64,
}

pub fn set_session_root(path: PathBuf) {
    let mut guard = SESSION_ROOT_OVERRIDE.lock().expect("session root lock");
    *guard = Some(path);
}

#[cfg(test)]
pub fn clear_session_root_override() {
    let mut guard = SESSION_ROOT_OVERRIDE.lock().expect("session root lock");
    *guard = None;
}

pub fn session_root() -> PathBuf {
    if let Ok(guard) = SESSION_ROOT_OVERRIDE.lock() {
        if let Some(path) = guard.as_ref() {
            return path.clone();
        }
    }
    session_root_from(
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf())),
    )
}

fn session_root_from(exe_dir: Option<PathBuf>) -> PathBuf {
    exe_dir
        .unwrap_or_else(|| PathBuf::from("."))
        .join(SESSION_DIR)
}

pub fn tab_dir(session_id: &str) -> PathBuf {
    session_root().join(TABS_DIR).join(session_id)
}

pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn system_time_to_ms(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

pub fn now_ms() -> u64 {
    system_time_to_ms(SystemTime::now()).unwrap_or(0)
}

pub fn has_any_undo_files() -> io::Result<bool> {
    let tabs = session_root().join(TABS_DIR);
    if !tabs.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(tabs)? {
        let entry = entry?;
        if entry.path().join("undo.json").is_file() || entry.path().join("redo.json").is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn purge_tab(session_id: &str) -> io::Result<()> {
    let path = tab_dir(session_id);
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn purge_undo(session_id: &str) -> io::Result<()> {
    let dir = tab_dir(session_id);
    let _ = fs::remove_file(dir.join("undo.json"));
    let _ = fs::remove_file(dir.join("redo.json"));
    Ok(())
}

pub fn purge_all_undo() -> io::Result<()> {
    let tabs = session_root().join(TABS_DIR);
    if !tabs.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(tabs)? {
        let entry = entry?;
        let _ = purge_undo(&entry.file_name().to_string_lossy());
    }
    Ok(())
}

pub fn purge_all() -> io::Result<()> {
    let root = session_root();
    if root.is_dir() {
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

pub fn purge_orphans(valid_ids: &[String]) -> io::Result<()> {
    let tabs = session_root().join(TABS_DIR);
    if !tabs.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(tabs)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !valid_ids.iter().any(|id| id == &name) {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

pub fn write_content_tmp(session_id: &str, content: &str) -> io::Result<PathBuf> {
    let dir = tab_dir(session_id);
    fs::create_dir_all(&dir)?;
    let path = dir.join("content.tmp");
    fs::write(&path, content)?;
    Ok(path)
}

pub fn read_content_tmp(session_id: &str) -> io::Result<Option<String>> {
    let path = tab_dir(session_id).join("content.tmp");
    if path.is_file() {
        Ok(Some(fs::read_to_string(path)?))
    } else {
        Ok(None)
    }
}

pub fn write_meta(session_id: &str, meta: &SessionMeta) -> io::Result<()> {
    let dir = tab_dir(session_id);
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(meta)?;
    fs::write(dir.join("meta.json"), json)?;
    Ok(())
}

pub fn read_meta(session_id: &str) -> io::Result<Option<SessionMeta>> {
    let path = tab_dir(session_id).join("meta.json");
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

pub fn write_undo_stacks(session_id: &str, stacks: &HistoryStacks) -> io::Result<()> {
    if stacks.undo.len() < PERSIST_UNDO_MIN {
        return Ok(());
    }
    let dir = tab_dir(session_id);
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("undo.json"),
        serde_json::to_string_pretty(&stacks.undo)?,
    )?;
    fs::write(
        dir.join("redo.json"),
        serde_json::to_string_pretty(&stacks.redo)?,
    )?;
    Ok(())
}

pub fn read_undo_stacks(session_id: &str) -> io::Result<Option<HistoryStacks>> {
    let undo_path = tab_dir(session_id).join("undo.json");
    if !undo_path.is_file() {
        return Ok(None);
    }
    let undo: Vec<_> = serde_json::from_str(&fs::read_to_string(undo_path)?)?;
    let redo_path = tab_dir(session_id).join("redo.json");
    let redo = if redo_path.is_file() {
        serde_json::from_str(&fs::read_to_string(redo_path)?)?
    } else {
        Vec::new()
    };
    Ok(Some(HistoryStacks { undo, redo }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::HistoryStacks;
    use std::sync::{Mutex, MutexGuard};

    struct Guard {
        _lock: std::sync::MutexGuard<'static, ()>,
        root: PathBuf,
    }

    impl Guard {
        fn new() -> Self {
            let lock = super::test_lock();
            let root = std::env::temp_dir().join(format!(
                "edit-session-test-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            set_session_root(root.clone());
            Self { _lock: lock, root }
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
            clear_session_root_override();
        }
    }

    #[test]
    fn purge_tab_removes_directory() {
        let _guard = Guard::new();
        let id = "test-tab";
        let dir = tab_dir(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("undo.json"), "[]").unwrap();
        purge_tab(id).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn purge_undo_keeps_content_tmp() {
        let _guard = Guard::new();
        let id = "tab-undo";
        write_content_tmp(id, "draft").unwrap();
        let mut history = crate::editor::history::EditHistory::new();
        history.record_change(0, String::new(), "a".into(), 0, 1);
        let stacks = history.export_stacks();
        write_undo_stacks(id, &stacks).unwrap();
        purge_undo(id).unwrap();
        assert!(tab_dir(id).join("content.tmp").is_file());
        assert!(!tab_dir(id).join("undo.json").exists());
    }

    #[test]
    fn undo_round_trip() {
        let _guard = Guard::new();
        let id = "tab-round";
        let mut history = crate::editor::history::EditHistory::new();
        for i in 0..6 {
            history.record_change(0, String::new(), format!("{i}"), 0, 1);
        }
        let stacks = history.export_for_persist();
        write_undo_stacks(id, &stacks).unwrap();
        let loaded = read_undo_stacks(id).unwrap().expect("stacks");
        assert_eq!(loaded.undo.len(), 6);
    }

    #[test]
    fn purge_orphans_removes_unknown_tab_dirs() {
        let _guard = Guard::new();
        fs::create_dir_all(tab_dir("keep")).unwrap();
        fs::create_dir_all(tab_dir("orphan")).unwrap();
        purge_orphans(&["keep".to_string()]).unwrap();
        assert!(tab_dir("keep").is_dir());
        assert!(!tab_dir("orphan").exists());
    }

    #[test]
    fn content_hash_stable() {
        assert_eq!(content_hash("hello"), content_hash("hello"));
        assert_ne!(content_hash("hello"), content_hash("world"));
    }
}
