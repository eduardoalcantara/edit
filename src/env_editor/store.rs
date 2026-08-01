//! Load/save de variáveis por escopo (Linux environment.d / Windows registry).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::conf::{parse_conf, serialize_conf, EnvVar};

const CONF_FILE: &str = "99-edit.conf";
const ENV_DIR: &str = "environment.d";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvScope {
    User,
    System,
}

impl EnvScope {
    pub fn label(self) -> &'static str {
        match self {
            EnvScope::User => "Usuário",
            EnvScope::System => "Sistema",
        }
    }
}

#[derive(Debug)]
pub enum EnvStoreError {
    Io(io::Error),
    Message(String),
    NeedsElevation,
    ElevationCancelled,
    ElevationFailed(String),
}

impl std::fmt::Display for EnvStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvStoreError::Io(e) => write!(f, "{e}"),
            EnvStoreError::Message(m) => write!(f, "{m}"),
            EnvStoreError::NeedsElevation => {
                write!(f, "É necessário privilégio de administrador")
            }
            EnvStoreError::ElevationCancelled => {
                write!(f, "Elevação cancelada")
            }
            EnvStoreError::ElevationFailed(m) => write!(f, "{m}"),
        }
    }
}

impl From<io::Error> for EnvStoreError {
    fn from(value: io::Error) -> Self {
        EnvStoreError::Io(value)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    home_dir()
        .map(|h| h.join(".config"))
        .unwrap_or_else(|| PathBuf::from(".config"))
}

/// Path do conf de utilizador (testável via `EDIT_ENV_USER_CONF` override).
pub fn user_conf_path() -> PathBuf {
    if let Ok(p) = std::env::var("EDIT_ENV_USER_CONF") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    #[cfg(windows)]
    {
        // No Windows o store usa registry; path só para export/debug.
        return std::env::temp_dir().join("edit-user-env.conf");
    }
    #[cfg(not(windows))]
    {
        xdg_config_home().join(ENV_DIR).join(CONF_FILE)
    }
}

pub fn system_conf_path() -> PathBuf {
    if let Ok(p) = std::env::var("EDIT_ENV_SYSTEM_CONF") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    #[cfg(windows)]
    {
        return std::env::temp_dir().join("edit-system-env.conf");
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/etc").join(ENV_DIR).join(CONF_FILE)
    }
}

pub fn load_scope(scope: EnvScope) -> Result<Vec<EnvVar>, EnvStoreError> {
    #[cfg(windows)]
    {
        return load_windows(scope);
    }
    #[cfg(not(windows))]
    {
        let path = match scope {
            EnvScope::User => user_conf_path(),
            EnvScope::System => system_conf_path(),
        };
        load_conf_file(&path)
    }
}

fn load_conf_file(path: &Path) -> Result<Vec<EnvVar>, EnvStoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    Ok(parse_conf(&text))
}

pub fn save_scope(scope: EnvScope, vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    #[cfg(windows)]
    {
        return save_windows(scope, vars);
    }
    #[cfg(not(windows))]
    {
        match scope {
            EnvScope::User => save_linux_user(vars),
            EnvScope::System => save_linux_system(vars),
        }
    }
}

#[cfg(not(windows))]
fn save_linux_user(vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    let path = user_conf_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serialize_conf(vars);
    fs::write(&path, body)?;
    let _ = Command::new("systemctl")
        .args(["--user", "import-environment"])
        .status();
    Ok(())
}

/// Grava sistema: se o path for gravável (testes), escreve direto; senão NeedsElevation.
#[cfg(not(windows))]
pub fn save_linux_system(vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    let path = system_conf_path();
    if can_write_parent(&path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, serialize_conf(vars))?;
        return Ok(());
    }
    Err(EnvStoreError::NeedsElevation)
}

fn can_write_parent(path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if parent.exists() {
            return fs::OpenOptions::new()
                .write(true)
                .open(parent)
                .map(|_| true)
                .unwrap_or_else(|_| {
                    // Diretório existe mas não abre para escrita — tentar criar ficheiro teste
                    false
                })
                || tempfile_in(parent).is_ok();
        }
        // parent não existe: verificar ancestral gravável
        let mut cur = parent.to_path_buf();
        while let Some(p) = cur.parent() {
            if p.exists() {
                return tempfile_in(p).is_ok();
            }
            cur = p.to_path_buf();
        }
    }
    false
}

fn tempfile_in(dir: &Path) -> io::Result<()> {
    let name = dir.join(format!(".edit-write-test-{}", std::process::id()));
    fs::write(&name, b"")?;
    let _ = fs::remove_file(&name);
    Ok(())
}

