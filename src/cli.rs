//! Parâmetros de linha de comando do `edit`.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::{
    global_config_path, local_edit_dir, local_workspace_config_path, set_config_path, EditConfig,
};
use crate::session::set_session_root;

const SESSION_SUBDIR: &str = ".edit-session";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LaunchOptions {
    pub clean: bool,
    pub workspace: bool,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    HelpRequested,
    UnknownFlag(String),
    WorkspaceIo(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::HelpRequested => write!(f, "help"),
            CliError::UnknownFlag(flag) => write!(f, "Parâmetro desconhecido: {flag}"),
            CliError::WorkspaceIo(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CliError {}

pub fn parse_args<I>(args: I) -> Result<LaunchOptions, CliError>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut opts = LaunchOptions::default();
    let mut iter = args.into_iter();
    let _program = iter.next();

    for arg in iter {
        let arg = arg.into();
        match arg.as_str() {
            "--clean" => opts.clean = true,
            "--workspace" => opts.workspace = true,
            "--help" | "-h" => return Err(CliError::HelpRequested),
            s if s.starts_with('-') => return Err(CliError::UnknownFlag(s.to_string())),
            _ => opts.files.push(PathBuf::from(arg)),
        }
    }

    Ok(opts)
}

pub fn print_help(program: &str) {
    eprintln!(
        "Uso: {program} [--clean] [--workspace] [arquivo...]\n\
         \n\
         Opções:\n\
           --clean       Limpa dados de workspace (sessão e abas salvas)\n\
           --workspace   Usa configuração local em ./.edit/.edit.workspace\n\
           -h, --help    Exibe esta ajuda\n\
         \n\
         Arquivos listados são abertos em abas ao iniciar (até 10)."
    );
}

/// Configura paths e persiste config limpa **antes** de `App::new`.
pub fn prepare_launch(opts: &LaunchOptions) -> Result<(), CliError> {
    if opts.workspace {
        setup_local_workspace(opts.clean)?;
    } else if opts.clean {
        apply_clean(global_config_path(), global_session_root())?;
    }
    Ok(())
}

fn setup_local_workspace(clean: bool) -> Result<(), CliError> {
    let cwd = env::current_dir().map_err(|e| CliError::WorkspaceIo(e.to_string()))?;
    let edit_dir = local_edit_dir(&cwd);
    let config_path = local_workspace_config_path(&cwd);
    let session_root = edit_dir.join(SESSION_SUBDIR);

    fs::create_dir_all(&edit_dir).map_err(|e| CliError::WorkspaceIo(e.to_string()))?;
    set_config_path(config_path.clone());
    set_session_root(session_root.clone());

    if clean {
        apply_clean(config_path.clone(), session_root)?;
        return Ok(());
    }

    if config_path.is_file() {
        return Ok(());
    }

    let mut config = load_config_template(&global_config_path());
    config.clear_workspace_state();
    config
        .save()
        .map_err(|e| CliError::WorkspaceIo(format!("Erro ao criar .edit.workspace: {e}")))?;
    Ok(())
}

fn apply_clean(config_path: PathBuf, session_root: PathBuf) -> Result<(), CliError> {
    let _ = purge_all_at(&session_root);

    let mut config = if config_path.is_file() {
        load_config_file(&config_path).unwrap_or_default()
    } else {
        EditConfig::default()
    };
    config.clear_workspace_state();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| CliError::WorkspaceIo(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| CliError::WorkspaceIo(e.to_string()))?;
    fs::write(&config_path, json).map_err(|e| CliError::WorkspaceIo(e.to_string()))?;
    Ok(())
}

fn load_config_template(path: &Path) -> EditConfig {
    load_config_file(path).unwrap_or_else(EditConfig::default)
}

fn load_config_file(path: &Path) -> Option<EditConfig> {
    let content = fs::read_to_string(path).ok()?;
    let mut config = serde_json::from_str::<EditConfig>(&content).ok()?;
    config.normalize();
    Some(config)
}

fn global_session_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(SESSION_SUBDIR)))
        .unwrap_or_else(|| PathBuf::from(SESSION_SUBDIR))
}

fn purge_all_at(root: &Path) -> io::Result<()> {
    if root.is_dir() {
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

pub fn canonicalize_open_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flags_and_files() {
        let opts = parse_args(["edit", "--clean", "--workspace", "a.txt", "b.rs"]).unwrap();
        assert!(opts.clean);
        assert!(opts.workspace);
        assert_eq!(
            opts.files,
            vec![PathBuf::from("a.txt"), PathBuf::from("b.rs")]
        );
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(matches!(
            parse_args(["edit", "--foo"]),
            Err(CliError::UnknownFlag(_))
        ));
    }

    #[test]
    fn help_is_requested() {
        assert!(matches!(
            parse_args(["edit", "-h"]),
            Err(CliError::HelpRequested)
        ));
    }
}
