# SPEC — Editor split view (duas abas visíveis)

**Status:** done  
**Data:** 2026-06-08  
**Implementado:** 2026-06-08 (branch `editor-split`)  
**Atalhos PO:** `Ctrl+1` (único/esquerda), `Ctrl+2` (split/direita) — em vez de `Ctrl+E` do rascunho inicial  
**Relacionado:** `PROJECT_RULES.md`, `SPEC-MULTPLOS-ARQUIVOS.md`, `SPEC-UX-FIDELIDADE-TURBO-VISION.md`, `src/workspace/`, `src/ui/layout.rs`, `src/ui/layers/editor.rs`

---

## 1. Objetivo

Permitir **dois arquivos (abas) visíveis ao mesmo tempo** na área do editor, em molduras lado a lado — como no Turbo C++/Turbo Vision (janelas tiled), para comparar código, copiar trechos e ler header/implementação sem alternar aba a cada momento.

**Escopo desta spec (MVP):**

- Split **horizontal** (esquerda | direita), proporção inicial **50/50**.
- Cada painel mostra **uma aba** do workspace (até 10 abas já abertas).
- **Um painel com foco** recebe teclado; o outro permanece visível mas inativo (borda simples vs. dupla, estilo TV).
- Fechar split volta ao modo **aba única** (painel secundário oculto).

**Fora do escopo (futuro):**

- Mais de 2 painéis; split vertical; divisor arrastável; mesma aba em dois painéis; painéis Message/Project como no screenshot do Turbo C++.

---

## 2. Motivação

| Hoje | Com split |
|------|-----------|
| `Ctrl+Tab` / menu Abas troca o conteúdo na tela inteira | Dois arquivos permanecem visíveis |
| `Tab` já guarda `Editor` + `Document` por aba | Reaproveita modelo de workspace |
| Terminal inferior compete por altura vertical | Split horizontal preserva altura útil do editor |

---

## 3. Conceitos

| Conceito | Significado |
|----------|-------------|
| **Painel** | Retângulo do editor com moldura, título `[ arquivo ]` e viewport de texto |
| **Painel primário** | Painel com foco de teclado; sincronizado com `App.editor` / `App.document` |
| **Painel secundário** | Segunda aba visível; renderizada a partir de `Tab.editor` ou segundo buffer quente |
| **Modo split** | `SplitMode::Off` \| `SplitMode::Horizontal` |
| **Aba do painel** | Índice em `Workspace.tabs` exibido em cada painel |

Regra: **a mesma aba não pode ocupar os dois painéis** ao mesmo tempo.

---

## 4. UX / comportamento

### 4.1. Ativação

- Menu **Exibir** → **Dividir editor** (toggle) ou submenu **Horizontal**.
- Atalho: menu **Exibir → Dividir editor** (toggle); **`Ctrl+1`** / **`Ctrl+2`** (ver `PROJECT_RULES.md`).
- Ao ativar com uma aba: painel esquerdo = aba ativa; direito = próxima aba na lista ou vazio com prompt “Selecione aba…”.

### 4.2. Foco

- Clique no painel → foco + sincroniza `active_index` do workspace para a aba daquele painel.
- `F6` continua alternando Editor ↔ Terminal; dentro do editor em split, `F6` não troca painel (usar mouse ou atalho dedicado, ex. `Ctrl+E` cicla foco esquerda/direita).

### 4.3. Molduras (fidelidade TV)

| Estado | Borda |
|--------|-------|
| Painel com foco | Dupla (`PanelBorder::Double`) |
| Painel sem foco | Simples (`PanelBorder::Plain`) |
| Título | `[ nome ]` ou `[ Novo2* ]` por aba |

### 4.4. Rodapé

- `Pos`, `Tam`, encoding e tabulação refletem a **aba do painel com foco** (comportamento atual preservado).

### 4.5. Terminal inferior

- Split divide apenas a faixa **editor** acima do divisor do terminal; layout existente (`UiLayout::compute`) ganha subdivisão horizontal de `editor_content` / `shell` antes da reserva do PTY.

