# SPEC — Terminal inferior integrado

**Status:** to-do  
**Data:** 2026-06-08  
**Relacionado:** `PROJECT_RULES.md`, `SPEC-MULTPLOS-ARQUIVOS.md`, `docs/EDITOR_LINUX_MASTER_REQUIREMENTS.md`, `src/ui/layers/terminal.rs`, `src/widgets/panel.rs`

---

## 1. Objetivo

Substituir o placeholder **Terminal (em breve)** por um **shell real** embutido (PTY), com borda ASCII no estilo do editor, scrollback, **múltiplas sessões** e coluna lateral de gestão — sem sair do alternate screen e **sem perder** o estado das abas de edição.

---

## 2. Layout visual (decisão PO)

O painel inferior compartilha a **mesma moldura contínua** do editor: o título da aba de texto fica no topo; o bloco terminal ocupa a faixa inferior, dividido em **área de output** (esquerda) e **coluna de sessões** (direita).

```
┌─[ Novo ]───────────────────────────────────────┐
│ ...                                            │
├─[ bash ]───────────────────────────┬───────────┤
│ user@pc$ ls_                       │        [+]│
│                                    │ 1 pwsh [-]│
│                                    │ 2 cmd  [-]│
│                                    │ −         │
└────────────────────────────────────┴───────────┘
```

### 2.1. Elementos

| Região | Descrição |
|--------|-----------|
| **Topo** | Título da aba de **edição** ativa: `─[ nome ]─` (com `*` se dirty). |
| **Área central** | Buffer de texto do editor (como hoje). |
| **Divisor horizontal** | Linha `├─[ {shell} ]─…─┬─…─┤` separa editor do bloco terminal. `{shell}` = rótulo da sessão ativa (`bash`, `pwsh`, `cmd`, …). |
| **Output (esquerda)** | Scrollback + linha de input do PTY; cursor do shell; borda inferior `└…┴…┘` fecha o retângulo. |
| **Coluna sessões (direita)** | Largura fixa (~10–12 colunas). **`[+]`** nova sessão; lista **`N nome [-]`**; sessão ativa marcada (`√` ou negrito); **`[-]`** por linha fecha aquela sessão; **`−`** na base fecha sessão ativa (atalho visual). |

### 2.2. Bordas e render

- Reutilizar `panel.rs` (`Plain`, cantos `┌┐└┘│─├┤┴┬`).
- Título do sub-bloco terminal na aresta superior esquerda: `─[ bash ]─` (nome configurável / detectado do shell).
- **Scroll** apenas na área de output (PgUp/PgDn, roda do mouse com foco no terminal).
- Altura mínima do bloco terminal: **6 linhas** (configurável depois em `edit.json`).

### 2.3. Foco de input

| `InputFocus` | Teclado/mouse |
|--------------|----------------|
| **Editor** | Comportamento atual do `EditorLayer`. |
| **Terminal** | Bytes/teclas encaminhados ao PTY da sessão ativa. |
| **Coluna sessões** | Opcional fase 2: clique em `[+]` / `[-]` / linha `N`; teclado continua prioritário. |

---

## 3. Modelo de dados

```rust
struct TerminalWorkspace {
    sessions: Vec<TerminalSession>,
    active: usize,
}

struct TerminalSession {
    id: String,
    label: String,           // "pwsh", "cmd", "bash" — editável depois
    pty: /* portable-pty */,
    scrollback: ScrollBuffer,
    scroll_offset: usize,
    cwd: PathBuf,
}
```

- **Spawn:** ao criar sessão, `cwd` = diretório do arquivo da aba de edição ativa; se sem path, `cwd` do processo.
- **Máximo de sessões:** 10 (paridade com abas de arquivo) — fase 2 se necessário começar com 4.
- **Shutdown:** matar processos filhos e fechar PTYs em `App::shutdown`.

---

## 4. Atalhos — terminal e foco

| Atalho | Ação |
|--------|------|
| **`Ctrl+T`** | Mostrar / ocultar painel terminal inferior (toggle `view.terminal`). |
| **`Ctrl+'`** | **Mesma ação** que `Ctrl+T` (compatibilidade VS Code). |
| **`F6`** | Com painel **visível:** alterna foco **Editor ↔ Terminal** (não troca aba de arquivo). Com painel **oculto:** noop ou abre painel e foca terminal (implementação: abrir + focar). |
| **`Esc`** | Com foco no terminal → devolve foco ao editor (painel permanece visível). |

