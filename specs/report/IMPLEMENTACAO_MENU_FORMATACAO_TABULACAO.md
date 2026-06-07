# Relatório — Menu Formatar (Codificação e Tabulação)

**Data:** 2026-06-07  
**Spec:** SPEC-MENU-FORMATACAO-TABULACAO.md

## Resumo

Menu **Formatar** (`Alt+F`) com submenus Codificação e Tabulação; estado visível na status bar; TAB conforme configuração.

## Módulos

| Módulo | Função |
|--------|--------|
| `src/encoding.rs` | UTF-8, UTF-8 sem BOM, UTF-16 LE/BE, ISO-8859-1, ANSI |
| `src/document.rs` | Campos `encoding`, `tabulation` |
| `src/menus.rs` | Topo Formatar com submenus |
| `src/editor.rs` | TAB → N espaços ou `\t` |
| `src/ui.rs` | Status: encoding + tabulação |

## Escopo

Codificação base funcional com reinterpretação ao abrir e conversão ao salvar; confirmação modal em mudanças sensíveis.

## Build

`cargo build` — OK.
