//! Catálogo curado de ficheiros de configuração Linux (painel Editor 2).

mod catalog;
mod pane;

pub use catalog::{expand_path, list_existing_configs, ConfigEntry, ResolvedConfig};
pub use pane::{
    paint_linux_configs_pane, LinuxConfigsAction, LinuxConfigsButtonHit, LinuxConfigsPane,
};
