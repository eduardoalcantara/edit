# SPEC — Múltiplos arquivos (workspace / abas)

**Status:** to-do  
**Data:** 2026-06-07  
**Relacionado:** `PROJECT_RULES.md` v2.0, `src/config.rs` (`edit.json`), `src/app.rs`

---

## 1. Objetivo

Permitir até **10 arquivos abertos simultaneamente** no editor, alternando entre eles via **menu Abas**, atalhos de teclado e (futuro) barra de abas visual. O documento único atual (`App.document` + `App.editor`) evolui para um **workspace** com várias abas, cada uma encapsulando estado completo de edição.

**Fase 1 (esta spec):** seleção de abas **somente pelo menu Abas** — sem barra de abas horizontal. A borda do editor continua exibindo o título da aba ativa: `[ nome ]` ou `[ Novo3* ]`.

**Fase 2 (futuro, fora do escopo imediato):** barra de abas abaixo do menu com `[ ativa ]` / `( inativa )` e clique de mouse.

---

## 2. Conceitos e glossário

| Conceito | Significado |
|----------|-------------|
| **Aba** | Unidade de edição: conteúdo, cursor, metadados, dirty, encoding etc. |
| **Aba ativa** | Aba em foco; única cujo conteúdo aparece no editor e no rodapé. |
| **Arquivo → Recentes** | Histórico de arquivos **fechados** pelo usuário (até 10). Ao fechar uma aba com path, o arquivo vai para o **topo** dessa lista. Comportamento universal — **não muda** em relação ao produto atual. |
| **Menu Abas** | Lista dos arquivos **abertos** no momento (até 10), com atalhos e ações de workspace. **Não** é lista de fechados. |
| **Documento Sem Título** | Aba sem `filepath` no disco; nome virtual `Novo`, `Novo1`, `Novo2`… |
| **Pristine** | Aba sem alterações desde último save/abertura/criação (`EMPTY_DOCUMENT_TEXT` para novos). |
| **Sessão FS** | Pasta `.edit-session/` ao lado do executável: dados por aba (conteúdo temp, undo/redo, metadados). **Não** vai dentro de `edit.json`. |
| **`tab_id`** | Identificador estável da aba na sessão (UUID ou hash); nome da subpasta em `.edit-session/tabs/{tab_id}/`. |

---

## 3. Menus

### 3.1. Arquivo (sem mudança de propósito em Recentes)

- **Recentes:** arquivos que o usuário **fechou** e pode reabrir sem procurar no disco. Fechar aba com path → entra no topo de `arquivo.recentes` (máx. 10). Abrir da lista foca/cria aba se ainda não estiver aberta.
- Demais itens (Novo, Abrir, Salvar, **Salvar Todos** (`Ctrl+Alt+S`), Salvar Como, Fechar, Sair) operam no contexto da **aba ativa**, salvo **Salvar Todos** (todas as dirty).

### 3.2. Abas (menu de topo novo)

Menu separado de Arquivo; mnemônico **`Alt+S`** (Abas).

| Item | Comportamento |
|------|----------------|
| Lista dinâmica (até 10) | Um item por aba aberta. Rótulo = nome do arquivo ou `NovoN`. Asterisco se dirty. Item da aba ativa marcado (radio/check). **Clique ou Enter** troca foco imediatamente. |
| Atalhos `Alt+1` … `Alt+0` | Foca aba na posição 1–10 da lista atual (mesma ordem do menu). |
| **Fechar Todos** | Atalho **`Ctrl+Shift+W`**. Mesmo fluxo de confirmação de **Sair** (seção 7.2): percorre abas dirty na **ordem do menu** (topo → fim). |
| **Fechar tudo ao sair** | Toggle (estilo TV `√`). Quando **ativado**, persiste lista de abas e metadados em `edit.json` + artefatos em `.edit-session/` (seção 8). |
| **Salvar desfazer recentes** | Toggle (estilo TV `√`). Persiste undo/redo em `.edit-session/` ao sair (requer “fechar tudo ao sair”). Help: *“Mantém até 5+ passos de desfazer por aba entre sessões; desligue para economizar disco.”* **Ao desmarcar:** modal de confirmação (3.2.1). |
| **Ordenar por** (submenu) | Reordena abas **abertas** na sessão. **Não** persiste estratégia; só a ordem resultante (se “fechar tudo ao sair” estiver ativo). Opções abaixo. |

