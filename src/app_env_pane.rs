//! Abrir/fechar/salvar painel de variáveis de ambiente.

use crate::app::App;
use crate::editor_split::{SplitMode, SplitPane};
use crate::env_editor::{
    install_system_conf_elevated, load_scope, save_scope, validate_var_name, EnvScope, EnvStoreError,
    EnvVar,
};
use crate::env_pane::{EnvPane, EnvPaneAction};
use crate::modal::{ConfirmKind, DialogButtonAction, Modal};
use crate::view_state::InputFocus;

impl App {
    pub fn has_env_pane(&self) -> bool {
        self.editor_split.env_pane.is_some()
    }

    pub fn env_pane_active(&self) -> bool {
        self.has_env_pane() && self.editor_split.focused_pane == SplitPane::Right
    }

    pub fn open_env_pane(&mut self) {
        if self.has_reference_pane() {
            self.close_reference_pane();
        }
        self.sync_active_tab();

        let stash = if let Some(ref pane) = self.editor_split.env_pane {
            pane.stashed_right_tab
        } else {
            self.editor_split.right_tab
        };

        if !self.editor_split.is_active() {
            self.editor_split.mode = SplitMode::Horizontal;
            self.editor_split.left_tab = self.workspace.active_index;
        }

        let scope = EnvScope::User;
        let vars = load_scope(scope).unwrap_or_default();
        self.editor_split.env_pane = Some(EnvPane::new(scope, vars, stash));
        self.editor_split.right_tab = None;
        self.editor_split.focused_pane = SplitPane::Right;
        self.input_focus = InputFocus::Editor;
        self.set_status("Variáveis de ambiente — Usuário (Ctrl+Alt+V)");
    }

    pub fn close_env_pane(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_ref() else {
            return;
        };
        if pane.dirty {
            self.modal = Modal::confirm(
                "Variáveis",
                "Há alterações não salvas. Fechar mesmo assim?",
                ConfirmKind::EnvDiscardClose,
            );
            return;
        }
        self.close_env_pane_force();
    }

    pub fn close_env_pane_force(&mut self) {
        let Some(pane) = self.editor_split.env_pane.take() else {
            return;
        };
        self.editor_split.right_tab = pane.stashed_right_tab;
        self.env_close_hit = None;
        self.env_list_area = None;
        self.env_button_hits.clear();
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
        self.set_status("Variáveis fechadas");
    }

    pub fn env_dispatch_action(&mut self, action: EnvPaneAction) {
        match action {
            EnvPaneAction::None => {}
            EnvPaneAction::Close => self.close_env_pane(),
            EnvPaneAction::Save => self.env_save(),
            EnvPaneAction::Reload => self.env_reload(),
            EnvPaneAction::New => self.env_open_edit_modal(true),
            EnvPaneAction::Edit => self.env_open_edit_modal(false),
            EnvPaneAction::Delete => self.env_confirm_delete(),
            EnvPaneAction::ToggleScope => self.env_toggle_scope(),
            EnvPaneAction::SwitchScope(scope) => self.env_switch_scope(scope),
        }
    }

    fn env_toggle_scope(&mut self) {
        let next = match self
            .editor_split
            .env_pane
            .as_ref()
            .map(|p| p.scope)
            .unwrap_or(EnvScope::User)
        {
            EnvScope::User => EnvScope::System,
            EnvScope::System => EnvScope::User,
        };
        self.env_switch_scope(next);
    }

    fn env_switch_scope(&mut self, scope: EnvScope) {
        let Some(pane) = self.editor_split.env_pane.as_ref() else {
            return;
        };
        if pane.dirty {
            self.pending_env_scope = Some(scope);
            self.modal = Modal::confirm(
                "Variáveis",
                "Há alterações não salvas. Descartar e trocar de escopo?",
                ConfirmKind::EnvDiscardSwitchScope,
            );
            return;
        }
        self.env_switch_scope_force(scope);
    }

    pub fn env_switch_scope_force(&mut self, scope: EnvScope) {
        let Some(pane) = self.editor_split.env_pane.as_mut() else {
            return;
        };
        pane.scope = scope;
        pane.vars = load_scope(scope).unwrap_or_default();
        pane.dirty = false;
        pane.selected = 0;
        pane.scroll = 0;
        self.set_status(format!("Escopo: {}", scope.label()));
    }

    fn env_reload(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_mut() else {
            return;
        };
        let scope = pane.scope;
        match load_scope(scope) {
            Ok(vars) => {
                pane.vars = vars;
                pane.dirty = false;
                pane.ensure_selection();
                self.set_status("Variáveis recarregadas");
            }
            Err(e) => self.set_status(format!("Erro ao carregar: {e}")),
        }
    }

