//! Configurações persistentes do usuário em `edit.json` (ao lado do executável).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::encoding::{FileEncoding, Tabulation};
use crate::theme::ThemeId;
use crate::view_state::{EditorBorder, EditorMargin, GuideColumn};

const CONFIG_FILE: &str = "edit.json";
const WORKSPACE_CONFIG_FILE: &str = ".edit.workspace";
const LOCAL_EDIT_DIR: &str = ".edit";
const CONFIG_VERSION: u32 = 2;
const MAX_RECENT: usize = 10;

const LEGACY_APP_DIR: &str = ".editor-linux";
const LEGACY_RECENT_DIR: &str = ".edit";
const LEGACY_RECENT_FILE: &str = "recent.json";

static CONFIG_PATH_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditConfig {
    pub version: u32,
    pub arquivo: ArquivoConfig,
    pub exibir: ExibirConfig,
    pub formatar: FormatarConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArquivoConfig {
    pub recentes: Vec<String>,
    #[serde(default)]
    pub abas: AbasConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbasConfig {
    /// **true:** ao sair, descarta abas (não grava `sessao`). **false:** persiste workspace.
    #[serde(default)]
    pub fechar_tudo_ao_sair: bool,
    #[serde(default = "default_true")]
    pub salvar_desfazer_recentes: bool,
    #[serde(default)]
    pub indice_ativo: usize,
    #[serde(default = "default_tab_limit")]
    pub limite: usize,
    #[serde(default)]
    pub sessao: Vec<SessaoTabEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessaoTabEntry {
    pub tab_id: String,
    #[serde(default)]
    pub caminho: Option<String>,
    #[serde(default)]
    pub nome_virtual: Option<String>,
    #[serde(default)]
    pub temporario: bool,
    #[serde(default)]
    pub cursor_linha: usize,
    #[serde(default)]
    pub cursor_coluna: usize,
    #[serde(default = "default_encoding_str")]
    pub encoding: String,
    #[serde(default = "default_tabulation_str")]
    pub tabulacao: String,
    #[serde(default)]
    pub fs_mtime_ms: Option<u64>,
    #[serde(default)]
    pub fs_len: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_tab_limit() -> usize {
    10
}

fn default_encoding_str() -> String {
    "utf-8".to_string()
}

fn default_tabulation_str() -> String {
    "4".to_string()
}

impl Default for AbasConfig {
    fn default() -> Self {
        Self {
            fechar_tudo_ao_sair: false,
            salvar_desfazer_recentes: true,
            indice_ativo: 0,
            limite: 10,
            sessao: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExibirConfig {
    pub zoom: u8,
    pub word_wrap: bool,
    pub mostrar: MostrarConfig,
    pub painel_lateral: bool,
    pub terminal: bool,
    pub rodape: bool,
    pub memoria: bool,
    pub tema: String,
    pub colunas: String,
    pub borda: String,
    pub margem: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MostrarConfig {
    pub simbolos: bool,
    pub espacos: bool,
    pub tabs: bool,
    pub fim_de_linha: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatarConfig {
    pub codificacao: String,
    pub tabulacao: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewSettingsSnapshot {
    pub zoom: u8,
    pub word_wrap: bool,
    pub show_symbols: bool,
    pub show_spaces: bool,
    pub show_tabs: bool,
    pub show_eol: bool,
    pub side_panel: bool,
    pub terminal: bool,
    pub footer_visible: bool,
    pub show_memory: bool,
    pub guide_column: GuideColumn,
    pub margin: EditorMargin,
    pub border: EditorBorder,
    pub theme: ThemeId,
}

pub fn config_from_view(
    recent_paths: &[PathBuf],
    view: &crate::view_state::ViewState,
    encoding: FileEncoding,
    tabulation: Tabulation,
    abas: AbasConfig,
) -> EditConfig {
    EditConfig {
        version: CONFIG_VERSION,
        arquivo: ArquivoConfig {
            recentes: recent_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            abas,
        },
        exibir: ExibirConfig {
            zoom: view.zoom,
            word_wrap: view.word_wrap,
            mostrar: MostrarConfig {
                simbolos: view.show_symbols,
                espacos: view.show_spaces,
                tabs: view.show_tabs,
                fim_de_linha: view.show_eol,
            },
            painel_lateral: view.side_panel,
            terminal: view.terminal,
            rodape: view.footer_visible,
            memoria: view.show_memory,
            tema: theme_to_str(view.theme).to_string(),
            colunas: guide_column_to_str(view.guide_column).to_string(),
            borda: border_to_str(view.border).to_string(),
            margem: margin_to_str(view.margin).to_string(),
        },
        formatar: FormatarConfig {
            codificacao: encoding_to_str(encoding).to_string(),
            tabulacao: tabulation_to_str(tabulation).to_string(),
        },
    }
}

impl Default for EditConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            arquivo: ArquivoConfig {
                recentes: Vec::new(),
                abas: AbasConfig::default(),
            },
            exibir: ExibirConfig {
                zoom: 1,
                word_wrap: false,
                mostrar: MostrarConfig {
                    simbolos: false,
                    espacos: false,
                    tabs: false,
                    fim_de_linha: false,
                },
                painel_lateral: false,
                terminal: false,
                rodape: true,
                memoria: true,
                tema: theme_to_str(ThemeId::ClassicBlue).to_string(),
                colunas: "ilimitado".to_string(),
                borda: "visivel".to_string(),
                margem: "sem".to_string(),
            },
            formatar: FormatarConfig {
                codificacao: encoding_to_str(FileEncoding::Utf8).to_string(),
                tabulacao: tabulation_to_str(Tabulation::Spaces4).to_string(),
            },
        }
    }
}

impl EditConfig {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<EditConfig>(&content) {
                    config.normalize();
                    return config;
                }
            }
        }

        let mut config = Self::default();
        if let Some(recentes) = migrate_legacy_recent_paths() {
            config.arquivo.recentes = recentes;
            let _ = config.save();
        }
        config
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(path, json)
    }

    pub fn default_encoding(&self) -> FileEncoding {
        parse_encoding(&self.formatar.codificacao)
    }

    pub fn default_tabulation(&self) -> Tabulation {
        parse_tabulation(&self.formatar.tabulacao)
    }

    pub fn view_settings(&self) -> ViewSettingsSnapshot {
        ViewSettingsSnapshot {
            zoom: self.exibir.zoom.clamp(1, 3),
            word_wrap: self.exibir.word_wrap,
            show_symbols: self.exibir.mostrar.simbolos,
            show_spaces: self.exibir.mostrar.espacos,
            show_tabs: self.exibir.mostrar.tabs,
            show_eol: self.exibir.mostrar.fim_de_linha,
            side_panel: self.exibir.painel_lateral,
            terminal: self.exibir.terminal,
            footer_visible: self.exibir.rodape,
            show_memory: self.exibir.memoria,
            guide_column: parse_guide_column(&self.exibir.colunas),
            margin: parse_margin(&self.exibir.margem),
            border: parse_border(&self.exibir.borda),
            theme: parse_theme(&self.exibir.tema),
        }
    }

    pub fn recent_paths(&self) -> Vec<PathBuf> {
        self.arquivo
            .recentes
            .iter()
            .map(PathBuf::from)
            .collect()
    }

    pub(crate) fn normalize(&mut self) {
        self.version = CONFIG_VERSION;
        self.arquivo.recentes.truncate(MAX_RECENT);
        self.arquivo.abas.limite = self.arquivo.abas.limite.clamp(1, 10);
        self.exibir.zoom = self.exibir.zoom.clamp(1, 3);
    }

    /// Remove abas persistidas — equivalente a iniciar sem sessão de workspace.
    pub fn clear_workspace_state(&mut self) {
        self.arquivo.abas.sessao.clear();
        self.arquivo.abas.indice_ativo = 0;
        self.arquivo.abas.fechar_tudo_ao_sair = false;
    }
}

pub fn global_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(CONFIG_FILE)))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
}

pub fn local_workspace_config_path(base_dir: &Path) -> PathBuf {
    base_dir.join(LOCAL_EDIT_DIR).join(WORKSPACE_CONFIG_FILE)
}

pub fn local_edit_dir(base_dir: &Path) -> PathBuf {
    base_dir.join(LOCAL_EDIT_DIR)
}

pub fn set_config_path(path: PathBuf) {
    let mut guard = CONFIG_PATH_OVERRIDE.lock().expect("config path lock");
    *guard = Some(path);
}

pub fn config_path() -> PathBuf {
    if let Ok(guard) = CONFIG_PATH_OVERRIDE.lock() {
        if let Some(path) = guard.as_ref() {
            return path.clone();
        }
    }
    global_config_path()
}

#[cfg(test)]
pub fn set_config_path_for_tests(path: PathBuf) {
    let mut guard = CONFIG_PATH_OVERRIDE.lock().expect("config path lock");
    *guard = Some(path);
}

#[cfg(test)]
pub fn clear_config_path_override() {
    let mut guard = CONFIG_PATH_OVERRIDE.lock().expect("config path lock");
    *guard = None;
}

fn migrate_legacy_recent_paths() -> Option<Vec<String>> {
    for dir in [LEGACY_RECENT_DIR, LEGACY_APP_DIR] {
        let path = PathBuf::from(dir).join(LEGACY_RECENT_FILE);
        if let Some(paths) = read_legacy_recent_file(&path) {
            let strings: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            return Some(strings);
        }
    }
    None
}

fn read_legacy_recent_file(path: &Path) -> Option<Vec<PathBuf>> {
    let content = fs::read_to_string(path).ok()?;
    parse_legacy_recent_json(&content)
}

fn parse_legacy_recent_json(content: &str) -> Option<Vec<PathBuf>> {
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

fn theme_to_str(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "escuro",
        ThemeId::Light => "claro",
        ThemeId::ClassicBlue => "azul_classico",
        ThemeId::Matrix => "matrix",
    }
}

fn parse_theme(value: &str) -> ThemeId {
    match value.trim().to_ascii_lowercase().as_str() {
        "escuro" | "dark" => ThemeId::Dark,
        "claro" | "light" => ThemeId::Light,
        "azul_classico" | "classic_blue" | "azul clássico" => ThemeId::ClassicBlue,
        "matrix" => ThemeId::Matrix,
        _ => ThemeId::ClassicBlue,
    }
}

fn guide_column_to_str(column: GuideColumn) -> &'static str {
    match column {
        GuideColumn::Col80 => "80",
        GuideColumn::Col120 => "120",
        GuideColumn::Col160 => "160",
        GuideColumn::Unlimited => "ilimitado",
    }
}

