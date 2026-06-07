# PROJECT_STATUS — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.1

## Estado atual

### Concluído
- Estrutura base do repositório definida.
- Diretórios principais criados.
- Especificação funcional v1 elaborada.
- Regras iniciais de comportamento documentadas.
- **V1 compilável (spec Cursor v1.1):** projeto Rust com Ratatui, Crossterm e `tui-textarea`.
- Arquitetura modular em `src/` (`main`, `app`, `editor`, `ui`, `events`, `theme`).
- Loop principal de aplicação com renderização TUI.
- Área central de edição real via `tui-textarea`.
- Tema escuro nativo como padrão (paleta semântica preparada para claro e azul clássico).
- Layout básico: barra superior, editor central e rodapé com atalhos.
- Tratamento de eventos de teclado e mouse com degradação graciosa (SSH/TTY).
- Saída limpa com `Ctrl+Q` e verificação de terminal interativo.

### Em andamento
- Planejar pipeline de testes e validação automatizada.

### Pendências
- Implementar persistência de arquivos (Ctrl+S / Ctrl+O).
- Implementar menus interativos completos.
- Implementar alternância de temas (claro, azul clássico, customizado).
- Implementar abrir recente.
- Implementar clipboard interno.
- Implementar modais de confirmação.
- Implementar terminal inferior.
- Implementar seleção retangular.
- Implementar sistema de abas.

## Ponto de retorno

Se houver perda de contexto, iniciar por este arquivo e seguir para `PROJECT_RULES.md`, depois `PROJECT_TIMELINE.md`.
