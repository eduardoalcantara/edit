# SPEC — TextInput em modais e diálogos Buscar / Ir para linha

**Status:** done  
**Data:** 2026-06-09  
**Implementado:** 2026-06-09 (integrado em `main`)  
**Relacionado:** `PROJECT_RULES.md`, `SPEC-MENU-EDITAR.md`, `SPEC-MODAL-ARQUIVO.md`, `src/modal/text_input.rs`, `src/modal/find_replace.rs`, `src/modal/go_to_line.rs`

---

## 1. Objetivo

1. Oferecer **edição de linha única** nos modais com paridade a um campo de texto padrão (Windows/Linux): cursor, seleção, clipboard e teclas de navegação.
2. Formalizar modais dedicados para **Buscar**, **Substituir** e **Ir para linha** com layout estilo navegador de arquivos (campo → linha em branco → botões).
3. Corrigir **Esc** nos modais: fecha o diálogo (equivalente a **[Fechar]** / **[Cancelar]**), sem limpar busca ativa no editor.

---

## 2. Componente `TextInput`

**Módulo:** `src/modal/text_input.rs`

| Capacidade | Teclas |
|------------|--------|
| Inserir texto | Caracteres imprimíveis |
| Apagar | `Backspace`, `Delete` |
| Cursor | `←` `→` |
| Seleção | `Shift` + `←` `→` `Home` `End` |
| Início / fim | `Home`, `End` |
| Clipboard | `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+A` |
| Filtro numérico | `CharAccept::AsciiDigit` (Ir para linha) |

**API:**

- `TextInput::new`, `text()`, `set_text()`, `clear()`
- `handle_key(key, clipboard, accept) -> bool` — consome tecla se tratada
- `display_focused()` — cursor `▌`; `display_unfocused()` — texto sem cursor
- Renderização via `paint_text_input()` em `src/modal/form_controls.rs`

**Consumidores:**

| Modal | Campos |
|-------|--------|
| Buscar / Substituir | `pattern`, `replacement` |
| Ir para linha | `line`, `col` |
| Navegador de arquivos | `name_input`, `filter_input` |
| Path input (Renomear) | `input` |

O `ModalLayer` passa `&mut app.clipboard` para todos os `handle_key`.

---

## 3. Modal Buscar / Substituir

**Módulo:** `src/modal/find_replace.rs`

- Layout: label + campo, linha em branco, botões de comando.
- **Buscar:** [Buscar] [Próximo] [Anterior] [Limpar] [Fechar]
- **Substituir:** [Substituir] [Substituir tudo] [Limpar] [Fechar]
- **Esc** → `FindReplaceCommand::Close` (fecha modal; mantém destaque se busca já aplicada).
- **[Limpar]** → limpa campos e remove destaque no texto (`clear_search`).
- **F3** / **Shift+F3** no editor navegam ocorrências sem modal aberto.

**Busca no editor** (`src/editor/search.rs`, `render.rs`):

- Destaque de **todas** as ocorrências (fg+bg); ocorrência ativa mais forte.
- Cache `search_match_positions` para navegação sem revarrer o buffer a cada tecla.

---

## 4. Modal Ir para linha

**Módulo:** `src/modal/go_to_line.rs`

- Campos linha e coluna (`CharAccept::AsciiDigit`).
- **Esc** fecha sem aplicar.
- Confirmar posiciona cursor na aba ativa.

---

## 5. Regras de Esc (global vs modal)

| Contexto | Esc |
|----------|-----|
| Modal aberto | Fecha / Cancela (ação do botão homônimo) |
| Menu aberto | Fecha menu |
| Terminal com foco | Devolve foco ao editor |
| Editor, busca ativa, sem modal | `clear_search()` |
| Editor, sem busca, sem modal | `request_quit()` |

Implementação global: `src/ui/compositor.rs` (só age no editor quando `!modal.is_active()`).

---

## 6. Critérios de aceite

1. Em qualquer campo de modal, `Delete`, setas, `Home`/`End` e `Ctrl+C/V/X/A` funcionam.
2. **Esc** em modal de busca fecha o diálogo sem limpar destaque já aplicado.
3. **[Limpar]** no modal de busca limpa campos e destaque.
4. Buscar destaca todas as ocorrências; Próximo/Anterior cicla entre elas.
5. Ir para linha aceita só dígitos nos campos.

---

## 7. Histórico

| Data | Nota |
|------|------|
| 2026-06-09 | `TextInput`, modais find/go-to-line, Esc corrigido, destaque múltiplo de busca. |
