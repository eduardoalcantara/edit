# SPEC — Modal navegador de arquivos (Abrir / Salvar / Salvar Como)

**Status:** to-do  
**Data:** 2026-06-08  
**Prioridade:** alta (fecha **TV7** em `SPEC-UX-FIDELIDADE-TURBO-VISION.md`; substitui modal path manual)  
**Relacionado:** `PROJECT_RULES.md`, `SPEC-UX-FIDELIDADE-TURBO-VISION.md` (TV7), `SPEC-MULTPLOS-ARQUIVOS.md`, `src/modal/`, `src/file_io.rs`, `src/app.rs`

**Referência visual:** diálogo *Save File As* do Turbo Pascal 7 / Windows clássico (captura PO: painel cinza, borda double-line, área de listagem ciano, labels amarelas, botões verdes com sombra).

---

## 1. Objetivo

Substituir o modal atual de **entrada de caminho** (`Modal::PathInput` — uma linha de texto) por um **navegador de arquivos integrado**, reutilizável como base única para:

| Modo | Menu / atalho | Ação ao confirmar |
|------|---------------|-------------------|
| **Abrir** | Arquivo → Abrir (`Ctrl+O`) | Carrega arquivo existente em nova aba ou foca aba já aberta |
| **Salvar** | Arquivo → Salvar (`Ctrl+S` / `F10`) | Grava na aba ativa; **sem path** → abre este modal em modo Salvar Como |
| **Salvar Como** | Arquivo → Salvar Como (`Ctrl+Shift+S`) | Grava cópia no caminho escolhido |

O modal deve permitir **digitar** caminho/nome **ou** **navegar** por diretórios e arquivos, com filtro de extensão e opção de arquivos ocultos — paridade funcional com o diálogo nativo do Windows, com aparência Turbo Vision.

**Fora do escopo (fase 1 desta spec):**

- Renomear (`F2`) — pode reutilizar só o campo **Name** numa fase 2.
- Seleção múltipla de arquivos.
- Árvore de drives em painel separado (Windows); ver §4.6 para atalho `..` e drives.
- Rede / UNC avançado (listar, mas sem browsar shares remotos profundos).

---

## 2. Estado atual

| Hoje | Problema |
|------|----------|
| `Modal::PathInput` com prompt `"Caminho:"` | Usuário precisa digitar path completo |
| Sem listagem de diretório | Não há `..`, pastas, nem preview de seleção |
| Mesmo shell para Abrir e Salvar Como | OK estruturalmente, mas UI insuficiente |

**Substituição:** `PathInputKind::Open` e `PathInputKind::SaveAs` passam a abrir `FileBrowserModal`; `PathInputKind::Rename` permanece texto simples até fase 2.

---

## 3. Layout visual (wireframe)

Inspirado na captura Turbo Pascal; adaptado ao `Dialog` + `panel.rs` existentes.

