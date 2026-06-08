# edit — Editor de terminal (Power Edit)

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 2.2

Editor TUI para Linux/Windows com menus estilo **Turbo Vision**, núcleo **ropey**, UX previsível e proteção contra perda de trabalho.

## Requisitos

- Rust **1.85+**
- Terminal interativo (TTY)

## Build e execução

```bash
cargo build
cargo run
cargo run -- arquivo.txt outro.rs
cargo run -- --clean
cargo run -- --workspace src/main.rs
# binário: target/debug/edit
```

### Linha de comando

| Parâmetro | Ação |
|-----------|------|
| `arquivo…` | Abre um ou mais arquivos em abas (até 10); o **primeiro** listado fica ativo no topo |
| `--clean` | Limpa sessão (`.edit-session/`) e abas salvas no config ativo |
| `--workspace` | Usa config local `./.edit/.edit.workspace` + sessão em `./.edit/.edit-session/` |
| `--help` | Ajuda |

Com `--workspace`, na primeira execução copia preferências do `edit.json` global **sem abas abertas**. Combine `--clean --workspace` para resetar o workspace do projeto.

Exemplos:

```bash
edit README.md src/lib.rs
edit --clean
edit --workspace -- main.rs tests/mod.rs
```

## Funcionalidades

### Edição

- Buffer de texto **ropey** (`src/editor/`) — Insert/Replace, Enter, seleção linear (Shift+setas, mouse, Ctrl+A).
- **Smart Word Navigation:** `Ctrl+←/→`, `Ctrl+Shift+←/→` (camelCase, separadores, dígitos).
- Tabulação literal e por espaços (2/4/8); expansão visual de `\t`; conversão De/Para no modal.
- Undo/redo **por aba** (pilha isolada no engine de cada aba).
- Multi-cursor (Ctrl+clique) e seleção em bloco (Alt+arraste) — parcial.

### Workspace / múltiplas abas (fase 1)

- Até **10 abas** abertas simultaneamente; troca via menu **Abas** (`Alt+S`), atalhos ou lista dinâmica.
- **Recentes (Arquivo):** arquivos **fechados** (até 10). **Menu Abas:** arquivos **abertos**.
- Documentos sem título: `Novo`, `Novo1`, `Novo2`…
- `Ctrl+N`: noop se aba ativa pristine; senão foca `NovoN` pristine existente; senão cria nova no topo.
- `Ctrl+W`: fecha aba ativa; última aba → uma aba pristine `Novo`.
- `Ctrl+Shift+W`: Fechar Todos (fila de confirmação por aba dirty).
- `Ctrl+Alt+S`: Salvar Todos; `Ctrl+Shift+S`: Salvar Como (aba ativa).
- `Ctrl+Tab` / `Ctrl+Shift+Tab`: próxima/anterior aba (circular).
- `Alt+1` … `Alt+0`: foco direto na posição 1–10 do menu Abas.
- 11ª aba: evicção da aba no **final da fila** (modal se dirty).
- Toggles Abas: **Fechar tudo ao sair**, **Salvar desfazer recentes** (modal ao desligar se houver undo no disco).
- Submenu **Ordenar por** (nome, caminho, abertos primeiro/último, status).
- Sessão em **`.edit-session/`** ao lado do executável (`content.tmp`, higiene por `tab_id`).
- **`edit.json` v2** — seção `arquivo.abas` (toggles, ordem, metadados, cursor).

### Menus (Alt+A / S / E / X / F)

| Menu | Destaques |
|------|-------------|
| **Arquivo** | Novo, Abrir, **Recentes** (10 fechados), Salvar, Salvar Como, **Salvar Todos**, Fechar, Sair |
| **Abas** | Lista dinâmica (até 10, radio na ativa), Fechar Todos, toggles sessão, Ordenar por |
| **Editar** | Recortar/Copiar/Colar, clipboard 5 itens, Buscar/Substituir (modal), Selecionar tudo |
| **Exibir** | Temas (Escuro, Claro, Azul Clássico, Matrix), zoom, word wrap, mostrar símbolos/espaços/tabs/EOL, colunas, margem, borda, painel, terminal, rodapé, **memória** |
| **Formatar** | Codificação (UTF-8, ANSI, UTF-16…), tabulação, converter tabulação |