**Reservado:** `F6` **não** deve ser reutilizado para outras ações globais enquanto o terminal existir.

---

## 5. Atalhos — abas de edição (referência cruzada)

Ver **`SPEC-MULTPLOS-ARQUIVOS.md` §6.7** e **`PROJECT_RULES.md`**.

| Atalho | Ação |
|--------|------|
| `Ctrl+Tab` | Próxima aba (quando o host repassar ao app) |
| `Ctrl+Shift+Tab` | Aba anterior |
| **`F4`** | Próxima aba (circular) |
| **`Shift+F4`** | Aba anterior (circular) |
| `Alt+1` … `Alt+0` | Foco direto na posição 1–10 |

**Nota Windows:** `Ctrl+Tab` / `Ctrl+Shift+Tab` podem ser capturados pelo **Windows Terminal** ou outro host. **`F4` / `Shift+F4`** são alternativa segura documentada e implementada no app.

---

## 6. Atalhos — funções de arquivo (F-keys)

| Tecla | Ação |
|-------|------|
| **`F2`** | **Renomear** arquivo no FS — aba ativa com path; modal com novo nome (mesmo diretório ou path relativo); `std::fs::rename`; atualiza path da aba, título, `fs_snapshot` e recentes; erro se destino existir. Sem path → noop ou pedir Salvar Como primeiro. |
| **`F6`** | Alternar foco **Editor ↔ Terminal** (§4) |
| **`F10`** | **Salvar** aba ativa (`Ctrl+S` equivalente; sem path → Salvar Como) |
| **`F1`** | Ajuda (placeholder) |
| **`F3`** | Próxima ocorrência de busca (`Shift+F3` = anterior) |

---

## 7. Coluna de sessões — interações

| Gesto | Ação |
|-------|------|
| **`[+]`** ou atalho futuro | Nova sessão PTY no `cwd` atual |
| **`N nome [-]`** (clique na linha) | Foca sessão `N` |
| **`[-]`** na linha | Fecha sessão `N`; confirma se processo filho ainda corre |
| **`−`** (rodapé da coluna) | Fecha sessão **ativa** |
| Última sessão fechada | Se painel ainda visível, mostrar área vazia ou fechar painel (decisão implementação: manter painel aberto com hint *“Nenhuma sessão — [+]”*) |

---

## 8. Implementação técnica (fases)

### Fase 1 — MVP

- Crate **`portable-pty`** (Linux PTY + Windows ConPTY).
- Uma sessão; borda + scrollback por linhas; foco Editor/Terminal; `Ctrl+T` / `Ctrl+'`; **`F6`** foco.
- Loop principal: `drain_pty()` non-blocking antes de cada frame.

### Fase 2 — Multi-sessão + layout PO

- Split horizontal output | coluna `[+]` / lista / `[-]`.
- Divisor `├─[ shell ]─┬─┤` integrado ao frame do editor.
- Até N sessões; rótulos `pwsh`, `cmd`, etc.

### Fase 3 — Polish

- ANSI/cores básicas (`vte` ou parser mínimo).
- Redimensionar altura do painel (arrastar divisor ou `Ctrl+↑/↓`).
- Persistir altura / última visibilidade em `edit.json` → `exibir.terminal_altura`.

---

## 9. Compatibilidade SSH / TTY

- PTY executa **no mesmo host** que o binário `edit` (correto em SSH).
- Se o emulador não repassar mouse ou cores, degradar sem crash.
- Teclado permanece caminho principal.

---

## 10. Critérios de aceite

- [ ] Painel com layout conforme §2 (mock ASCII aprovado pelo PO).
- [ ] Shell interativo (`cd`, `ls`, programas não-fullscreen) funcional.
- [ ] Scrollback com PgUp/PgDn no foco terminal.
- [ ] `Ctrl+T` e `Ctrl+'` equivalentes.
- [ ] `F6` alterna foco Editor ↔ Terminal com painel visível.
- [ ] `F4`/`Shift+F4` trocam abas conforme §5.
- [ ] Fechar sessão / sair do app não deixa processos zumbis.
- [ ] Múltiplas sessões na coluna direita (fase 2).

---

## 11. Histórico

| Data | Nota |
|------|------|
| 2026-06-08 | Spec inicial: layout PO com coluna de sessões; atalhos F6 foco terminal, F4/Shift+F4 abas, Ctrl+'. |
| 2026-06-08 | **F2** renomear; **F10** salvar (menu Arquivo → **Alt+A**). |
