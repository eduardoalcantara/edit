# PROJECT_TIMELINE — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.1

## Objetivo

Registrar cronologicamente mudanças do repositório, incluindo implementações de especificações, documentação, correções de bugs, ajustes de UI/UX e commits relevantes.

## Regra de uso

Cada entrada deve conter:

- data;
- tipo da mudança;
- resumo objetivo;
- arquivos afetados;
- referência de commit, quando existir.

## Registro

- 2026-06-06 — Criação da estrutura inicial do repositório.
- 2026-06-06 — Definição da especificação funcional v1 do Editor Linux.
- 2026-06-06 — Definição das regras iniciais de UX, temas, recentes, clipboard, modais e terminal inferior.
- 2026-06-06 — **Implementação:** V1 compilável conforme `specs/to-do/EDITOR_LINUX_SPEC_CURSOR_V1.1.md`.
  - **Tipo:** implementação de base técnica.
  - **Resumo:** Editor TUI compilável com loop de app, `tui-textarea`, tema escuro, layout (barra/editor/rodapé), eventos teclado/mouse, compatibilidade TTY/SSH com degradação graciosa e saída via `Ctrl+Q`.
  - **Arquivos:** `Cargo.toml`, `.gitignore`, `src/main.rs`, `src/app.rs`, `src/editor.rs`, `src/ui.rs`, `src/events.rs`, `src/theme.rs`, `PROJECT_STATUS.md`, `PROJECT_TIMELINE.md`, `specs/to-do/EDITOR_LINUX_SPEC_CURSOR_V1.1.md`.
  - **Commit:** ver histórico git desta data.
