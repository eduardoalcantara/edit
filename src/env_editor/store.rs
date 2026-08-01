//! Load/save de variáveis por escopo (Linux environment.d / Windows registry).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::conf::{parse_conf, serialize_conf, validate_var_name, EnvVar};

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

/// Path do conf de utilizador.
/// Em testes: `EDIT_ENV_USER_CONF` (nunca honrar em builds de produção — evita
/// leftovers no systemd --user a apontar para `/tmp/...` e lista vazia).
pub fn user_conf_path() -> PathBuf {
    #[cfg(test)]
    {
        if let Ok(p) = std::env::var("EDIT_ENV_USER_CONF") {
            if !p.is_empty() {
                return PathBuf::from(p);
            }
        }
    }
    #[cfg(windows)]
    {
        return std::env::temp_dir().join("edit-user-env.conf");
    }
    #[cfg(not(windows))]
    {
        xdg_config_home().join(ENV_DIR).join(CONF_FILE)
    }
}

pub fn system_conf_path() -> PathBuf {
    #[cfg(test)]
    {
        if let Ok(p) = std::env::var("EDIT_ENV_SYSTEM_CONF") {
            if !p.is_empty() {
                return PathBuf::from(p);
            }
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
        match scope {
            EnvScope::User => load_linux_user(),
            EnvScope::System => load_linux_system(),
        }
    }
}

fn load_conf_file(path: &Path) -> Result<Vec<EnvVar>, EnvStoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    Ok(parse_conf(&text))
}

/// Junta variáveis; nomes posteriores substituem os anteriores. Resultado ordenado.
fn merge_var_lists(lists: impl IntoIterator<Item = Vec<EnvVar>>) -> Vec<EnvVar> {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    for list in lists {
        for var in list {
            map.insert(var.name.clone(), var);
        }
    }
    map.into_values().collect()
}

#[cfg(not(windows))]
fn load_conf_dir(dir: &Path) -> Result<Vec<EnvVar>, EnvStoreError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("conf"))
        })
        .collect();
    files.sort();
    let mut lists = Vec::new();
    for path in files {
        lists.push(load_conf_file(&path)?);
    }
    Ok(merge_var_lists(lists))
}

#[cfg(not(windows))]
fn load_linux_user() -> Result<Vec<EnvVar>, EnvStoreError> {
    #[cfg(test)]
    {
        if std::env::var("EDIT_ENV_USER_CONF").is_ok() {
            return load_conf_file(&user_conf_path());
        }
    }

    // Só ficheiros persistentes em ~/.config/environment.d/ — como HKCU no Windows.
    // NÃO misturar `systemctl --user show-environment` (sessão live: Cursor, GNOME, RUST…)
    // nem o environ do processo: gravar isso em 99-edit.conf polui logins futuros.
    let dir = xdg_config_home().join(ENV_DIR);
    Ok(filter_ephemeral_vars(load_conf_dir(&dir)?))
}

fn filter_ephemeral_vars(vars: Vec<EnvVar>) -> Vec<EnvVar> {
    vars.into_iter()
        .filter(|v| !is_ephemeral_env_name(&v.name))
        .collect()
}

#[cfg(not(windows))]
fn load_linux_system() -> Result<Vec<EnvVar>, EnvStoreError> {
    #[cfg(test)]
    {
        if std::env::var("EDIT_ENV_SYSTEM_CONF").is_ok() {
            return load_conf_file(&system_conf_path());
        }
    }

    let etc_env = load_conf_file(Path::new("/etc/environment"))?;
    let etc_dir = load_conf_dir(&PathBuf::from("/etc").join(ENV_DIR))?;
    Ok(filter_ephemeral_vars(merge_var_lists([etc_env, etc_dir])))
}

fn is_ephemeral_env_name(name: &str) -> bool {
    matches!(
        name,
        "_" | "PWD"
            | "OLDPWD"
            | "SHLVL"
            | "PS1"
            | "PS2"
            | "PS3"
            | "PS4"
            | "COMP_WORDBREAKS"
            | "COLUMNS"
            | "LINES"
            | "NO_COLOR"
            | "TERM"
            | "COLORTERM"
            | "APPDIR"
            | "APPIMAGE"
            | "ARGV0"
            | "CHROME_DESKTOP"
            | "AGENT_TRANSCRIPTS"
    ) || name.starts_with("CARGO_")
        || name.starts_with("RUST_")
        || name.starts_with("CURSOR_")
        || name.starts_with("VSCODE_")
        || name.starts_with("ELECTRON_")
        || name.starts_with("EDIT_ENV_")
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
    // NUNCA chamar `import-environment` sem nomes: importa o environ inteiro do
    // processo (ex. Cursor: NO_COLOR=1, TERM=dumb) para o systemd --user.
    apply_systemd_user_vars(vars);
    Ok(())
}

/// Propaga só as variáveis gravadas via `set-environment` (valores do ficheiro).
#[cfg(not(windows))]
fn apply_systemd_user_vars(vars: &[EnvVar]) {
    if vars.is_empty() {
        return;
    }
    let mut cmd = Command::new("systemctl");
    cmd.arg("--user").arg("set-environment");
    for var in vars {
        if !validate_var_name(&var.name) {
            continue;
        }
        // set-environment rejeita valores com espaço em alguns systemd; passar KEY=VAL.
        cmd.arg(format!("{}={}", var.name, var.value));
    }
    let _ = cmd.status();
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
    // Testes / paths graváveis: escrever directo (sem pkexec).
    if can_write_parent(&path) {
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
        assert_eq!(user_conf_path(), path);

        let vars = vec![EnvVar {
            name: "EDIT_TEST_VAR".into(),
            value: "hello".into(),
        }];
        save_scope(EnvScope::User, &vars).unwrap();
        let loaded = load_conf_file(&path).unwrap();
        assert_eq!(loaded, vars);

        std::env::remove_var("EDIT_ENV_USER_CONF");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn system_roundtrip_via_override() {
        let dir = std::env::temp_dir().join(format!("edit-env-sys-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("99-edit.conf");
        std::env::set_var("EDIT_ENV_SYSTEM_CONF", &path);
        assert_eq!(system_conf_path(), path);

        let vars = vec![EnvVar {
            name: "EDIT_SYS_TEST".into(),
            value: "world".into(),
        }];
        save_scope(EnvScope::System, &vars).unwrap();
        let loaded = load_conf_file(&path).unwrap();
        assert_eq!(loaded, vars);

        std::env::remove_var("EDIT_ENV_SYSTEM_CONF");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn merge_var_lists_later_wins_and_sorts() {
        let a = vec![EnvVar {
            name: "B".into(),
            value: "1".into(),
        }];
        let b = vec![
            EnvVar {
                name: "A".into(),
                value: "x".into(),
            },
            EnvVar {
                name: "B".into(),
                value: "2".into(),
            },
        ];
        let m = merge_var_lists([a, b]);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].name, "A");
        assert_eq!(m[1].name, "B");
        assert_eq!(m[1].value, "2");
    }
}