### 4.6. Desativação

- Toggle off em Exibir → volta a aba única (painel primário); estado do secundário não perde dirty (aba continua no workspace).

---

## 5. Modelo de dados

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    #[default]
    Off,
    Horizontal,
}

pub struct EditorSplit {
    pub mode: SplitMode,
    /// Índice da aba no painel esquerdo.
    pub left_tab: usize,
    /// Índice da aba no painel direito (`None` = painel vazio).
    pub right_tab: Option<usize>,
    /// Qual lado tem foco (`Left` | `Right`).
    pub focused_pane: SplitPane,
}
```

Persistência em `edit.json` → `exibir.split_editor` (opcional fase 2 desta spec):

```json
"exibir": {
  "split_editor": "off" | "horizontal",
  "split_right_tab": 2
}
```

---

## 6. Arquitetura técnica

### 6.1. Problema atual

Hoje existe **um** `App.editor` + `App.document` “quentes”; `focus_tab` faz `sync_active_tab` + `replace_content`. Só uma aba é editável por frame.

### 6.2. Abordagem recomendada (MVP)

1. **Manter** `App.editor` como motor do painel com foco.
2. **Painel inativo:** renderizar com `Editor::draw` parametrizado por referência a `Tab.editor` **somente leitura** (sem cursor), ou clonar viewport mínimo.
3. Ao **clicar** no painel inativo: `focus_tab` + trocar `focused_pane` (mesmo fluxo de troca de aba atual).
4. **Layout:** em `UiLayout` ou helper, `split_shell_horizontally(shell, ratio) -> (left, right)`.
5. **Input:** `EditorLayer::on_mouse` detecta qual metade do `shell` foi clicada antes de delegar ao mouse do editor.

### 6.3. Módulos impactados

| Módulo | Mudança |
|--------|---------|
| `src/view_state.rs` | `SplitMode`, `EditorSplit` |
| `src/config.rs` | persistência opcional |
| `src/ui/layout.rs` | rects `editor_left` / `editor_right` |
| `src/ui/layers/editor.rs` | paint duplo; hit-test de foco |
| `src/app_workspace.rs` | sync ao trocar painel sem perder dirty |
| `src/editor/render.rs` | aceitar `&EditorEngine` externo (já quase isolado) |
| `src/menus.rs` | item Exibir |
| `PROJECT_RULES.md` | atalho `Ctrl+E` |

### 6.4. Riscos

- Duas viewports estreitas em terminais 80×25 — documentar largura mínima (~40 colunas por painel).
- Word wrap + números de linha em painel estreito — reutilizar lógica atual sem duplicar.
- Race de sync se editar muito rápido ao trocar painel — sempre `sync_active_tab` antes de trocar `active_index`.

---

## 7. Critérios de aceite

1. Com 2+ abas abertas, ativar split mostra **dois arquivos** lado a lado com títulos corretos.
2. Digitar altera só o painel com foco; dirty aparece no título (`*`).
3. Clique no outro painel troca foco e aba ativa do workspace.
4. Com terminal visível, split não sobrepõe o painel PTY.
5. Desativar split restaura layout de aba única sem perda de conteúdo.
6. `cargo test` verde; testes de layout para rects não sobrepostos.

---

## 8. Plano de implementação sugerido

1. Tipos `SplitMode` / `EditorSplit` + toggle no menu (sem persistência).
2. Layout horizontal 50/50 em `UiLayout`.
3. `EditorLayer::paint` — segunda chamada de render para aba secundária.
4. Mouse + foco de painel.
5. Borda ativa/inativa.
6. Persistência `edit.json` + testes.
7. Atalho `Ctrl+E`.

Estimativa: **1–2 semanas** (MVP); +1 semana com persistência e polish TV.

---

## 9. Histórico

| Data | Nota |
|------|------|
| 2026-06-08 | Implementado MVP: split 50/50, borda dupla, Ctrl+1/Ctrl+2, persistência `edit.json`. |