    pub fn env_save(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_ref() else {
            return;
        };
        let scope = pane.scope;
        let vars = pane.vars.clone();
        match save_scope(scope, &vars) {
            Ok(()) => {
                if let Some(pane) = self.editor_split.env_pane.as_mut() {
                    pane.dirty = false;
                }
                self.set_status(format!("Variáveis ({}) salvas", scope.label()));
            }
            Err(EnvStoreError::NeedsElevation) => {
                self.modal = Modal::confirm(
                    "Privilégio necessário",
                    "Variáveis de sistema exigem administrador.\nContinuar para autenticar (pkexec / UAC)?",
                    ConfirmKind::EnvNeedsElevation,
                );
            }
            Err(e) => self.set_status(format!("Erro ao salvar: {e}")),
        }
    }

    pub fn env_save_elevated(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_ref() else {
            return;
        };
        let vars = pane.vars.clone();
        self.pending_env_elevation = true;
        self.pending_env_vars = Some(vars);
    }

    /// Chamado no loop principal: sai da TUI, pkexec, volta.
    pub fn take_pending_env_elevation(&mut self) -> Option<Vec<EnvVar>> {
        if !self.pending_env_elevation {
            return None;
        }
        self.pending_env_elevation = false;
        self.pending_env_vars.take()
    }

    pub fn finish_env_elevation(&mut self, result: Result<(), EnvStoreError>) {
        match result {
            Ok(()) => {
                if let Some(pane) = self.editor_split.env_pane.as_mut() {
                    pane.dirty = false;
                }
                self.set_status("Variáveis de sistema salvas");
            }
            Err(EnvStoreError::ElevationCancelled) => {
                self.set_status("Elevação cancelada");
            }
            Err(e) => self.set_status(format!("Falha na elevação: {e}")),
        }
    }

    fn env_confirm_delete(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_ref() else {
            return;
        };
        let Some(var) = pane.selected_var() else {
            self.set_status("Nenhuma variável selecionada");
            return;
        };
        let name = var.name.clone();
        self.modal = Modal::confirm(
            "Excluir",
            &format!("Excluir a variável {name}?"),
            ConfirmKind::EnvDeleteVar,
        );
    }

    pub fn env_delete_selected(&mut self) {
        let Some(pane) = self.editor_split.env_pane.as_mut() else {
            return;
        };
        if pane.vars.is_empty() {
            return;
        }
        let idx = pane.selected.min(pane.vars.len() - 1);
        let name = pane.vars[idx].name.clone();
        pane.vars.remove(idx);
        pane.dirty = true;
        pane.ensure_selection();
        self.set_status(format!("Removida {name} (não salvo)"));
    }

    fn env_open_edit_modal(&mut self, is_new: bool) {
        let (name, value) = if is_new {
            (String::new(), String::new())
        } else {
            let Some(pane) = self.editor_split.env_pane.as_ref() else {
                return;
            };
            let Some(var) = pane.selected_var() else {
                self.set_status("Nenhuma variável selecionada");
                return;
            };
            (var.name.clone(), var.value.clone())
        };
        self.modal = Modal::env_var_edit(is_new, name, value);
    }

    pub fn env_apply_edit(&mut self, is_new: bool, name: String, value: String) {
        if !validate_var_name(&name) {
            self.set_status("Nome inválido (use A-Z, 0-9, _; comece com letra ou _)");
            return;
        }
        let Some(pane) = self.editor_split.env_pane.as_mut() else {
            return;
        };
        if is_new {
            if pane.vars.iter().any(|v| v.name == name) {
                self.set_status(format!("Já existe {name}"));
                return;
            }
            pane.vars.push(EnvVar { name: name.clone(), value });
            pane.selected = pane.vars.len() - 1;
            pane.dirty = true;
            self.set_status(format!("Adicionada {name} (não salvo)"));
        } else {
            let idx = pane.selected;
            if idx >= pane.vars.len() {
                return;
            }
            let old = pane.vars[idx].name.clone();
            if name != old && pane.vars.iter().any(|v| v.name == name) {
                self.set_status(format!("Já existe {name}"));
                return;
            }
            pane.vars[idx].name = name.clone();
            pane.vars[idx].value = value;
            pane.dirty = true;
            self.set_status(format!("Alterada {name} (não salvo)"));
        }
    }

    pub fn env_title(&self) -> Option<String> {
        self.editor_split.env_pane.as_ref().map(|p| p.title())
    }
}

pub fn run_elevated_install(vars: &[EnvVar]) -> Result<(), EnvStoreError> {
    install_system_conf_elevated(vars)
}