### Interface

- Compositor de camadas (`src/ui/`): menu dropdown opaco, modais `Dialog`, rodapé contextual.
- Título do editor na borda: `[ nome ]` ou `[ Novo2* ]` (asterisco se dirty).
- Rodapé: help do menu ou hover dos grupos à esquerda; à direita `Tam XXX/NNN/YYYY | Pos XX/YY | modo | encoding | tab | Mem NMB` (XXX = chars visíveis no viewport, NNN = linhas, YYY = total com `\n`).
- Modais com mouse/hover; sair/fechar com **[Salvar] [Não Salvar] [Cancelar]**; `Ctrl+Q` / `Alt+F4` global.
- **Navegador de arquivos** estilo Turbo Pascal para Abrir / Salvar Como (`Ctrl+O`, `Ctrl+Shift+S`).

### Persistência

- **`edit.json`** ao lado do executável — `arquivo` (recentes + **abas**), `exibir`, `formatar`.
- Migração automática de `.edit/recent.json` legado.
- **`.edit-session/`** — conteúdo temporário de abas `NovoN`; purge ao fechar aba.

### Atalhos principais

| Atalho | Ação |
|--------|------|
| `Ctrl+N/O/S/W` | Novo / Abrir / Salvar / Fechar aba |
| **`F10`** | Salvar aba ativa |
| **`F2`** | Renomear arquivo no FS (aba com path) |
| `Ctrl+Shift+S` | Salvar Como |
| `Ctrl+Alt+S` | Salvar Todos |
| `Ctrl+Shift+W` | Fechar Todos |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Próxima / anterior aba |
| `Alt+1` … `Alt+0` | Foco aba 1–10 |
| `Alt+S` | Menu Abas |
| `Ctrl+F/H` | Buscar / Substituir |
| `Ctrl+Q`, `Alt+F4` | Sair (global) |
| `Esc` | Sair (editor, sem menu/modal) |
| `Alt+A` | Menu Arquivo |
| `Alt+H` | Menu Ajuda |
| **`F1`** | Ajuda → Features |
| `Ctrl+T` / `Ctrl+'` | Do editor: abre/foca terminal; do terminal: fecha painel |
| **`Ctrl+E`** | Foco no editor |
| **`Ctrl+G`** | Ir para linha... |
| **`F6`** | Alterna foco Editor ↔ Terminal |
| **`F4`** / **`Shift+F4`** | Próxima / anterior aba (Windows-safe) |

Ver `PROJECT_RULES.md` para lista completa.

## Estrutura do repositório

| Pasta | Conteúdo |
|-------|----------|
| `src/` | Código — `editor/`, `workspace/`, `session/`, `terminal/`, `ui/`, `modal/`, `config.rs`, `menus.rs`… |
| `specs/done/` | Especificações implementadas |
| `specs/to-do/` | Pendências (fase 2 abas, limitações) |
| `specs/report/` | Relatórios de implementação |
| `docs/` | Requisitos mestre e docs auxiliares |

## Documentação

1. [`PROJECT_RULES.md`](PROJECT_RULES.md) — regras estáveis (v2.0)
2. [`PROJECT_STATUS.md`](PROJECT_STATUS.md) — estado e pendências
3. [`PROJECT_TIMELINE.md`](PROJECT_TIMELINE.md) — histórico
4. [`.cursorrules`](.cursorrules) — regras para o Cursor

## Próximo marco

- **Barra de abas visual** (fase 2) — [`specs/to-do/SPEC-MULTPLOS-ARQUIVOS.md`](specs/to-do/SPEC-MULTPLOS-ARQUIVOS.md)
- Serialização completa `undo.json`/`redo.json` entre sessões
- Detecção de alteração externa ao focar aba

## Testes

```bash
cargo test
```

77 testes unitários.
