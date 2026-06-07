# PROJECT_STATUS — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 2.1

## Estado atual

### Concluído
- Estrutura base do repositório e V1 compilável.
- Inicial Completa: documento, file I/O, modais, Insert/Replace, status bar contextual.
- **Menu Shell (SPEC-MENU-SHELL):** subsistema Turbo Vision real (`src/menus.rs`).
- **Menus Arquivo/Exibir (SPEC-MENU-ARQUIVO-EXIBIR):** recentes, view_state, toggles visuais.
- **Menu Editar (SPEC-MENU-EDITAR):** clipboard 5 itens, busca, bloco/multi-cursor.
- **Menu Formatar (SPEC-MENU-FORMATACAO-TABULACAO):** encoding e tabulação.
- **Arquitetura ropey (EDITOR_LINUX_SPEC_ARQUITETURA_GERAL):** core `ropey`, render custom, input desacoplado; `tui-textarea` removido.
- **Correções pós-migração ropey (2026-06-07):**
  - Enter cria nova linha; cursor EOF corrigido (`char_idx_to_line_col`).
  - Seleção linear: Shift+setas, arraste mouse, Ctrl+A; highlight no render.
  - Modo Replace: não apaga `\n` (linhas permanecem independentes).
  - Layout TV4 parcial: título da janela = só nome do arquivo; rodapé com Ln/Col | Insert | encoding | tab | status (sem F-keys).
  - Placeholder removido; editor vazio com cursor em 0,0.
  - Clipboard: `arboard` (SO + ring interno); `copy_text` corrigido para bloco e linear.
- **Correções UX menu/modal/bordas (2026-06-07):**
  - Menu dropdown opaco (linhas preenchidas até a largura); cores alinhadas à barra de menu/rodapé (`footer_bg`).
  - Submenus só abrem com Right/Enter/clique (`expanded_submenus`), não ao focar item pai.
  - Bordas: editor/painéis `Plain`; modais e menus `Double`.
  - Modal sair sem salvar: pergunta com nome do arquivo; botões [Salvar] [Não Salvar] [Cancelar].
- **Widget painel reutilizável (`src/widgets/panel.rs`) (2026-06-07):**
  - Dropdown renderizado **após** o editor (z-order correto, opaco).
  - Bordas ASCII manuais com separadores `╠══╣` conectados às verticais.
  - Altura do painel corrigida (+2 bordas); barra de menu preenchida em cinza.
- **Compositor de camadas UI (`src/ui/`) (2026-06-07):**
  - Trait `UiLayer` + `Compositor`: pintura bottom→top, input top→bottom.
  - Camadas: Desktop, Editor, Footer, MenuBar, MenuDropdown (overlay), Modal.
  - Menu dropdown e modal capturam input; modal fecha menu automaticamente.
- **Atalhos sair + mouse em modais (2026-06-07):**
  - `Alt+F4` e `Ctrl+Q` encerram o programa globalmente (compositor), inclusive com modal/menu aberto.
  - Botões de modal respondem a clique e hover (Confirm, PathInput, Find).
- **Baseline pristine / dirty (2026-06-07):**
  - Documento novo ou vazio não dispara confirmação de saída; baseline `saved_content` alinhada ao rope (`EMPTY_DOCUMENT_TEXT`).
- **Caracteres e sombra Turbo Vision (2026-06-07):**
  - Submenu `»` (UTF-8); sombra vertical `█`; sombra horizontal `▀` (estilo fg sombra + bg editor).
- **Barra de menu (2026-06-07):**
  - Mnemônicos Alt+letra corretos na renderização (Exibir = X); help contextual no rodapé por item em foco.
- **Editor e temas (2026-06-07):**
  - Título na borda: `[ nome do arquivo ]`; tema **Matrix** (verde terminal) no lugar de Personalizado.
- **Correção cursor (2026-06-07):**
  - Documento vazio usa rope `""` (não `"\n"`); viewport resetado ao abrir/novo; digitar não preenche colunas fantasma após navegação vertical.
- **Margem do editor (2026-06-07):**
  - Exibir → Margem: Sem / Uma linha / Duas linhas; padding interno no render (topo, base, esquerda, direita).
- **Borda do editor (2026-06-07):**
  - Exibir → Borda: Visível / Invisível; laterais e base ocultas no modo invisível; título mantido no topo (`└ [ nome ] ─┘`).
  - Com terminal ativo: divisor `├─────┤` (borda visível) ou `─────` (invisível); layout divide área editor/terminal; camada placeholder `TerminalLayer`.
- Relatórios em `specs/report/` para as 4 fases de menu + migração ropey.
- Specs de menu e arquitetura em `specs/done/`.

### Em andamento
- **Fidelidade Turbo Vision:** `specs/to-do/SPEC-UX-FIDELIDADE-TURBO-VISION.md` (TV1–TV3 paleta/rodapé; TV7 file picker pendente).
- Resolução das demais limitações em `specs/to-do/SPEC-LIMITACOES-PENDENTES.md`.

### Pendências
- L4 — multi-cursor: setas sincronizadas.
- L5 — busca regex / substituir todas.
- L14 — clipboard SO integrado parcialmente (`arboard`); fallback silencioso se indisponível.
- Demais itens L6–L20 — ver `specs/to-do/SPEC-LIMITACOES-PENDENTES.md`.

## Ponto de retorno

`PROJECT_RULES.md` → `PROJECT_TIMELINE.md` → specs em `specs/done/`.
