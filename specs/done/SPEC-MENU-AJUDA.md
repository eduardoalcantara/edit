# SPEC — Menu Ajuda

**Status:** done  
**Data:** 2026-06-08  
**Relacionado:** `PROJECT_RULES.md`, `SPEC-MENU-SHELL.md`, `README.md`, `src/menus.rs`, `src/ui/compositor.rs`

---

## 1. Objetivo

Completar a barra de menu com o último item de topo **Ajuda** (`Alt+H` ou `Alt+?` conforme mnemônico escolhido na implementação), no estilo Turbo Vision / Turbo Pascal, com três entradas:

| Item | Propósito |
|------|-----------|
| **Features** | Lista resumida das capacidades do editor (abas, terminal, temas, etc.) |
| **Atalhos** | Referência de teclado agrupada por categoria |
| **Sobre** | Nome do produto, versão, licença, créditos |

Substituir o placeholder atual `F1` → *"Ajuda: em breve"*.

---

## 2. UX

### 2.1. Menu pull-down

- Posição: último item da barra (`… Formatar | Ajuda `).
- Mnemônico sugerido: **`H`** (`Alt+H`), sem conflito com **Editar** (`Alt+E`).
- Itens abrem **modais somente leitura** (scroll se necessário) ou painel de ajuda dedicado — preferir **modal `Dialog`** reutilizando `src/modal/dialog.rs` com botão único **[Fechar]**.

### 2.2. Features

Conteúdo estático (markdown-like em texto puro), seções:

- Editor (ropey, undo, busca, bloco, multi-cursor parcial)
- Workspace (até 10 abas, `Ctrl+Tab`, menu Abas)
- Terminal integrado (PTY, VT100, multi-sessão)
- Temas e Exibir (Turbo Vision, números de linha, word wrap)
- Persistência (`edit.json`, `.edit-session/`)

Fonte: extrair bullets de `README.md` e `PROJECT_STATUS.md` na implementação.

### 2.3. Atalhos

Tabela em texto monoespaçado, agrupada:

| Grupo | Exemplos |
|-------|----------|
| Global | `Ctrl+Q`, `Alt+F4`, `Ctrl+E` (editor), `Ctrl+T`/`Ctrl+'` (terminal) |
| Arquivo | `Ctrl+N/O/S`, `F10`, `F2` |
| Editar | `Ctrl+Z/Y`, `Ctrl+F/H`, `Ctrl+G` (ir para linha) |
| Navegação | `F3`, `F4` abas, `F6` foco, `F7` editor→terminal |
| Abas | `Alt+1…0`, `Ctrl+Shift+W` |

Manter sincronizado com `PROJECT_RULES.md` e `README.md` ao alterar atalhos.

### 2.4. Sobre

- Nome: **edit**
- Versão: de `Cargo.toml` (`env!("CARGO_PKG_VERSION")`) ou constante única
- Descrição uma linha: *Editor TUI estilo Turbo Vision*
- Repositório / autor conforme `README.md`
- Sem telemetria; config local `edit.json`

---

## 3. Arquitetura

| Componente | Mudança |
|------------|---------|
| `src/menus.rs` | `MenuTopItem` Ajuda; `ActionId::HelpFeatures`, `HelpShortcuts`, `HelpAbout` |
| `src/modal/` | `Modal::HelpView { kind, body }` ou três presets leves |
| `src/app.rs` | `dispatch_action` abre modal correspondente |
| `src/ui/compositor.rs` | `F1` → `ActionId::HelpFeatures` ou abre menu Ajuda |
| `src/ui/layers/modal.rs` | render texto longo com scroll (PgUp/PgDn) se corpo > área |

### 3.1. Scroll em modal de ajuda

Se o texto exceder a altura do diálogo:

- `PgUp` / `PgUp` rolam o corpo;
- rodapé mantém help do botão **[Fechar]**.

Alternativa MVP: corpo truncado com *"(ver README)"* — evitar se possível.

---

## 4. Critérios de aceite

1. Barra exibe **Ajuda**; `Alt+H` abre o menu.
2. **Features**, **Atalhos** e **Sobre** abrem modais com conteúdo legível.
3. `F1` abre ajuda (Features ou primeiro item — definir na implementação).
4. Modal fecha com Esc, **[Fechar]** ou clique.
5. Nenhuma regressão nos menus existentes; `cargo test` verde.

---

## 5. Plano de implementação

1. `ActionId` + entradas no `MenuBar::build`.
2. Modal `HelpView` com corpo string estática (ou include_str de `docs/`).
3. Handler em `App::dispatch_action`.
4. `F1` no compositor.
5. Teste: menu contém "Ajuda"; modal About exibe versão.

Estimativa: **2–4 dias**.

---

## 6. Histórico

| Data | Nota |
|------|------|
| 2026-06-08 | Rascunho inicial (PO: último menu; Features / Atalhos / Sobre). |
