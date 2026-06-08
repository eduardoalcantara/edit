# Limitações pendentes — Editor Linux

**Autor:** Cursor (documentação pós-implementação menus)  
**Data:** 2026-06-07  
**Versão:** 1.0  
**Origem:** relatórios em `specs/report/` (fases Menu Shell → Formatar), `PROJECT_STATUS.md`, código em `src/`

## Objetivo

Registrar limitações conhecidas da implementação atual como **pontos a resolver** em specs futuras. Este arquivo não substitui specs funcionais; serve como backlog técnico priorizado.

---

## Prioridade alta

### L0 — Fidelidade visual Turbo Vision (ver spec dedicada)

| Campo | Valor |
|-------|--------|
| **Estado** | UI funcional mas visualmente distante da referência Borland |
| **Impacto** | Menus transparentes, cores erradas, rodapé ausente, modais texto-only, sem file picker |
| **Referência** | `specs/to-do/SPEC-UX-FIDELIDADE-TURBO-VISION.md` (itens TV1–TV13) |
| **Resolução esperada** | Paleta VGA, menus opacos com sombra, rodapé F-keys, botões reais, browser FS |

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
| **Estado** | `find.rs` — busca literal simples; modal substitui **uma** ocorrência; sem regex; sem highlight de ocorrências |
| **Impacto** | `Ctrl+F` / `Ctrl+H` / `F3` funcionais mas rudimentares |
| **Módulos** | `src/find.rs`, `src/modal.rs`, `src/app.rs` |
| **Resolução esperada** | Navegação com wrap documentado, substituir todas, opcional regex, contagem de resultados |

### L6 — Modal de busca/substituição rudimentar

| Campo | Valor |
|-------|--------|
| **Estado** | Um único campo editável; alternância busca/substituição sem foco real entre campos |
| **Impacto** | UX de substituição confusa |
| **Resolução esperada** | Dois campos com Tab; Enter confirma ação contextual |

---

## Prioridade média

### L7 — Painel lateral e terminal inferior (placeholders)

| Campo | Valor |
|-------|--------|
| **Estado** | Toggles em `view_state.rs` / menu Exibir; **sem widget funcional** |
| **Impacto** | Status bar indica "on" mas não há painel/terminal real |
| **Referência** | `PROJECT_RULES.md` (`Ctrl+T`), `app.rs` mensagens "placeholder" |
| **Resolução esperada** | Ver `specs/to-do/SPEC-TERMINAL-INFERIOR.md` — layout PO, PTY, multi-sessão, atalhos |

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
| **Estado** | Item menu "Personalizado" → mensagem "em breve" |
| **Impacto** | `PROJECT_RULES.md` exige tema customizável |
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
| **Estado** | `OpenRecent` dispara modal genérico `DiscardForOpen` sem preservar path alvo |
| **Impacto** | Após confirmar, usuário precisa reescolher o arquivo recente |
| **Resolução esperada** | `ConfirmKind::DiscardForRecent { path }` ou fila de intenção pós-modal |

### L13 — Persistência de `ViewState` e preferências

| Campo | Valor |
|-------|--------|
| **Estado** | Wrap, coluna guia, tema etc. vivem só na sessão |
| **Impacto** | Preferências resetam ao fechar o editor |
| **Resolução esperada** | JSON em `.edit/config.json` (ou equivalente) |

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
| **Estado** | ~10 warnings (`ActionId` variants, helpers em `find.rs`, `block_highlight_rows`, etc.) |
| **Impacto** | Ruído no CI futuro |
| **Resolução esperada** | Usar APIs ou `#[allow]` justificado + `-D warnings` em CI quando estável |

### L18 — Pipeline de testes inexistente

| Campo | Valor |
|-------|--------|
| **Estado** | Validação manual + `cargo build` |
| **Impacto** | Regressões em bloco, encoding, menus não detectadas automaticamente |
| **Resolução esperada** | Testes unitários (`find`, `encoding`, `block_select`, `recent`) + smoke TUI opcional |

### L19 — Sistema de abas

| Campo | Valor |
|-------|--------|
| **Estado** | Um documento por instância |
| **Impacto** | Fora do escopo das 4 specs de menu; listado em `PROJECT_STATUS.md` |
| **Resolução esperada** | Spec dedicada multi-documento |

### L20 — Atalho `Alt+R` para recentes

| Campo | Valor |
|-------|--------|
| **Estado** | Spec Arquivo cita `Alt+R`; implementação abre menu Arquivo (`events.rs`) sem foco em Recentes |
| **Impacto** | Atalho documentado ≠ comportamento |
| **Resolução esperada** | Abrir cascata Recentes diretamente ou submenu com foco |

---

## Mapa por módulo

| Módulo | Limitações relacionadas |
|--------|-------------------------|
| `src/events.rs` | L2, L20 |
| `src/editor.rs` | L3, L4 |
| `src/cursors.rs`, `src/block_select.rs` | L1, L2, L4 |
| `src/find.rs`, `src/modal.rs` | L5, L6 |
| `src/view_state.rs`, `src/ui.rs` | L7, L8, L9, L16 |
| `src/encoding.rs`, `src/file_io.rs` | L11 |
| `src/recent.rs`, `src/app.rs` | L12, L13 |
| `src/clipboard.rs` | L14 |
| `src/theme.rs`, menus Temas | L10 |
| Projeto / CI | L17, L18, L19 |

---

## Critério de remoção deste item

Cada limitação **Li** deve ser riscada ou movida para `specs/done/` quando uma spec dedicada for implementada, testada e referenciada em `PROJECT_STATUS.md` e `PROJECT_TIMELINE.md`.

## Ordem sugerida de ataque

1. L1 + L2 (bloco visível e mouse correto)  
2. L3 + L4 + L5 (Editar completo e confiável)  
3. L7 + L8 (Exibir com efeito real)  
4. L11 + L13 (Formatar e persistência)  
5. L18 (testes antes de abas e tema custom)
