# Relatório — Implementação Menu Shell

**Data:** 2026-06-06  
**Spec:** SPEC-MENU-SHELL.md  
**Commit:** feat(menus): implementa menu shell estilo Turbo Vision / EDIT.EXE

## Resumo

Substituído o menu falso (strings estáticas em `menu.rs`) por subsistema interativo `menus.rs` estilo Turbo Vision / EDIT.EXE.

## Arquivos alterados

| Arquivo | Ação |
|---------|------|
| `src/menus.rs` | Criado — `MenuBar`, `MenuNode`, `ActionId`, `MenuState`, render, teclado/mouse |
| `src/menu.rs` | Removido |
| `src/theme.rs` | Estilos `menu_bar`, `menu_top_active`, `menu_item_focus`, etc. |
| `src/ui.rs` | Layout 4 faixas: título + menu bar + editor + status |
| `src/events.rs` | Prioridade modal → menu → editor; `Alt+A/E/X`, `F10`; removidos `Ctrl+Alt+*` |
| `src/app.rs` | `menu_bar`, `menu_state`, `dispatch_action` |
| `src/main.rs` | `mod menus` |

## Comportamento

- Barra de menu com Arquivo, Editar, Exibir (pull-down + cascata).
- Atalhos: `F10` (primeiro menu), `Alt+A/E/X` (mnemônicos).
- Navegação: setas, Enter, Esc, mouse (clique topo, itens, fora fecha).
- Apenas **Sair** habilitado no menu; demais itens desabilitados (fases seguintes).
- `Ctrl+Q` e menu Sair → fluxo de confirmação existente.

## Build

`cargo build` — OK.

## Próximo passo

Fase 2: menus Arquivo e Exibir com `recent.rs` e `view_state.rs`.