/// Instala conf de sistema via pkexec (chamar após modal de privilégio).
#[cfg(not(windows))]
pub fn install_system_conf_elevated(vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    let path = system_conf_path();
    if std::env::var("EDIT_ENV_SYSTEM_CONF").is_ok() {
        return save_linux_system(vars);
    }

    let tmp = std::env::temp_dir().join(format!("edit-env-sys-{}.conf", std::process::id()));
    fs::write(&tmp, serialize_conf(vars))?;

    let script = format!(
        "mkdir -p /etc/environment.d && cp '{}' '{}' && chmod 644 '{}'",
        tmp.display(),
        path.display(),
        path.display()
    );

    let status = Command::new("pkexec")
        .args(["sh", "-c", &script])
        .status()
        .map_err(|e| {
            EnvStoreError::ElevationFailed(format!(
                "pkexec indisponível ({e}). Instale polkit ou grave só variáveis de Usuário."
            ))
        })?;

    let _ = fs::remove_file(&tmp);

    if status.success() {
        Ok(())
    } else if matches!(status.code(), Some(126) | Some(127)) {
        Err(EnvStoreError::ElevationCancelled)
    } else {
        // 1 = tipicamente cancelado pelo utilizador no diálogo polkit
        if status.code() == Some(1) {
            Err(EnvStoreError::ElevationCancelled)
        } else {
            Err(EnvStoreError::ElevationFailed(format!(
                "pkexec terminou com código {:?}",
                status.code()
            )))
        }
    }
}

#[cfg(windows)]
pub fn install_system_conf_elevated(_vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    Err(EnvStoreError::Message(
        "No Windows, execute o edit como administrador para gravar variáveis de sistema.".into(),
    ))
}

#[cfg(windows)]
fn load_windows(scope: EnvScope) -> Result<Vec<EnvVar>, EnvStoreError> {
    use winreg::enums::*;
    use winreg::RegKey;

    let (hive, subkey) = match scope {
        EnvScope::User => (HKEY_CURRENT_USER, r"Environment"),
        EnvScope::System => (
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
    };
    let root = RegKey::predef(hive);
    let key = match root.open_subkey(subkey) {
        Ok(k) => k,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(EnvStoreError::NeedsElevation);
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(EnvStoreError::Io(e)),
    };
    let mut vars = Vec::new();
    for item in key.enum_values() {
        let (name, _) = item.map_err(EnvStoreError::Io)?;
        if name.is_empty() {
            continue;
        }
        let value: String = key.get_value(&name).unwrap_or_default();
        vars.push(EnvVar { name, value });
    }
    vars.sort_by(|a, b| a.name.to_ascii_uppercase().cmp(&b.name.to_ascii_uppercase()));
    Ok(vars)
}

#[cfg(windows)]
fn save_windows(scope: EnvScope, vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    use winreg::enums::*;
    use winreg::RegKey;

    let (hive, subkey) = match scope {
        EnvScope::User => (HKEY_CURRENT_USER, r"Environment"),
        EnvScope::System => (
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
    };
    let root = RegKey::predef(hive);
    let (key, _) = root.create_subkey(subkey).map_err(|e| {
        if scope == EnvScope::System {
            EnvStoreError::NeedsElevation
        } else {
            EnvStoreError::Io(e)
        }
    })?;

    let existing: Vec<String> = key
        .enum_values()
        .filter_map(|r| r.ok().map(|(n, _)| n))
        .collect();
    let keep: std::collections::HashSet<&str> = vars.iter().map(|v| v.name.as_str()).collect();
    for name in existing {
        if !keep.contains(name.as_str()) {
            let _ = key.delete_value(&name);
        }
    }
    for var in vars {
        key.set_value(&var.name, &var.value).map_err(|e| {
            if scope == EnvScope::System {
                EnvStoreError::NeedsElevation
            } else {
                EnvStoreError::Io(e)
            }
        })?;
    }
    broadcast_environment_change();
    Ok(())
}

#[cfg(windows)]
fn broadcast_environment_change() {
    #[link(name = "user32")]
    extern "system" {
        fn SendMessageTimeoutW(
            hwnd: isize,
            msg: u32,
            wparam: usize,
            lparam: *const u16,
            flags: u32,
            timeout: u32,
            result: *mut usize,
        ) -> isize;
    }
    const HWND_BROADCAST: isize = 0xffff;
    const WM_SETTINGCHANGE: u32 = 0x001A;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;
    let env: Vec<u16> = "Environment\0".encode_utf16().collect();
    let mut result = 0usize;
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env.as_ptr(),
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_roundtrip_via_override() {
        let dir = std::env::temp_dir().join(format!("edit-env-user-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("99-edit.conf");
        std::env::set_var("EDIT_ENV_USER_CONF", &path);

        let vars = vec![EnvVar {
            name: "EDIT_TEST_VAR".into(),
            value: "hello".into(),
        }];
        save_scope(EnvScope::User, &vars).unwrap();
        let loaded = load_scope(EnvScope::User).unwrap();
        assert_eq!(loaded, vars);

        std::env::remove_var("EDIT_ENV_USER_CONF");
        let _ = fs::remove_dir_all(&dir);
    }
}