fn parse_guide_column(value: &str) -> GuideColumn {
    match value.trim() {
        "80" => GuideColumn::Col80,
        "120" => GuideColumn::Col120,
        "160" => GuideColumn::Col160,
        _ => GuideColumn::Unlimited,
    }
}

fn border_to_str(border: EditorBorder) -> &'static str {
    match border {
        EditorBorder::Visible => "visivel",
        EditorBorder::Hidden => "invisivel",
    }
}

fn parse_border(value: &str) -> EditorBorder {
    match value.trim().to_ascii_lowercase().as_str() {
        "invisivel" | "hidden" => EditorBorder::Hidden,
        _ => EditorBorder::Visible,
    }
}

fn margin_to_str(margin: EditorMargin) -> &'static str {
    match margin {
        EditorMargin::None => "sem",
        EditorMargin::OneLine => "uma_linha",
        EditorMargin::TwoLines => "duas_linhas",
    }
}

fn parse_margin(value: &str) -> EditorMargin {
    match value.trim().to_ascii_lowercase().as_str() {
        "uma_linha" | "uma linha" => EditorMargin::OneLine,
        "duas_linhas" | "duas linhas" => EditorMargin::TwoLines,
        _ => EditorMargin::None,
    }
}

pub fn encoding_to_config_str(encoding: FileEncoding) -> String {
    encoding_to_str(encoding).to_string()
}