#### 3.2.1. Modal ao desmarcar “Salvar desfazer recentes”

Quando o usuário **desliga** o toggle (menu ou ação equivalente):

1. Se **não** existir `undo.json`/`redo.json` em `.edit-session/` → desligar silenciosamente e persistir toggle em `edit.json`.
2. Se existir dados de undo no disco → modal **`[ Apagar desfazer ]` [ Manter no disco ] [ Cancelar ]`** (título na borda: `[ Desfazer ]`).
   - **Apagar desfazer:** `SessionStore::purge_all_undo()` — remove todos os `undo.json`/`redo.json`; desliga toggle; grava `edit.json`.
   - **Manter no disco:** desliga toggle **sem** apagar arquivos (não serão usados na restauração enquanto toggle off); próximo shutdown com toggle off também não regrava undo.
   - **Cancelar:** toggle **permanece ligado** (`√`); nenhuma alteração.

Help no rodapé do modal: *“Apaga os passos de desfazer salvos ao lado do executável para liberar espaço.”*

**Submenu Ordenar por:**

| Estratégia | Critério |
|------------|----------|
| Nome de Arquivo | Ordem alfabética do rótulo exibido (nome de arquivo ou `NovoN`). |
| Caminho | Ordem alfabética do path completo; abas sem path agrupadas ao final (ordem estável entre `NovoN`). |
| Abertos Primeiro | Menor `opened_at` primeiro (mais antiga no topo). |
| Abertos por Último | Maior `opened_at` primeiro (mais recente no topo). |
| Status | Dirty no topo; dentro de cada grupo, ordem estável atual. |

Após ordenar, **`active_tab_index` deve ser recalculado** para manter a mesma aba lógica em foco.

**Ordem padrão ao abrir/criar:** nova aba ou arquivo aberto entra no **topo** da lista (índice 0). Ordenação manual não se reaplica automaticamente.

---

## 4. Interface visual (fase 1)

### 4.1. Borda do editor

- Mantém título `[ nome ]` da aba ativa (regra atual de `PROJECT_RULES`).
- Dirty: asterisco no título (`[ main.rs* ]`, `[ Novo2* ]`).
- Troca de aba atualiza título, conteúdo, cursor, encoding/tab no rodapé.

### 4.2. Barra de abas (fase 2 — referência)

Reservado para implementação futura:

- Abaixo da barra de menus; sem scroll horizontal (máx. 10 abas).
- Ativa: `[ arquivo.rs ]`; inativa: `( utils.rs )`; dirty: `*`.
- Clique troca foco (consistente com mouse no menu Abas).

---

## 5. Estruturas de dados

### 5.1. Runtime

```rust
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabSortStrategy {
    FileName,
    FilePath,
    OpenedFirst,
    OpenedLast,
    Status,
}

/// Estado completo de uma aba (encapsula o que hoje está em Document + Editor).
pub struct Tab {
    /// `None` = documento só em memória (Sem Título / NovoN).
    pub filepath: Option<PathBuf>,
    /// Nome exibido: file_name ou "Novo", "Novo1", "Novo2"…
    pub display_name: String,
    /// Conteúdo e edição (rope, cursor, viewport, seleção, histórico interno do engine).
    pub editor: Editor,
    /// Metadados persistidos: encoding, tabulação, baseline saved_content.
    pub document: Document,
    pub opened_at: Instant,
    /// Snapshot FS para detecção de alteração externa (mtime/size; extensível).
    pub fs_snapshot: Option<FileSnapshot>,
    /// Identificador da pasta em `.edit-session/tabs/{tab_id}/`.
    pub session_id: String,
    /// Se true, conteúdo também em `.edit-session/.../content.tmp`.
    pub is_temp_file: bool,
}

