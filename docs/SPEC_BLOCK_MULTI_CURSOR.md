# SPEC — Seleção em Bloco e Multi-cursor

**Autor:** PO / Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 1.0

## Objetivo

Comportamento customizado de seleção retangular (bloco) e multi-cursor no core ropey, alinhado ao EDIT.EXE / editores clássicos.

## Modos

| Modo | Descrição |
|------|-----------|
| `Normal` | Um cursor; core rope |
| `Block` | Retângulo `[StartRow..EndRow] × [StartCol..EndCol]` |
| `Multi` | Vários cursores alinhados |

## Ativação

- **Bloco:** `Alt` + arraste botão esquerdo (LMB)
- **Multi:** ao soltar mouse após bloco → 1 cursor/linha em `EndCol`
- **Adicionar cursor:** `Ctrl` + clique LMB
- **Cancelar:** `Esc` ou clique simples sem modificador → `Normal` (cursor primário)

## Copy bloco

- Linhas unidas por `\n`
- Padding com espaços onde `linha.len() < EndCol`

## Digitação multi

- Inserir em todos os cursores
- Materializar espaço virtual (expandir linha com espaços)
- Backspace/Delete sincronizados
- Merge de cursores em colisão

## Módulos

- `src/editor/cursor.rs` — `Cursor`, `SelectionMode`
- `src/editor/selection.rs` — bloco, copy com padding, merge
- `src/editor/engine.rs` — multiplexação de input conforme modo
- `src/editor/render.rs` — highlight visual de seleção

## Critérios de aceite

- Bloco via Alt+drag funcional
- Copy bloco com padding correto
- Multi-cursor após soltar bloco
- Ctrl+clique adiciona cursor
- Esc cancela para Normal