pub fn tabulation_to_config_str(tabulation: Tabulation) -> String {
    tabulation_to_str(tabulation).to_string()
}

fn encoding_to_str(encoding: FileEncoding) -> &'static str {
    match encoding {
        FileEncoding::Utf8 => "utf-8",
        FileEncoding::Utf8NoBom => "utf-8_sem_bom",
        FileEncoding::Utf16Le => "utf-16_le",
        FileEncoding::Utf16Be => "utf-16_be",
        FileEncoding::Iso88591 => "iso-8859-1",
        FileEncoding::Ansi => "ansi",
    }
}

fn parse_encoding(value: &str) -> FileEncoding {
    match value.trim().to_ascii_lowercase().as_str() {
        "utf-8_sem_bom" | "utf8_sem_bom" => FileEncoding::Utf8NoBom,
        "utf-16_le" | "utf16_le" => FileEncoding::Utf16Le,
        "utf-16_be" | "utf16_be" => FileEncoding::Utf16Be,
        "iso-8859-1" | "iso8859-1" => FileEncoding::Iso88591,
        "ansi" => FileEncoding::Ansi,
        _ => FileEncoding::Utf8,
    }
}

fn tabulation_to_str(tabulation: Tabulation) -> &'static str {
    match tabulation {
        Tabulation::Spaces2 => "2",
        Tabulation::Spaces4 => "4",
        Tabulation::Spaces8 => "8",
        Tabulation::TabLiteral => "literal",
    }
}

