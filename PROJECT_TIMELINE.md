# PROJECT_TIMELINE — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 1.8

## Registro

- 2026-06-06 — V1 compilável (`2446fa7`).
- 2026-06-06 — Inicial Completa (documento, modais, file I/O) — commit pendente anterior.
- 2026-06-07 — **Menu Shell** — `src/menus.rs`, remoção de `menu.rs` falso, prioridade modal→menu→editor.
  - Relatório: `specs/report/IMPLEMENTACAO_MENU_SHELL.md`
  - Spec: `specs/done/SPEC-MENU-SHELL.md`
- 2026-06-07 — **Menus Arquivo/Exibir** — `recent.rs`, `view_state.rs`, dispatch completo.
  - Relatório: `specs/report/IMPLEMENTACAO_MENU_ARQUIVO_EXIBIR.md`
  - Spec: `specs/done/SPEC-MENU-ARQUIVO-EXIBIR.md`
- 2026-06-07 — **Menu Editar + bloco/multi-cursor** — `clipboard.rs`, `find.rs`, `cursors.rs`, `block_select.rs`.
  - Doc: `docs/SPEC_BLOCK_MULTI_CURSOR.md`
  - Relatório: `specs/report/IMPLEMENTACAO_MENU_EDITAR.md`
  - Spec: `specs/done/SPEC-MENU-EDITAR.md`
- 2026-06-07 — **Menu Formatar** — `encoding.rs`, tabulação TAB, `Alt+F`.
  - Relatório: `specs/report/IMPLEMENTACAO_MENU_FORMATACAO_TABULACAO.md`
  - Spec: `specs/done/SPEC-MENU-FORMATACAO-TABULACAO.md`
