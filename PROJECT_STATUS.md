# PROJECT_STATUS — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 1.8

## Estado atual

### Concluído
- Estrutura base do repositório e V1 compilável.
- Inicial Completa: documento, file I/O, modais, Insert/Replace, status bar contextual.
- **Menu Shell (SPEC-MENU-SHELL):** subsistema Turbo Vision real (`src/menus.rs`).
- **Menus Arquivo/Exibir (SPEC-MENU-ARQUIVO-EXIBIR):** recentes, view_state, toggles visuais.
- **Menu Editar (SPEC-MENU-EDITAR):** clipboard 5 itens, busca, bloco/multi-cursor custom.
- **Menu Formatar (SPEC-MENU-FORMATACAO-TABULACAO):** encoding e tabulação.
- Relatórios em `specs/report/` para as 4 fases de menu.
- Specs de menu em `specs/done/`.

### Em andamento
- **Fidelidade Turbo Vision:** `specs/to-do/SPEC-UX-FIDELIDADE-TURBO-VISION.md` (TV7 file picker pendente; TV1–TV6 parcialmente endereçados).
- Resolução das demais limitações em `specs/to-do/SPEC-LIMITACOES-PENDENTES.md`.

### Pendências
- Ver backlog completo (20 itens L1–L20) em `specs/to-do/SPEC-LIMITACOES-PENDENTES.md`.

## Ponto de retorno

`PROJECT_RULES.md` → `PROJECT_TIMELINE.md` → specs em `specs/done/`.