fn parse_tabulation(value: &str) -> Tabulation {
    match value.trim().to_ascii_lowercase().as_str() {
        "2" => Tabulation::Spaces2,
        "8" => Tabulation::Spaces8,
        "literal" | "tab" => Tabulation::TabLiteral,
        _ => Tabulation::Spaces4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct TestConfigGuard {
        _lock: MutexGuard<'static, ()>,
        path: PathBuf,
    }

    impl TestConfigGuard {
        fn new(name: &str) -> Self {
            let lock = TEST_LOCK.lock().unwrap();
            let path = std::env::temp_dir().join(format!("edit-config-{name}.json"));
            let _ = fs::remove_file(&path);
            set_config_path_for_tests(path.clone());
            Self { _lock: lock, path }
        }
    }

    impl Drop for TestConfigGuard {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            clear_config_path_override();
        }
    }

    #[test]
    fn saves_and_loads_roundtrip() {
        let _guard = TestConfigGuard::new("roundtrip");
        let mut config = EditConfig::default();
        config.exibir.tema = "matrix".to_string();
        config.arquivo.recentes = vec!["/tmp/a.txt".to_string()];
        config.formatar.tabulacao = "8".to_string();
        config.save().unwrap();

        let loaded = EditConfig::load();
        assert_eq!(loaded.exibir.tema, "matrix");
        assert_eq!(loaded.arquivo.recentes, vec!["/tmp/a.txt"]);
        assert_eq!(loaded.formatar.tabulacao, "8");
    }

    #[test]
    fn migrates_legacy_recent_json() {
        let _guard = TestConfigGuard::new("legacy");
        let legacy_dir = PathBuf::from(LEGACY_RECENT_DIR);
        fs::create_dir_all(&legacy_dir).unwrap();
        let legacy_file = legacy_dir.join(LEGACY_RECENT_FILE);
        fs::write(&legacy_file, r#"["/tmp/exemplo.txt"]"#).unwrap();

        let config = EditConfig::load();
        assert_eq!(config.arquivo.recentes, vec!["/tmp/exemplo.txt"]);
        assert!(config_path().exists());

        let _ = fs::remove_file(&legacy_file);
        let _ = fs::remove_dir(legacy_dir);
    }

    #[test]
    fn view_settings_from_config() {
        let _guard = TestConfigGuard::new("apply");
        let mut config = EditConfig::default();
        config.exibir.word_wrap = true;
        config.exibir.tema = "matrix".to_string();
        config.formatar.codificacao = "ansi".to_string();
        config.formatar.tabulacao = "literal".to_string();

        let view = config.view_settings();
        assert!(view.word_wrap);
        assert_eq!(view.theme, ThemeId::Matrix);
        assert_eq!(config.default_encoding(), FileEncoding::Ansi);
        assert_eq!(config.default_tabulation(), Tabulation::TabLiteral);
    }
}
