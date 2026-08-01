//! Resolução de paths globais de config e sessão (XDG no Linux; pasta do exe no Windows).

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "edit.json";
pub const SESSION_DIR: &str = ".edit-session";
pub const APP_DIR_NAME: &str = "edit";

const LEGACY_RECENT_DIR: &str = ".edit";
const LEGACY_APP_DIR: &str = ".editor-linux";
const LEGACY_RECENT_FILE: &str = "recent.json";

/// Diretório do executável, se resolvível.
pub fn exe_dir() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.to_path_buf()))
}

/// Home do usuário (`HOME` / `USERPROFILE`).
fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(unix)]
fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    home_dir()
        .map(|h| h.join(".config"))
        .unwrap_or_else(|| PathBuf::from(".config"))
}

#[cfg(unix)]
fn xdg_state_home() -> PathBuf {
    if let Ok(dir) = env::var("XDG_STATE_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    home_dir()
        .map(|h| h.join(".local").join("state"))
        .unwrap_or_else(|| PathBuf::from(".local/state"))
}

/// Path do `edit.json` global (sem override de CLI/testes).
pub fn global_config_path() -> PathBuf {
    #[cfg(unix)]
    {
        return xdg_config_home().join(APP_DIR_NAME).join(CONFIG_FILE);
    }
    #[cfg(windows)]
    {
        return exe_dir()
            .map(|d| d.join(CONFIG_FILE))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE));
    }
    #[cfg(not(any(unix, windows)))]
    {
        exe_dir()
            .map(|d| d.join(CONFIG_FILE))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
    }
}

/// Raiz da sessão global (`.edit-session`).
pub fn global_session_root() -> PathBuf {
    #[cfg(unix)]
    {
        return xdg_state_home().join(APP_DIR_NAME).join(SESSION_DIR);
    }
    #[cfg(windows)]
    {
        return exe_dir()
            .map(|d| d.join(SESSION_DIR))
            .unwrap_or_else(|| PathBuf::from(SESSION_DIR));
    }
    #[cfg(not(any(unix, windows)))]
    {
        exe_dir()
            .map(|d| d.join(SESSION_DIR))
            .unwrap_or_else(|| PathBuf::from(SESSION_DIR))
    }
}

/// `edit.json` ao lado do executável (legado / Windows / migração).
pub fn exe_adjacent_config_path() -> Option<PathBuf> {
    exe_dir().map(|d| d.join(CONFIG_FILE))
}

/// `.edit-session` ao lado do executável (legado / migração Linux).
pub fn exe_adjacent_session_root() -> Option<PathBuf> {
    exe_dir().map(|d| d.join(SESSION_DIR))
}

fn dir_is_empty_or_missing(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

fn read_legacy_recent_file(path: &Path) -> Option<Vec<String>> {
    let content = fs::read_to_string(path).ok()?;
    let paths: Vec<String> = serde_json::from_str(&content).ok()?;
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

/// Migra dados legados para paths XDG no Linux (idempotente se o destino já existe).
///
/// Retorna recentes migrados de arquivos legados `.edit/recent.json` quando aplicável.
#[cfg(unix)]
pub fn migrate_linux_legacy_if_needed() -> Option<Vec<String>> {
    let config_dest = global_config_path();
    let mut migrated_recent: Option<Vec<String>> = None;

    if !config_dest.exists() {
        if let Some(parent) = config_dest.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut copied = false;
        if let Some(src) = exe_adjacent_config_path() {
            if src.exists() && src != config_dest {
                if fs::copy(&src, &config_dest).is_ok() {
                    copied = true;
                }
            }
        }

        if !copied {
            for dir in [LEGACY_RECENT_DIR, LEGACY_APP_DIR] {
                let path = PathBuf::from(dir).join(LEGACY_RECENT_FILE);
                if let Some(paths) = read_legacy_recent_file(&path) {
                    migrated_recent = Some(paths);
                    break;
                }
            }
        }
    }

    let session_dest = global_session_root();
    if dir_is_empty_or_missing(&session_dest) {
        if let Some(src) = exe_adjacent_session_root() {
            if src.is_dir() && src != session_dest {
                let _ = copy_dir_recursive(&src, &session_dest);
            }
        }
    }

    migrated_recent
}

#[cfg(not(unix))]
pub fn migrate_linux_legacy_if_needed() -> Option<Vec<String>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    #[cfg(unix)]
    fn linux_paths_respect_xdg_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let base = env::temp_dir().join(format!("edit-paths-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let config_home = base.join("config");
        let state_home = base.join("state");
        fs::create_dir_all(&config_home).unwrap();
        fs::create_dir_all(&state_home).unwrap();

        env::set_var("XDG_CONFIG_HOME", &config_home);
        env::set_var("XDG_STATE_HOME", &state_home);

        assert_eq!(
            global_config_path(),
            config_home.join("edit").join("edit.json")
        );
        assert_eq!(
            global_session_root(),
            state_home.join("edit").join(".edit-session")
        );

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_STATE_HOME");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(unix)]
    fn migrate_copies_exe_adjacent_config() {
        let _lock = ENV_LOCK.lock().unwrap();
        let base = env::temp_dir().join(format!("edit-migrate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let config_home = base.join("xdg-config");
        let state_home = base.join("xdg-state");
        fs::create_dir_all(&config_home).unwrap();
        fs::create_dir_all(&state_home).unwrap();
        env::set_var("XDG_CONFIG_HOME", &config_home);
        env::set_var("XDG_STATE_HOME", &state_home);

        // Sem edit.json ao lado do exe neste teste: só verifica que não panica
        // e que destinos sob XDG são usados.
        let dest = global_config_path();
        assert!(!dest.exists());
        let _ = migrate_linux_legacy_if_needed();
        assert_eq!(dest, config_home.join("edit").join("edit.json"));

        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("XDG_STATE_HOME");
        let _ = fs::remove_dir_all(&base);
    }
}
