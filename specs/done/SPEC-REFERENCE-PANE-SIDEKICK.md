# SPEC — Painel de referência e UX SideKick

**Status:** done  
**Data:** 2026-06-09  
**Implementado:** 2026-06-09 (integrado em `main`)  
**Autor:** Perplexity AI

## Objetivo

Trazer três evoluções inspiradas no Borland SideKick:

1. Ajuda e tabela ASCII no **painel direito** do split (conteúdo virtual read-only).
2. **Sair da tela** — suspende a TUI e volta ao prompt do terminal (**somente Unix/Linux**).
3. **Mnemônicos entre parênteses** em terminais monocromáticos.

## 1. Painel de referência

### Modelo

- `ReferencePane` em `src/reference_pane.rs` — **não** é aba em `workspace.tabs`.
- Campos em `EditorSplit`: `reference: Option<ReferencePane>`, `stashed_right_tab`.
- Tipos: `HelpFeatures`, `HelpShortcuts`, `AsciiTable`.

### Abrir

1. `sync_active_tab()` (ignorado se referência já em foco).
2. Ativa split horizontal se necessário (mesmo com uma aba).
3. Guarda `right_tab` em `stashed_right_tab` (preserva stash original ao trocar página de ajuda).
4. Carrega texto de `help_content.rs` em editor read-only; foco no painel direito (swap com `app.editor`).

### Fechar

- Botão **Fechar** na borda, `Esc`, ou `F` no painel.
- Restaura `right_tab` do stash; desativa split se só restar uma aba sem conteúdo direito.

### Exclusões

| Subsistema | Comportamento |
|------------|---------------|
| `document.path()` | `None` na referência |
| Menu Abas | não listada |
| `build_abas_config` / sessão | não serializa |
| Dirty / save | ignorado (`read_only`) |

## 2. Sair da tela

- Menu **Arquivo → Sair da tela** e atalho **`Ctrl+Shift+Alt+E`** — **visíveis apenas em Unix/Linux** (`src/platform.rs`: `terminal_suspend_to_shell_supported()`).
- Windows: item oculto e atalho ignorado (sem daemon TSR / SIGTSTP).
- `TerminalGuard::suspend` / `resume` em `main.rs`.
- Loop suspenso captura apenas o chord de retorno.
- Diferente de **Sair** (`Ctrl+Q`): processo continua até quit explícito.

## 3. Modo monocromático

- Config: `exibir.mnemonico_parenteses`: `auto` | `ligado` | `desligado`.
- Detecção auto: `NO_COLOR`, `TERM=dumb`, heurística `COLORTERM` / `TERM`.
- Render: topo ` Arquivo (F) `; dropdown `(N)ovo`; modais `(O)K`; Fechar `(F)echar`.

## 4. Abrir com documento dirty

- Diálogo **Salvar / Não Salvar / Ignorar / Cancelar** (`OPEN_UNSAVED_FULL` em `modal/buttons.rs`).
- Botão **Ignorar** oculto quando as 10 abas estão dirty (`OPEN_UNSAVED_NO_IGNORE`).
- Recentes e browser FS preservam path alvo via `pending_open_path`.

## Arquivos principais

| Área | Arquivos |
|------|----------|
| Referência | `reference_pane.rs`, `app_reference_pane.rs`, `editor_split.rs`, `ui/layers/editor.rs` |
| Read-only | `editor/engine.rs`, `editor/mod.rs`, `editor/commands.rs` |
| Suspend | `main.rs`, `platform.rs`, `app.rs`, `ui/compositor.rs`, `input/keyboard.rs`, `menus.rs` |
| Monocromático | `view_state.rs`, `config.rs`, `menus.rs`, `modal/dialog.rs` |
| Abrir dirty | `modal/buttons.rs`, `modal/mod.rs`, `app.rs` |

## Testes

- `reference_pane::tests::reference_editor_is_read_only`
- `app_workspace`: stash/restore, typing bloqueado, split com uma aba
- `menus`: mnemônico `(F)`, item Tabela ASCII, Sair da tela condicional
- `config`: `mnemonico_parenteses`
- `view_state`: `resolve_paren_mnemonics`, `format_item_label_paren`

Suite completa: `cargo test -- --test-threads=1`.
