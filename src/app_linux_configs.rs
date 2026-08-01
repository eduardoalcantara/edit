//! Abrir/fechar painel de configurações Linux e abrir ficheiros no Editor 1.

use crate::app::App;
use crate::editor_split::{SplitMode, SplitPane};
use crate::linux_configs::{LinuxConfigsAction, LinuxConfigsPane};
use crate::view_state::InputFocus;

/// Feature só em Unix/Linux (menu e atalho).
pub fn linux_configs_supported() -> bool {
    cfg!(unix)
}

impl App {
    pub fn has_linux_configs_pane(&self) -> bool {
        self.editor_split.linux_configs.is_some()
    }

    pub fn linux_configs_pane_active(&self) -> bool {
        self.has_linux_configs_pane() && self.editor_split.focused_pane == SplitPane::Right
    }

    pub fn open_linux_configs_pane(&mut self) {
        if !linux_configs_supported() {
            return;
        }
        if self.has_env_pane() {
            self.close_env_pane_force();
        }
        if self.has_reference_pane() {
            self.close_reference_pane();
        }
        self.sync_active_tab();

        let stash = if let Some(ref pane) = self.editor_split.linux_configs {
            pane.stashed_right_tab
        } else {
            self.editor_split.right_tab
        };

        if !self.editor_split.is_active() {
            self.editor_split.mode = SplitMode::Horizontal;
            self.editor_split.left_tab = self.workspace.active_index;
        }

        self.editor_split.linux_configs = Some(LinuxConfigsPane::new(stash));
        self.editor_split.right_tab = None;
        self.editor_split.focused_pane = SplitPane::Right;
        self.input_focus = InputFocus::Editor;
        let n = self
            .editor_split
            .linux_configs
            .as_ref()
            .map(|p| p.items.len())
            .unwrap_or(0);
        self.set_status(format!(
            "Configs Linux — {n} ficheiro(s) (Enter abre no Editor 1; Ctrl+Alt+L)"
        ));
    }

    pub fn close_linux_configs_pane(&mut self) {
        let Some(pane) = self.editor_split.linux_configs.take() else {
            return;
        };
        self.editor_split.right_tab = pane.stashed_right_tab;
        self.linux_configs_close_hit = None;
        self.linux_configs_list_area = None;
        self.linux_configs_button_hits.clear();
        self.editor_split.focused_pane = SplitPane::Left;
        let keep = self
            .editor_split
            .left_tab
            .min(self.workspace.tabs.len().saturating_sub(1));
        if self.editor_split.right_tab.is_none() && self.workspace.tabs.len() < 2 {
            self.editor_split.mode = SplitMode::Off;
        }
        self.editor_split.enforce_focus_invariant();
        self.focus_tab_unchecked(keep);
        if self.editor_split.right_tab.is_some() {
            self.focus_editor_pane(SplitPane::Right);
        }
        self.set_status("Configs Linux fechadas");
    }

    pub fn linux_configs_dispatch(&mut self, action: LinuxConfigsAction) {
        match action {
            LinuxConfigsAction::None => {}
            LinuxConfigsAction::Close => self.close_linux_configs_pane(),
            LinuxConfigsAction::Reload => {
                if let Some(pane) = self.editor_split.linux_configs.as_mut() {
                    pane.reload();
                    let n = pane.items.len();
                    self.set_status(format!("Configs Linux — {n} ficheiro(s)"));
                }
            }
            LinuxConfigsAction::OpenSelected => self.linux_configs_open_selected(),
        }
    }

    fn linux_configs_open_selected(&mut self) {
        let path = self
            .editor_split
            .linux_configs
            .as_ref()
            .and_then(|p| p.selected_item())
            .map(|i| i.path.clone());
        let Some(path) = path else {
            self.set_status("Nenhuma config selecionada");
            return;
        };
        // Nova aba (ou foca se já aberta) no Editor 1 — não substitui a aba atual.
        self.open_path_as_new_tab_in_left(path);
    }
}
