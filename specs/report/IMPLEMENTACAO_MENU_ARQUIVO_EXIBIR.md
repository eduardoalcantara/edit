# Relatório — Menus Arquivo e Exibir

**Data:** 2026-06-07  
**Spec:** SPEC-MENU-ARQUIVO-EXIBIR.md

## Resumo

Menus **Arquivo** e **Exibir** funcionais com histórico de recentes, estado visual persistente em sessão e reflexo na status bar.

## Módulos

| Módulo | Função |
|--------|--------|
| `src/recent.rs` | 10 arquivos, JSON em `.edit/recent.json`, submenu cascata |
| `src/view_state.rs` | Zoom, wrap, mostrar, painel, terminal, rodapé, coluna guia, tema |
| `src/menus.rs` | Árvore Arquivo/Exibir expandida, itens habilitados |
| `src/app.rs` | `dispatch_action` para operações de arquivo e toggles Exibir |
| `src/editor.rs` | `set_word_wrap` via `WrapMode` |
| `src/ui.rs` | Coluna guia, rodapé condicional, indicadores na status bar |
| `src/file_io.rs` | Leitura/escrita com encoding do documento |

## Atalhos

- Arquivo: `Ctrl+N/O/S/W/Q`, `Ctrl+Shift+S`, `Alt+R` (abre menu Arquivo)
- Recentes via submenu cascata

## Build

`cargo build` — OK.