pub struct FileSnapshot {
    pub modified: SystemTime,
    pub len: u64,
}

pub struct Workspace {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    pub max_tabs: usize, // padrão 10
    pub next_novo_index: u32, // incremento para Novo1, Novo2…
}

pub enum WorkspaceAction {
    Ok,
    /// Abrir/evict bloqueado: aba no índice precisa de decisão do usuário.
    PromptSaveRequired { tab_index: usize, reason: PromptReason },
}

pub enum PromptReason {
    CloseTab,
    /// Aba no **final da fila** (último índice do menu) removida por limite de 10.
    EvictTail,
    Quit,
    CloseAll,
}
```

**Notas:**

- `TabStatus` explícito **não é necessário** — derivar dirty via `Document::is_dirty(&editor.content_string())`.
- **Find** fica fora desta spec (ainda incompleto).
- **Undo/redo:** pilha isolada por aba; persistência opcional em `.edit-session/` — seções **6.9** e **8.4**.
- Cada aba possui **próprio** `Editor`/`Document`; ao trocar aba, o backend faz swap do par ativo exposto ao `App` ou delega leitura ao `Workspace`.

### 5.2. Persistência (`edit.json`)

Estender o modelo existente (não substituir `exibir` / `formatar`). Nova seção em **`arquivo.abas`**:

```json
{
  "version": 2,
  "arquivo": {
    "recentes": ["C:\\proj\\foo.txt"],
    "abas": {
      "fechar_tudo_ao_sair": false,
      "salvar_desfazer_recentes": true,
      "indice_ativo": 0,
      "limite": 10,
      "sessao": [
        {
          "tab_id": "a1b2c3d4-…",
          "caminho": "C:\\proj\\foo.txt",
          "nome_virtual": null,
          "temporario": false,
          "cursor_linha": 12,
          "cursor_coluna": 4,
          "encoding": "utf-8",
          "tabulacao": "4",
          "fs_mtime_ms": 1717785600000,
          "fs_len": 4096
        },
        {
          "tab_id": "e5f6…",
          "caminho": null,
          "nome_virtual": "Novo2",
          "temporario": true,
          "cursor_linha": 1,
          "cursor_coluna": 1,
          "encoding": "utf-8",
          "tabulacao": "4",
          "fs_mtime_ms": null,
          "fs_len": null
        }
      ]
    }
  },
  "exibir": { },
  "formatar": { }
}
```

| Campo | Regra |
|-------|--------|
| `fechar_tudo_ao_sair` | Toggle do menu Abas. **false (padrão):** restaurar `sessao` + `.edit-session/` na próxima execução. **true:** ao sair, limpar workspace (não restaurar abas). |
| `salvar_desfazer_recentes` | Toggle do menu Abas. **false:** não gravar `undo.json`/`redo.json`; apagar existentes no shutdown. **true:** gravar pilhas (≥ 5 passos) se “fechar tudo ao sair” ligado. Ignorado se “fechar tudo ao sair” desligado. Default sugerido: **true**. |
| `indice_ativo` | Índice em `sessao` da aba em foco ao sair. |
| `limite` | Máximo de abas (default **10**). |
| `sessao` | Ordem = ordem do menu Abas. Cada entrada referencia `tab_id` + metadados; blobs em `.edit-session/tabs/{tab_id}/`. |
| `tab_id` | Chave da subpasta de sessão; estável entre save/restore da mesma aba. |
| `fs_mtime_ms` / `fs_len` | Timestamp e tamanho do arquivo no FS **no momento do shutdown** (ou último sync). Usado na restauração e limpeza (8.5). |
| `temporario` | Se true, conteúdo em `.edit-session/tabs/{tab_id}/content.tmp`. |

Artefatos **fora** do JSON (`.edit-session/`):

```
.edit-session/
  manifest.json              # espelho leve: tab_ids, ordem, indice_ativo (opcional redundância)
  tabs/
    {tab_id}/
      content.tmp              # NovoN ou snapshot quando necessário
      undo.json                # pilha undo serializada (se toggle ligado)
      redo.json                # pilha redo serializada (se toggle ligado)
      meta.json                # content_hash, cursor, encoding, fs_mtime_ms, fs_len, saved_at
