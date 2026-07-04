# Limitações pendentes — Editor Linux

**Autor:** Cursor (documentação pós-implementação menus)  
**Data:** 2026-06-09 (revisão alinhada ao código)  
**Versão:** 1.1  
**Origem:** relatórios em `specs/report/`, `PROJECT_STATUS.md`, código em `src/`

## Objetivo

Registrar limitações conhecidas da implementação atual como **pontos a resolver** em specs futuras. Este arquivo não substitui specs funcionais; serve como backlog técnico priorizado.

---

## Prioridade alta

### L0 — Fidelidade visual Turbo Vision (ver spec dedicada)

| Campo | Valor |
|-------|--------|
| **Estado** | Parcial — shell funcional; gaps visuais e de chrome permanecem |
| **Progresso** | TV5 (Replace) ✅; TV7 (file browser) ✅; TV11 parcial (tema **VGA 16 cores**); painel referência SideKick ✅ (`specs/done/SPEC-REFERENCE-PANE-SIDEKICK.md`); terminal inferior ✅ |
| **Impacto** | Menus sem sombra TV, rodapé F-keys incompleto, modais sem botões reais, Find dialog rudimentar |
| **Referência** | `specs/to-do/SPEC-UX-FIDELIDADE-TURBO-VISION.md` (TV1–TV4, TV6, TV8–TV10, TV12–TV13 pendentes) |
| **Resolução esperada** | Paleta/menu opaco, rodapé F-keys, botões modais clicáveis, diálogo Find estilo Borland |

### L1 — Highlight visual de seleção em bloco

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — `editor/render.rs` aplica highlight por linha na área visível |
| **Resolução** | Migração ropey (2026-06-07) |

### L2 — Mapeamento mouse → cursor impreciso

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — `input/mouse.rs` + `Editor::inner_area()` com hit-test real |
| **Resolução** | Migração ropey (2026-06-07) |

### L3 — Recortar seleção normal incompleto

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — `EditorEngine::delete_selection` + `cut_selection` no core rope |
| **Resolução** | Migração ropey (2026-06-07) |

### L4 — Multi-cursor: setas e digitação parcial

| Campo | Valor |
|-------|--------|
| **Estado** | Modo multi trata Char, Backspace, Delete, Esc; **setas não sincronizadas** entre cursores |
| **Impacto** | Comportamento abaixo da spec PO (`docs/SPEC_BLOCK_MULTI_CURSOR.md`) |
| **Resolução esperada** | Mover todos os cursores; merge em colisão; materialização de espaço virtual consistente |

### L5 — Busca e substituição limitadas

| Campo | Valor |
|-------|--------|
| **Estado** | `editor/search.rs` — busca literal; `replace_all` e `replace_one` funcionais; **sem regex**; highlight de ocorrências parcial |
| **Impacto** | `Ctrl+F` / `Ctrl+H` / `F3` funcionais mas rudimentares |
| **Módulos** | `src/editor/search.rs`, `src/modal.rs`, `src/app.rs` |
| **Resolução esperada** | Wrap documentado, opcional regex, contagem de resultados, highlight persistente |

### L6 — Modal de busca/substituição rudimentar

| Campo | Valor |
|-------|--------|
| **Estado** | Um único campo editável; alternância busca/substituição sem foco real entre campos |
| **Impacto** | UX de substituição confusa |
| **Resolução esperada** | Dois campos com Tab; Enter confirma ação contextual (ver TV9) |

---

## Prioridade média

### L7 — Painel lateral e terminal inferior

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — PTY multi-sessão em `src/ui/layers/terminal.rs` |
| **Referência** | `specs/done/SPEC-TERMINAL-INFERIOR.md` |
| **Resolução** | Implementado 2026-06-08 |

### L8 — Mostrar símbolos, espaços, tabs e EOL

| Campo | Valor |
|-------|--------|
| **Estado** | Flags em `ViewState`; **sem efeito visual** no buffer renderizado |
| **Impacto** | Itens do menu Exibir → Mostrar não alteram a edição |
| **Resolução esperada** | Render pass que substitui caracteres invisíveis por glyphs (estilo Notepad++) |

### L9 — Zoom sem efeito no terminal

| Campo | Valor |
|-------|--------|
| **Estado** | `view.zoom` (1–3) refletido na status bar; **sem alteração de densidade** na TUI |
| **Impacto** | Menu Zoom cumpre estado lógico, não acessibilidade real |
| **Resolução esperada** | Ajuste de padding/fonte conforme capacidade do terminal ou documentar degradação |

### L10 — Tema personalizado

| Campo | Valor |
|-------|--------|
| **Estado** | Cinco temas fixos (Escuro, Claro, Azul Clássico, **VGA 16 cores**, Matrix); item "Personalizado" → "em breve" |
| **Impacto** | `PROJECT_RULES.md` exige tema customizável além dos presets |
| **Resolução esperada** | Arquivo de config local (cores/palette) + reload |