```
┌─[ Save File As ]────────────────────────────────────────────┐
│ [·]                                                          │
│ Name                                                         │
│ ┌──────────────────────────────────────┐ ┌──┐ ┌────┐        │
│ │ helloworld.pas▌                      │ │▼ │ │ Ok │        │
│ └──────────────────────────────────────┘ └──┘ └────┘        │
│                                                              │
│ Files                                                        │
│ ┌──────────────────────────────────────┐ ┌────────┐        │
│ │ grabscreen/                          │ │        │        │
│ │ old/                                 │ │ Cancel │        │
│ │ copper.pas          ◄── highlight    │ │        │        │
│ │ CustomChips.pa                       │ └────────┘        │
│ │ ../                                  │                    │
│ └──────────────────────────────────────┘                    │
│ ◄══════════════════════════════════════►  (scroll H opcional)│
│                                                              │
│ Filter                                                       │
│ ┌──────────────────────────────────────┐                    │
│ │ *.pas                                │                    │
│ └──────────────────────────────────────┘                    │
│ [ ] Mostrar arquivos ocultos                                 │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ Work:Sources/*.pas                                       ││
│ │ copper.pas   883   Nov 8, 2015   8:33pm                  ││
│ └──────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

### 3.1. Regiões e cores (tema Azul Clássico / TV)

| Região | Estilo sugerido |
|--------|-----------------|
| Moldura externa | `PanelBorder::Double`, fundo cinza (`footer_bg` / cinza menu) |
| Título na borda | `[ Save File As ]` / `[ Open File ]` / `[ Save File ]` conforme modo |
| Labels `Name`, `Files`, `Filter` | Amarelo (`accent` / `status`) |
| Campo Name, Filter | Fundo azul editor (`editor_bg`), texto claro |
| Lista Files | Fundo ciano (`Cyan`) ou azul claro; item focado amarelo/negrito |
| Botões Ok / Cancel | Verde (`button_bg`) com sombra preta 1 célula (como modais atuais) |
| Barra de status inferior | Azul escuro; linha 1 = diretório + filtro ativo; linha 2 = metadados do item focado |

### 3.2. Tamanho

- Largura: ~70–85% da área útil do terminal, mínimo 60 colunas.
- Altura: ~70% do terminal, mínimo 18 linhas úteis.
- Centralizado; drop shadow como `Dialog::outer_rect`.

---

## 4. Comportamento funcional

### 4.1. Campo **Name** (caminho / nome do arquivo)

- Exibe **nome do arquivo** ou **caminho relativo/absoluto** conforme edição do usuário.
- Ao abrir o modal:
  - **Abrir:** diretório inicial = pasta do arquivo ativo, ou `cwd` do processo, ou último diretório usado (persistir em `edit.json` → `arquivo.ultimo_diretorio`).
  - **Salvar / Salvar Como:** nome sugerido = nome do arquivo ativo ou `NovoN.txt`; diretório = pasta do path atual.
- **Enter** no campo Name com path válido existente (modo Abrir) ou path gravável (modo Salvar) → mesma ação que **[Ok]**.
- Botão **▼** (opcional fase 1.1): dropdown de nomes recentes do diretório ou histórico curto; MVP pode omitir.

Sincronização Name ↔ lista:

- Duplo clique / Enter em pasta na lista → entra na pasta (Name mostra só nome ao salvar; path completo resolvido internamente).
- Enter em arquivo (modo Abrir) → confirma abertura.
- Seleção de arquivo na lista → atualiza Name com o nome do arquivo (não path completo, estilo TP).
- Edição manual em Name → ao confirmar, resolve contra `current_dir` (relativo) ou path absoluto.

### 4.2. Lista **Files**

Colunas lógicas (uma coluna visual no TUI, metadados na barra de status):

| Entrada | Render |
|---------|--------|
| Diretório | `nome/` (sufixo `/`) |
| Arquivo | `nome` |
| `..` | sempre primeiro item (sobe ao parent) |

- Ordenação: diretórios primeiro (A–Z), depois arquivos (A–Z).
- Filtro aplicado **somente a arquivos**; diretórios e `..` sempre visíveis.
- Scroll vertical: ↑/↓, PgUp/PgDn, roda do mouse.
- Scroll horizontal: apenas se nomes longos (fase 2); barra `◄══►` opcional.

### 4.3. Campo **Filter** (máscara)

- Sintaxe estilo shell: `*.rs`, `*.pas`, `*.*`, `README*`.
- Padrão por modo:
  - **Abrir:** `*.*` ou extensão inferida do contexto (ex. `.md` se editando markdown).
  - **Salvar:** extensão do nome em Name, ou `*.*`.
- Enter no Filter → reaplica listagem sem fechar modal.
- Linha de status superior: `{drive ou volume}:{path}/{filter}` estilo `Work:Sources/*.pas`.

Implementação: conversão glob simples (`*` → `.*`, `?` → `.`) com crate `glob` ou matcher interno; documentar limitações.

### 4.4. Checkbox **Mostrar arquivos ocultos**

- Toggle estilo TV (`√` na margem ou `[X]`).
- **Windows:** arquivos com atributo `hidden` ou `system`; pastas ocultas idem.
- **Unix:** arquivos/pastas cujo nome começa com `.`.
- Desligado por padrão; persistir preferência em `edit.json` → `arquivo.mostrar_ocultos`.

### 4.5. Botões

| Botão | Ação |
|-------|------|
| **Ok** | Valida e executa (abrir / salvar / salvar como) |
| **Cancelar** | Fecha sem efeito; Esc equivalente |

Help no rodapé do app (via `ModalLayer::footer_hint`): texto do botão sob cursor, como demais modais.

### 4.6. Navegação de diretório

- **`..`** — diretório pai.
- **Enter** em pasta — desce.
- **Backspace** no campo Name — não sai do modal; opcional Alt+↑ sobe pasta (fase 2).
- **Windows:** prefixo de volume `C:\`, `D:\` na barra de status; trocar drive via input `D:` + Enter no Name (MVP) ou lista de drives (fase 2).
- **Linux/macOS:** path absoluto `/home/...`; sem drives.

### 4.7. Barra de status (duas linhas)

**Linha 1:** `{diretório_atual}` + filtro ativo, ex. `C:\Sources\*.pas`

**Linha 2:** metadados do item **focado** na lista (não do Name se diferente):

| Campo | Exemplo |
|-------|---------|
| Nome | `copper.pas` |
| Tamanho | `883` bytes |
| Data | `Nov 8, 2015  8:33pm` (locale PT ou ISO compacto) |

Pastas: tamanho omitido ou `-`; data = mtime do diretório.

### 4.8. Modos e validação

| Modo | Ok habilitado quando |
|------|----------------------|
| Abrir | Name ou seleção aponta para **arquivo existente** legível |
| Salvar / Salvar Como | Name não vazio; path válido; aviso se sobrescrever (modal `ConfirmKind::OverwriteSave` já existente) |

Erros: mensagem na barra de status ou modal de erro curto (*"Arquivo não encontrado"*, *"Acesso negado"*).

---

## 5. Teclado e mouse

### 5.1. Ciclo de foco (Tab)

Ordem sugerida:

1. Name  
2. Lista Files  
3. Filter  
4. Checkbox ocultos  
5. Ok  
6. Cancelar  

Shift+Tab inverte. Setas na lista movem foco; setas nos campos de texto movem cursor.

### 5.2. Atalhos no modal

| Tecla | Ação |
|-------|------|
| Esc | Cancelar |
| Enter | Ativa controle focado (Ok se botão; entra pasta se dir; confirma arquivo em Abrir) |
| Alt+O | Ok (mnemônico) |
| Alt+C ou Esc | Cancelar |
| F5 | Atualizar listagem do diretório atual |
| F2 | Renomear item (fase 2 — opcional) |

### 5.3. Mouse

- Clique em item da lista → foca e seleciona.
- Duplo clique em pasta → entra.
- Duplo clique em arquivo (Abrir) → confirma.
- Clique nos botões → mesma semântica dos modais atuais (`Dialog::hit_button`).

---

## 6. Arquitetura técnica

### 6.1. Novos módulos

```
src/modal/
  file_browser.rs    # FileBrowserModal — estado, paint, input
  file_browser/
    listing.rs       # leitura dir, filtro, ocultos, ordenação
    path_resolve.rs  # Name + cwd → PathBuf
```

Ou `src/file_browser/` no top-level se crescer; preferir `src/modal/file_browser.rs` para manter camada UI.

### 6.2. Tipos principais

```rust
pub enum FileBrowserMode {
    Open,
    Save,
    SaveAs,
}

pub struct FileBrowserModal {
    pub dialog: Dialog,           // moldura + botões Ok/Cancel
    pub mode: FileBrowserMode,
    pub current_dir: PathBuf,
    pub name_input: String,
    pub filter_input: String,
    pub show_hidden: bool,
    pub entries: Vec<FileEntry>,
    pub list_cursor: usize,
    pub list_scroll: usize,
    pub focus: FileBrowserFocus,
    // ...
}

pub struct FileEntry {
    pub name: String,
    pub kind: FileEntryKind,
    pub meta: Option<FileMetadata>,
}
```

### 6.3. Integração `App`

| Ponto atual | Mudança |
|-------------|---------|
| `request_open()` | `Modal::file_browser(Open, …)` |
| `request_save()` sem path | `Modal::file_browser(SaveAs, …)` |
| `request_save_as()` | `Modal::file_browser(SaveAs, …)` |
| `submit_path_input()` | `submit_file_browser()` → `open_path` / `save_to_path` |
| `ModalLayer` | branch paint/key/mouse para `FileBrowser` |

### 6.4. Persistência `edit.json`

```json
"arquivo": {
  "ultimo_diretorio": "C:\\Users\\…",
  "mostrar_ocultos": false,
  "filtro_abrir": "*.*"
}
```

### 6.5. Testes

| Teste | Tipo |
|-------|------|
| `listing::applies_glob_filter` | unit |
| `path_resolve::relative_and_absolute` | unit |
| `path_resolve::parent_dot_dot` | unit |
| `file_browser::enter_directory_updates_name` | unit |
| Abrir arquivo via temp dir | integração |

Usar `tempfile` / diretórios temporários; sem UI snapshot na fase 1.

---

## 7. Critérios de aceite

1. **Abrir** (`Ctrl+O`): navegar até um `.rs` sem digitar path completo; Enter abre o arquivo.
2. **Salvar Como**: navegar, digitar novo nome, Ok grava no disco.
3. **Salvar** sem path na aba: abre o mesmo modal com nome sugerido.
4. **Filter** `*.pas` oculta outros arquivos; pastas e `..` permanecem.
5. **Ocultos** desligado: não lista `.git` / atributo hidden; ligado: lista.
6. Barra de status mostra path + tamanho/data do item focado.
7. **Cancelar** / Esc não altera documento.
8. Visual: borda double-line, labels amarelas, lista destacada, botões verdes — reconhecível como TV/Borland.
9. `cargo test` verde; sem regressão em fluxos dirty/save existentes.

---

## 8. Plano de implementação sugerido

| Fase | Entrega |
|------|---------|
| **1** | `listing.rs` + leitura de diretório, filtro, ocultos, `..` |
| **2** | `FileBrowserModal` paint (layout fixo) + foco teclado |
| **3** | Mouse + integração Abrir / Salvar Como |
| **4** | Salvar sem path + persistência `ultimo_diretorio` |
| **5** | Polimento visual TV + barra de status com data/tamanho |
| **6** | Remover `PathInput` de Open/SaveAs; atualizar TV7 como done |

Estimativa: **2–3 semanas** (1 dev, incluindo testes e Windows + Linux).

---

## 9. Relação com outras specs

| Spec | Relação |
|------|---------|
| `SPEC-UX-FIDELIDADE-TURBO-VISION.md` TV7 | Esta spec **é** a implementação detalhada de TV7 |
| `SPEC-LIMITACOES-PENDENTES.md` L? | Fecha item “sem file picker / modais texto-only” |
| `SPEC-MULTPLOS-ARQUIVOS.md` | Abrir/evict/salvar todos usam os mesmos hooks `open_path` / `save_to_path` |
| `SPEC-MENU-AJUDA.md` | Documentar atalhos após estabilizar UI |

---

## 10. Decisões fechadas

| Tópico | Decisão |
|--------|---------|
| Base única para Abrir/Salvar/Salvar Como | Sim — `FileBrowserMode` |
| Renomear no mesmo modal | Não na fase 1 |
| Filtro | Campo separado `Filter`, não só status |
| Ocultos | Checkbox explícito, persistido |
| Path manual | Mantido no campo Name (power users) |
| Sobrescrever ao salvar | Reutiliza `ConfirmKind::OverwriteSave` existente |

---

## 11. Histórico

| Data | Nota |
|------|------|
| 2026-06-08 | Rascunho inicial (PO: captura Turbo Pascal Save File As; substitui PathInput). |
