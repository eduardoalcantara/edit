# Relatório — Implementação arquitetura ropey

**Autor:** Cursor  
**Data:** 2026-06-07  
**Versão:** 1.0  
**Spec:** `specs/done/EDITOR_LINUX_SPEC_ARQUITETURA_GERAL.md`

## Resumo

Substituição completa de `tui-textarea` por core próprio com buffer `ropey`, render Ratatui customizado e camada de input desacoplada. Build compila sem `tui-textarea` na árvore de dependências.

## Entregue

| Componente | Módulo | Status |
|------------|--------|--------|
| Buffer rope | `src/editor/engine.rs` | ✅ insert/delete/overtype, load/save |
| Cursor + virtual_col | `src/editor/cursor.rs` | ✅ movimento vertical preserva coluna |
| Viewport | `src/editor/viewport.rs` | ✅ scroll automático |
| Seleção bloco/linear/multi | `src/editor/selection.rs` | ✅ Alt+drag, Ctrl+click, copy/cut |
| Undo/redo | `src/editor/history.rs` | ✅ pilha incremental (100 entradas) |
| Busca | `src/editor/search.rs` | ✅ find next/prev/replace one |
| Comandos | `src/editor/commands.rs` | ✅ EditorCommand |
| Render | `src/editor/render.rs` | ✅ seleção, bloco, match de busca |
| Input | `src/input/` | ✅ keyboard + mouse hit-test real |
| Facade | `src/editor/mod.rs` | ✅ API compatível com app/events/ui |
| Integração | app, document, events, ui | ✅ dirty via string rope |

## Removido

- `tui-textarea` (Cargo.toml)
- `src/editor.rs` (wrapper antigo)
- `src/cursors.rs`, `src/block_select.rs`, `src/find.rs`

## Testes unitários

- `editor/engine.rs`: overtype avança cursor; virtual_col em movimento vertical
- `editor/cursor.rs`: roundtrip UTF-8; rope vazio
- `editor/selection.rs`: padding bloco; merge de cursores

## Limitações honestas (fora do big bang)

| Item | Estado |
|------|--------|
| File picker com árvore FS (TV7) | Pendente — modal path |
| Diálogo Find estilo Borland completo | Pendente — modal texto simples |
| Word wrap visual completo | Flag existe; wrap visual básico |
| Multi-cursor: setas sincronizadas | Pendente (L4) |
| Regex / substituir todas na busca | Pendente (L5) |
| Painel lateral / terminal funcional | Placeholder (L7) |
| Shift+setas seleção linear | Não implementado |
| Seleção linear por arraste mouse | Não implementado |

## Critérios de aceite

| Critério | Status |
|----------|--------|
| `cargo build` sem tui-textarea | ✅ |
| Digitar, backspace, delete, setas, home/end | ✅ |
| Insert e Overtype | ✅ |
| Seleção bloco + multi + copy/cut/paste | ✅ |
| Undo/redo | ✅ |
| Busca F3/Shift+F3, Ctrl+F modal | ✅ |
| Viewport scroll | ✅ |
| Menus via dispatch_action | ✅ |
| Testes unitários core | ✅ |