```

`arquivo.recentes` permanece independente: só arquivos **fechados**, nunca a lista de abertas.

---

## 6. Regras de negócio

### 6.1. Abrir arquivo

1. Se path já está aberto → focar aba existente, `Ok`.
2. Se `tabs.len() >= max_tabs` (10) → **evicção da aba no final da fila** (último índice do menu, `tabs.len() - 1`):
   - Focar essa aba e, se **dirty**, modal **[Salvar] [Não Salvar] [Cancelar]** (`PromptSaveRequired`, razão `EvictTail`). **Cancelar** aborta a abertura; aba permanece na lista.
   - **Não Salvar** → remove aba do final; se tinha path, push em `arquivo.recentes`.
   - **Salvar** → salvar (ou Salvar Como se sem path); depois remover como acima.
   - Se aba do final for **pristine/saved** → remover silenciosamente; se tinha path, push em `arquivo.recentes`.
3. Carregar arquivo, criar `Tab`, inserir no **topo** (`index 0`), `active_index = 0`.
4. Remover path de `arquivo.recentes` se presente.
5. Registrar `fs_snapshot`.

### 6.2. Ctrl+N (Novo)

Ordem de decisão:

1. Se aba **ativa** é pristine vazia → **noop** (mantém foco; não cria aba).
2. Senão, se existe **qualquer** aba `Novo` / `NovoN` pristine (sem path, conteúdo baseline) → **focar a primeira** encontrada na ordem do menu (topo → fim).
3. Senão → criar nova aba `Novo` / `Novo1` / `Novo2`… no **topo**, pristine, sem `filepath`.

### 6.3. Ctrl+W (Fechar — aba ativa)

- **Dirty** → modal **[Salvar] [Não Salvar] [Cancelar]** (padrão `PROJECT_RULES`) para a aba ativa.
- **Salvar** → fluxo Salvar/Salvar Como; depois fechar aba.
- **Não Salvar** → descartar e fechar aba.
- **Cancelar** → abortar; foco permanece.

**Após fechar:**

| Situação | Resultado |
|----------|-----------|
| Restam outras abas | Foco na próxima da fila (ex.: índice 0 após remoção). |
| Era a única aba | Workspace com **uma** aba nova pristine `Sem Título` / `Novo` — equivalente ao modo documento único atual. |

Fechar aba com `filepath` → path vai para **topo** de `arquivo.recentes`.  
**Limpeza FS:** remover `.edit-session/tabs/{tab_id}/` da aba fechada (seção 8.5).

### 6.4. Ctrl+S / F10 / Salvar

- **`Ctrl+S`** e **`F10`:** salvar **somente a aba ativa**.
- Sem path → abrir fluxo **Salvar Como** (modal path).

### 6.4b. F2 / Renomear

- **`F2`** e menu **Arquivo → Renomear** (quando existir): renomear no filesystem a aba ativa **com path**.
- Modal pede novo nome; default = nome atual; diretório permanece o mesmo salvo path explícito.
- Após sucesso: atualizar `document.path`, título na borda, `fs_snapshot`, entrada em recentes se aplicável.
- Aba sem path ou dirty com conflito → modal de erro ou fluxo Salvar Como antes (implementação fase 1: exigir path salvo).

### 6.5. Salvar Todos (novo)

- Menu **Arquivo → Salvar Todos**; atalho **`Ctrl+Alt+S`** (mesmo padrão do Notepad oficial Windows).
- **`Ctrl+Shift+S`** permanece **Salvar Como** (aba ativa).
- Itera abas dirty na ordem do menu: com path → salvar; sem path → Salvar Como sequencial por aba.

### 6.6. Ctrl+Shift+W (Fechar Todos)

- Equivalente ao item **Abas → Fechar Todos**.
- Percorre abas dirty na **ordem do menu** (topo → fim), um modal por aba (seção 7.2).
- Ao concluir (ou se não houver dirty), fecha todas as abas e deixa **uma** aba pristine `Sem Título`.

### 6.7. Trocar aba (menu, Alt+1…0, Ctrl+Tab, F4)

- **Ctrl+Tab:** próxima aba (circular). *Pode ser interceptado pelo emulador host (ex.: Windows Terminal); o app deve implementar mesmo assim.*
- **Ctrl+Shift+Tab:** aba anterior (circular). *Mesma ressalva.*
- **`F4`:** próxima aba (circular) — alternativa segura no Windows.
- **`Shift+F4`:** aba anterior (circular).
- **`F6`:** alternar foco **Editor ↔ Terminal** quando o painel terminal estiver visível (`SPEC-TERMINAL-INFERIOR.md` §4).
- **Alt+1 … Alt+0:** foco direto na posição 1–10.
- Ao focar aba com `filepath`:
  - Comparar `fs_snapshot` (`modified`, `len`) com FS.
  - **Modificado externamente** → modal *Recarregar do disco?* **[Sim] [Não] [Cancelar]**.
    - **Sim:** recarregar conteúdo; **apagar** `undo.json`/`redo.json` da aba; atualizar `fs_snapshot`; remover entrada obsoleta de `sessao` no próximo save se usuário fechar aba.
    - **Não:** manter buffer em memória; marcar aba como divergente do FS (undo persistido invalidado na próxima sessão via hash — 8.5).
  - **Arquivo excluído** → modal *Arquivo não encontrado. Fechar aba?* **[Fechar] [Manter]**.
    - **Fechar:** remover aba; **limpar** `.edit-session/tabs/{tab_id}/`; remover de `sessao` ao persistir.
    - **Manter:** aba dirty em memória sem path válido; limpar undo persistido e `fs_*` em meta.

### 6.8. Limite de 10 abas

- Menu Abas lista no máximo **10** entradas.
- Ao abrir o 11º, a aba no **final da fila** sai (6.1); dirty exige modal antes de remover.

### 6.9. Undo / Redo

| Regra | Detalhe |
|-------|---------|
| Isolamento | Cada aba: pilha própria (`EditHistory` no `Editor` da aba). |
| Em memória | **≥ 5 passos** undo por aba durante a sessão (nunca abaixo de 5). |
| Persistência | Somente se **Fechar tudo ao sair** **e** **Salvar desfazer recentes** estiverem ligados. Grava `undo.json` + `redo.json` em `.edit-session/tabs/{tab_id}/` — **nunca** em `edit.json`. |
| Profundidade persistida | Mínimo **5** entradas; máximo configurável depois (default: igual `max_depth` do engine ou cap 20 na fase 1). |
| Toggle desligado | Undo só RAM; no shutdown apagar `undo.json`/`redo.json` das abas da sessão encerrada. |
| Invalidação | Ver seção **8.5** (FS externo, hash, fechamento de aba). |

### 6.10. Limpeza de sessão (resumo)

Toda operação que encerra o ciclo de vida de uma aba ou invalida conteúdo deve chamar **`SessionStore::purge_tab(tab_id)`** (ou equivalente). Detalhes na seção **8.5**.

---

## 7. Encerramento e modais globais

### 7.1. Sair (`Ctrl+Q`, `Alt+F4`, Arquivo → Sair)

1. Coletar abas dirty na **ordem do menu** (topo → fim).
2. Para **cada** aba dirty, exibir modal **[Salvar] [Não Salvar] [Cancelar]** (um de cada vez, sem sumário).
3. **Cancelar** em qualquer etapa → abortar saída; foco na aba onde cancelou; estado intacto.
4. Após todas resolvidas → `shutdown`, persistir `edit.json`.

### 7.2. Fechar Todos (`Ctrl+Shift+W`, menu Abas)

1. Mesma fila de prompts da seção 7.1 (**ordem do menu**, topo → fim).
2. **Cancelar** em qualquer etapa → abortar; foco na aba onde cancelou; demais abas permanecem abertas.
3. Após resolver dirty (ou se não houver) → fechar **todas** as abas; workspace fica com **uma** aba pristine `Sem Título` (equivalente ao modo documento único).

### 7.3. Abas sem path ao sair

- Abas `NovoN` dirty entram na fila de prompts de 7.1.
- **Salvar** → Salvar Como obrigatório.
- **Não Salvar** → descartar conteúdo; **purge** `.edit-session/tabs/{tab_id}/`.
- Com toggles ligados: conteúdo não salvo pode ir para `content.tmp` antes do prompt (8.3); undo só persiste se usuário não descartar e toggle **Salvar desfazer recentes** ativo.

---

## 8. Persistência e restauração de sessão

### 8.1. Toggle “Fechar tudo ao sair” desligado (default)

- Ao iniciar: uma aba vazia pristine.
- **Startup hygiene:** executar limpeza global de `.edit-session/` órfã (8.5.4) — pasta inteira removível se vazia ou sem manifest válido alinhado a `edit.json`.

### 8.2. Toggle “Fechar tudo ao sair” ligado

- **Shutdown:** serializar `sessao`, `indice_ativo`, `fs_mtime_ms`/`fs_len` por entrada; gravar artefatos em `.edit-session/tabs/{tab_id}/`.
- **Startup:** repopular abas na ordem; focar `indice_ativo`.
- Para cada entrada com `caminho` no disco:
  - Comparar `fs_mtime_ms`/`fs_len` salvos com FS atual.
  - **Excluído** → modal (6.7); não restaurar aba se usuário fechar; **purge** `tab_id`.
  - **Modificado externamente** → modal recarregar; se recarregar, **não** restaurar undo; se ignorar, restaurar buffer da sessão se existir `content.tmp` ou memória salva.
- **Undo/redo:** restaurar somente se **Salvar desfazer recentes** estava ligado **e** `meta.json`.`content_hash` bate com conteúdo carregado **e** FS não divergiu (8.4).

### 8.3. Arquivos temporários de conteúdo (fase 1)

- Abas `NovoN` ou buffers dirty não salvos: `content.tmp` em `.edit-session/tabs/{tab_id}/`.
- `temporario: true` + `tab_id` em `edit.json`; path real do arquivo fica **dentro** da sessão, não no JSON principal.
- Ao **Salvar Como** com sucesso: migrar de temp para path real; atualizar `fs_snapshot`; manter ou recriar undo conforme hash.

### 8.4. Undo / redo persistido (fase 1b — mesmo milestone se possível)

**Condições para gravar no shutdown:**

1. `fechar_tudo_ao_sair == false` (sessão persistida)
2. `salvar_desfazer_recentes == true`

**Formato `undo.json` / `redo.json`:** serialização das entradas de `EditHistory` (`start`, `removed`, `inserted`, `cursor_before`, `cursor_after`) — compatível com `src/editor/history.rs`.

**`meta.json` por aba:**

```json
{
  "content_hash": "blake3 ou sha256 do conteúdo restaurável",
  "cursor_linha": 12,
  "cursor_coluna": 4,
  "encoding": "utf-8",
  "fs_mtime_ms": 1717785600000,
  "fs_len": 4096,
  "saved_at_ms": 1717789200000
}
```

**Restauração:**

1. Carregar conteúdo (disco, `content.tmp` ou ambos conforme aba).
2. Calcular hash do buffer.
3. Se hash == `meta.content_hash` **e** FS coerente com `fs_mtime_ms`/`fs_len` (ou aba sem path) → carregar `undo.json`/`redo.json`.
4. Caso contrário → **descartar** pilhas persistidas; **purge** apenas `undo.json`/`redo.json` (manter `content.tmp` se ainda válido).

### 8.5. Higiene do filesystem (`.edit-session/`)

Módulo dedicado recomendado: **`SessionStore`** (`src/session/` ou `src/workspace/session_store.rs`).

#### 8.5.1. Quando purgar `{tab_id}` inteiro

| Evento | Ação |
|--------|------|
| Fechar aba (`Ctrl+W`, evicção confirmada) | `purge_tab(tab_id)` — remove pasta |
| **Não Salvar** em modal | `purge_tab(tab_id)` |
| Arquivo **excluído** externamente + usuário **Fechar aba** | `purge_tab(tab_id)` + remover de `sessao` |
| Toggle **Fechar tudo ao sair** ligado no shutdown | `purge_all()` após salvar `edit.json` |
| Toggle **Salvar desfazer recentes** desligado no shutdown | `purge_undo(tab_id)` para cada aba (mantém `content.tmp` se existir) |
| Toggle **Salvar desfazer recentes** desligado via modal **Apagar desfazer** | `purge_all_undo()` imediato (3.2.1) |

#### 8.5.2. Quando purgar só undo/redo

| Evento | Ação |
|--------|------|
| **Recarregar** após modificação externa | `purge_undo(tab_id)`; atualizar `fs_snapshot` |
| Hash conteúdo ≠ `meta.content_hash` na restauração | `purge_undo(tab_id)` |
| `fs_mtime_ms`/`fs_len` ≠ FS na restauração (arquivo com path) | `purge_undo(tab_id)`; modal recarregar (6.7 / 8.2) |
| Salvar com sucesso (`Ctrl+S`) | Opcional: truncar redo; undo em memória mantido; na sessão seguinte regravar |

#### 8.5.3. Timestamp e detecção externa

- Sempre que aba com path é **aberta**, **focada** ou **salva**: atualizar `fs_snapshot { modified, len }`.
- Na **persistência de sessão**: copiar para `fs_mtime_ms` / `fs_len` em `edit.json` e `meta.json`.
- Comparação: `modified` em millis + `len`; se plataforma não der mtime confiável, fallback para hash do arquivo no disco vs hash esperado.

#### 8.5.4. Limpeza no startup

1. Se `fechar_tudo_ao_sair` **ligado** → remover `.edit-session/` inteira (se existir).
2. Se **desligado** → listar subpastas de `tabs/`; **remover** diretórios cujo `tab_id` **não** está em `sessao` (órfãos de crash).
3. Para cada `tab_id` em `sessao`: validar FS; aplicar 8.2 / 8.4; purgar undo inválido.
4. Se `manifest.json` inconsistente com `edit.json`, preferir **`edit.json`** como fonte de verdade.

#### 8.5.5. Limpeza no shutdown

1. Gravar `edit.json` + artefatos das abas na `sessao` atual.
2. Remover pastas `tabs/{tab_id}` **não** referenciadas.
3. Se `salvar_desfazer_recentes` off → garantir ausência de `undo.json`/`redo.json` em todas as pastas mantidas.

---

## 9. Integração com `App` e modais existentes

| Modal / fluxo | Escopo |
|---------------|--------|
| Salvar / Sobrescrever / Path input | Aba ativa |
| Trocar codificação | Aba ativa |
| Converter tabulação | Aba ativa |
| Descartar para Abrir / Novo | Aba ativa (se dirty) |
| Quit / Fechar Todos / `Ctrl+Shift+W` | Fila de abas dirty (ordem do menu) |
| Evicção no limite (aba final dirty) | Aba no índice `len - 1` |

Estender `ConfirmKind` (ou equivalente) com contexto `{ tab_index, workspace_action }` onde necessário.

---

## 10. Atalhos (resumo)

| Atalho | Ação |
|--------|------|
| `Ctrl+N` | Nova aba, reuso de `NovoN` pristine, ou noop se ativa pristine vazia (6.2) |
| `Ctrl+W` | Fechar aba ativa |
| `Ctrl+Shift+W` | Fechar todas as abas |
| `Ctrl+S` / **`F10`** | Salvar aba ativa |
| **`F2`** | Renomear arquivo no FS (aba com path) |
| `Ctrl+Shift+S` | Salvar Como (aba ativa) |
| `Ctrl+Alt+S` | Salvar Todos |
| `Ctrl+Tab` | Próxima aba (se o host repassar) |
| `Ctrl+Shift+Tab` | Aba anterior (se o host repassar) |
| **`F4`** | Próxima aba (alternativa segura no Windows) |
| **`Shift+F4`** | Aba anterior (alternativa segura no Windows) |
| **`F6`** | Foco Editor ↔ Terminal (painel terminal visível) |
| `Alt+1` … `Alt+0` | Focar aba 1–10 |
| `Alt+S` | Abrir menu Abas |

Barra de menu (existente): `Alt+A` Arquivo, `Alt+E` Editar, `Alt+X` Exibir, `Alt+F` Formatar.

---

## 11. Plano de implementação sugerido

1. **`src/workspace/`** + **`SessionStore`** — tabs, evicção, `.edit-session/`, purge.
2. **Refatorar `App`** — aba ativa; swap; integração FS.
3. **Menus** — Abas (`Alt+S`); toggles; Salvar Todos; **Salvar desfazer recentes**.
4. **`edit.json` v2** — `tab_id`, timestamps, toggles; migração v1→v2.
5. **Modais sequenciais** — quit, fechar todos, evicção, FS externo.
6. **Persistência undo** — `undo.json`/`redo.json` + validação hash/timestamp.
7. **Higiene FS** — startup/shutdown/close (8.5).
8. **Testes** — purge órfão, undo inválido após mtime, toggles, restore 5 passos undo.

---

## 12. Decisões fechadas

| Tópico | Decisão |
|--------|---------|
| Salvar Todos | `Ctrl+Alt+S`; Salvar Como mantém `Ctrl+Shift+S`; Salvar também **`F10`** |
| Renomear | **`F2`** (aba com path) |
| Menu Arquivo | **`Alt+A`** (substitui `F10` da spec menu shell) |
| Fechar Todos | `Ctrl+Shift+W` + menu Abas |
| 11ª aba | Remove aba no **final da fila**; dirty → modal na aba evictada |
| Menu Abas | `Alt+S` |
| Prompts quit/fechar | Ordem do menu (topo → fim) |
| Arquivos temp | Fase 1 (`.edit-session/.../content.tmp`) |
| Undo entre sessões | **Sim**, opcional via toggle **Salvar desfazer recentes**; arquivos `undo.json`/`redo.json` fora do JSON |
| Limpeza FS | `SessionStore::purge_tab` / `purge_undo` / startup órfãos; timestamp `fs_mtime_ms` + `fs_len` |
| Default undo persistido | Toggle **ligado**; modal ao desligar se houver undo no disco (3.2.1) |

---

## 13. Histórico da spec

| Data | Nota |
|------|------|
| 2026-06-07 | Rascunho inicial (Gemini): tab bar, FIFO 5 abas, `recent_files` no workspace. |
| 2026-06-07 | Revisão: menu Abas fase 1; Recentes = fechados; 10 abas; `edit.json`; NovoN; Notepad++ FS. |
| 2026-06-07 | Decisões: evicção final da fila; `Ctrl+Alt+S`; `Ctrl+Shift+W`; `Alt+S`; Ctrl+N reuso NovoN; temps fase 1. |
| 2026-06-07 | Modal ao desmarcar **Salvar desfazer recentes** (Apagar / Manter / Cancelar). |
| 2026-06-08 | **F2** renomear no FS; **F10** salvar; menu Arquivo via **Alt+A**. |
