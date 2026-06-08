# edit — Editor de terminal (Power Edit)

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 2.0

Editor TUI para Linux/Windows com menus estilo **Turbo Vision**, núcleo **ropey**, UX previsível e proteção contra perda de trabalho.

## Requisitos

- Rust **1.85+**
- Terminal interativo (TTY)

## Build e execução

```bash
cargo build
cargo run
# binário: target/debug/edit
```

## Funcionalidades

### Edição

- Buffer de texto **ropey** (`src/editor/`) — Insert/Replace, Enter, seleção linear (Shift+setas, mouse, Ctrl+A).
- **Smart Word Navigation:** `Ctrl+←/→`, `Ctrl+Shift+←/→` (camelCase, separadores, dígitos).
- Tabulação literal e por espaços (2/4/8); expansão visual de `\t`; conversão De/Para no modal.
- Undo/redo por documento (pilha no engine).
- Multi-cursor (Ctrl+clique) e seleção em bloco (Alt+arraste) — parcial.

### Menus (Alt+A / E / X / F)

| Menu | Destaques |
|------|-------------|
| **Arquivo** | Novo, Abrir, **Recentes** (10 fechados), Salvar, Salvar Como, Fechar, Sair |
| **Editar** | Recortar/Copiar/Colar, clipboard 5 itens, Buscar/Substituir (modal), Selecionar tudo |
| **Exibir** | Temas (Escuro, Claro, Azul Clássico, Matrix), zoom, word wrap, mostrar símbolos/espaços/tabs/EOL, colunas, margem, borda, painel, terminal, rodapé, **memória** |
| **Formatar** | Codificação (UTF-8, ANSI, UTF-16…), tabulação, converter tabulação |

### Interface

- Compositor de camadas (`src/ui/`): menu dropdown opaco, modais `Dialog`, rodapé contextual.
- Título do editor na borda: `[ nome ]` (asterisco se dirty).
- Rodapé: help do menu à esquerda; à direita `Tam XXX/YYY | Pos XX/YY | modo | encoding | tab | Mem NMB`.
- Modais com mouse/hover; sair com **[Salvar] [Não Salvar] [Cancelar]**; `Ctrl+Q` / `Alt+F4` global.

### Persistência

- **`edit.json`** ao lado do executável — seções `arquivo`, `exibir`, `formatar` (recentes, toggles, tema, formatação padrão).
- Migração automática de `.edit/recent.json` legado.

### Atalhos principais

| Atalho | Ação |
|--------|------|
| `Ctrl+N/O/S/W` | Novo / Abrir / Salvar / Fechar |
| `Ctrl+Shift+S` | Salvar Como |
| `Ctrl+F/H` | Buscar / Substituir |
| `Ctrl+Q`, `Alt+F4` | Sair |
| `F10` | Menu Arquivo |

Ver `PROJECT_RULES.md` para lista completa.

## Estrutura do repositório

| Pasta | Conteúdo |
|-------|----------|
| `src/` | Código — `editor/`, `ui/`, `modal/`, `config.rs`, `menus.rs`… |
| `specs/done/` | Especificações implementadas |
| `specs/to-do/` | Pendências (ex.: múltiplos arquivos/abas) |
| `specs/report/` | Relatórios de implementação |
| `docs/` | Requisitos mestre e docs auxiliares |

## Documentação

1. [`PROJECT_RULES.md`](PROJECT_RULES.md) — regras estáveis (v2.0)
2. [`PROJECT_STATUS.md`](PROJECT_STATUS.md) — estado e pendências
3. [`PROJECT_TIMELINE.md`](PROJECT_TIMELINE.md) — histórico
4. [`.cursorrules`](.cursorrules) — regras para o Cursor

## Próximo marco

**Múltiplos arquivos / workspace** — [`specs/to-do/SPEC-MULTPLOS-ARQUIVOS.md`](specs/to-do/SPEC-MULTPLOS-ARQUIVOS.md): até 10 abas via menu **Abas** (`Alt+S`), sessão `.edit-session/`, undo persistido opcional.

## Testes

```bash
cargo test
```

~70 testes unitários.