### L11 — Encoding: escopo mínimo e riscos de perda

| Campo | Valor |
|-------|--------|
| **Estado** | `encoding.rs` — UTF-8, UTF-16, ISO-8859-1/ANSI; sem detecção heurística; conversão ANSI/ISO simplificada (`char as u8`) |
| **Impacto** | Caracteres fora da página de código podem corromper silenciosamente |
| **Referência** | `specs/done/SPEC-MENU-FORMATACAO-TABULACAO.md` §Não deve incluir |
| **Resolução esperada** | Modal de confirmação em todos os caminhos sensíveis; conversão via crate especializado; BOM consistente |

### L12 — Abrir recente com documento dirty

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — `pending_open_path` + diálogo `OPEN_UNSAVED_FULL` (Salvar / Não Salvar / Ignorar / Cancelar) |
| **Resolução** | 2026-06-09 (`app.rs`, `modal/buttons.rs`) |

### L13 — Persistência de `ViewState` e preferências

| Campo | Valor |
|-------|--------|
| **Estado** | **Parcial** — `edit.json` persiste tema, wrap, terminal, colunas guia, abas, split, recentes (`config.rs` + `persist_user_config`) |
| **Impacto** | Algumas flags de sessão (ex.: zoom) ainda não serializadas |
| **Resolução esperada** | Completar campos faltantes; documentar schema v2 |

---

## Prioridade baixa / dívida técnica

### L14 — Clipboard apenas interno

| Campo | Valor |
|-------|--------|
| **Estado** | Ring buffer em memória (`clipboard.rs`); **sem OS clipboard** (Windows/Linux) |
| **Impacto** | Não integra com outras aplicações |
| **Resolução esperada** | `arboard` ou API nativa por plataforma |

### L15 — Recentes: parser JSON manual

| Campo | Valor |
|-------|--------|
| **Estado** | `recent.rs` parseia JSON com função ad hoc |
| **Impacto** | Fragilidade com paths escapados complexos |
| **Resolução esperada** | Dependência `serde_json` ou formato linha-a-linha |

### L16 — Coluna guia visual rudimentar

| Campo | Valor |
|-------|--------|
| **Estado** | `ui.rs` desenha bloco 1 coluna na posição fixa |
| **Impacto** | Pode sobrepor texto; não acompanha scroll horizontal do textarea |
| **Resolução esperada** | Overlay alinhado ao viewport do editor |

### L17 — `cargo build` com warnings de dead code

| Campo | Valor |
|-------|--------|
| **Estado** | Warnings esporádicos (`ActionId` variants, helpers não usados, etc.) |
| **Impacto** | Ruído no CI futuro |
| **Resolução esperada** | Usar APIs ou `#[allow]` justificado + `-D warnings` em CI quando estável |

### L18 — Pipeline de testes

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — suite `cargo test -- --test-threads=1` (200+ testes unitários) |
| **Resolução** | Cobertura em engine, config, menus, workspace, modals, split, referência |

### L19 — Sistema de abas

| Campo | Valor |
|-------|--------|
| **Estado** | ✅ Resolvido — até 10 abas, menu Abas, sessão `.edit-session/` |
| **Referência** | `specs/done/SPEC-MULTPLOS-ARQUIVOS.md` |
| **Resolução** | Implementado 2026-06-09 |

### L20 — Atalho `Alt+R` para recentes

| Campo | Valor |
|-------|--------|
| **Estado** | Spec Arquivo cita `Alt+R`; implementação abre menu Arquivo sem foco em Recentes |
| **Impacto** | Atalho documentado ≠ comportamento |
| **Resolução esperada** | Abrir cascata Recentes diretamente ou submenu com foco |

---

## Mapa por módulo

| Módulo | Limitações relacionadas |
|--------|-------------------------|
| `src/input/` | L2, L20 |
| `src/editor/` | L3, L4, L5 |
| `src/modal.rs`, `src/app.rs` | L5, L6, L12 |
| `src/view_state.rs`, `src/ui/` | L8, L9, L16 |
| `src/encoding.rs`, `src/file_io.rs` | L11 |
| `src/config.rs`, `src/app.rs` | L13 |
| `src/clipboard.rs` | L14 |
| `src/theme.rs`, menus Temas | L10 |
| Projeto / CI | L17 |

---

## Critério de remoção deste item

Cada limitação **Li** deve ser riscada ou movida para `specs/done/` quando uma spec dedicada for implementada, testada e referenciada em `PROJECT_STATUS.md` e `PROJECT_TIMELINE.md`.

## Ordem sugerida de ataque

1. ~~L1 + L2~~ ✅  
2. L4 + L5 + L6 (Editar e Find/Replace completos)  
3. L8 + L9 (Exibir com efeito real)  
4. L11 + L13 (Formatar e persistência restante)  
5. L0 / TV1–TV4, TV6, TV8–TV10 (fidelidade visual Turbo Vision)
