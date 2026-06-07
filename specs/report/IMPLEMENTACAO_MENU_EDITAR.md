# Relatório — Menu Editar, Clipboard, Busca, Bloco/Multi-cursor

**Data:** 2026-06-07  
**Specs:** SPEC-MENU-EDITAR.md, docs/SPEC_BLOCK_MULTI_CURSOR.md

## Resumo

Menu **Editar** completo com clipboard ring (5 itens), submenu Colar Anterior, busca/substituição modal, seleção bloco e multi-cursor custom.

## Módulos

| Módulo | Função |
|--------|--------|
| `src/clipboard.rs` | Ring buffer 5 itens |
| `src/find.rs` | Busca próximo/anterior, substituir uma ocorrência |
| `src/cursors.rs` | `CursorManager`, modos Normal/Block/Multi |
| `src/block_select.rs` | Geometria retangular, copy com padding |
| `src/editor.rs` | Input multiplexado, TAB, copy/cut/paste |
| `src/events.rs` | Alt+drag bloco, Ctrl+clique multi, F3/Shift+F3 |

## Comportamentos

- `Ctrl+C/X/V`, `Ctrl+Shift+V` (colar anterior)
- Submenu Colar Anterior: 5 previews (20 chars)
- Bloco: Alt+LMB drag; multi ao soltar; Ctrl+clique adiciona cursor
- `Esc` cancela seleção especial

## Build

`cargo build` — OK.
