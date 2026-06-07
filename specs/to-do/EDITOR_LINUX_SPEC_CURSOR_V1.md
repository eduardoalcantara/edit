# Especificação para o Cursor — Editor Linux V1 Compilável

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## Objetivo

Orientar o Cursor na criação da primeira versão compilável do Editor Linux, com base estrutural mínima, loop de aplicação e separação clara de responsabilidades.

## Escopo desta versão

Esta versão deve entregar apenas a base funcional inicial do aplicativo, sem tentar concluir todas as features do produto.

### Deve incluir
- Projeto Rust compilável.
- Estrutura modular em `src/`.
- Loop principal de aplicação.
- Renderização básica da interface TUI.
- Tratamento básico de eventos de teclado e mouse.
- Estado global inicial da aplicação.
- Tema inicial.
- Layout básico com área principal, rodapé e espaço para expansão futura.

### Não deve incluir ainda
- Terminal inferior funcional completo.
- Clipboard interno completo.
- Histórico de recentes completo.
- Modais complexos.
- Sistema completo de menus.
- Seleção retangular avançada.
- Integração com Drive ou servidor.
- Sistema completo de abas.

## Stack obrigatória

- Linguagem: Rust.
- Interface: Ratatui.
- Backend de terminal: Crossterm.
- Base inicial do editor de texto: `tui-textarea` ou estrutura própria simples, se necessário para compilar.

## Estrutura de arquivos esperada

O Cursor deve criar ou manter os seguintes arquivos em `src/`:

- `main.rs`
- `app.rs`
- `editor.rs`
- `ui.rs`
- `events.rs`
- `theme.rs`

Se necessário, pode adicionar outros módulos, mas somente se houver justificativa técnica clara.

## Regras de implementação

- O projeto deve compilar sem erros.
- O código deve ser simples e legível.
- Cada módulo deve ter responsabilidade única.
- Não misturar entrada de eventos com renderização.
- Não implementar antecipadamente funcionalidades ainda não aprovadas.
- Não alterar os atalhos ou comportamentos definidos em `PROJECT_RULES.md`.

## Comportamento mínimo esperado

Ao iniciar o editor, o Cursor deve entregar uma aplicação com:

- tela TUI funcionando;
- estado inicial carregado;
- área central de edição ou placeholder compatível;
- rodapé com indicação de atalhos básicos;
- encerramento limpo do terminal ao sair.

## Ordem de trabalho sugerida

1. Criar `Cargo.toml` mínimo.
2. Criar `main.rs` com setup/teardown do terminal.
3. Criar `app.rs` com o estado principal.
4. Criar `ui.rs` com renderização inicial.
5. Criar `events.rs` com loop de eventos.
6. Criar `theme.rs` com tema inicial.
7. Garantir que tudo compile.

## Critérios de aceite

A entrega será aceita quando:

- o projeto compilar;
- o aplicativo abrir no terminal;
- a interface básica for renderizada;
- o usuário puder sair de forma limpa;
- a estrutura do projeto estiver pronta para evolução futura.

## Observação para o Cursor

A prioridade agora é criar uma base técnica sólida, não um editor completo. A implementação deve ser incremental e alinhada às regras do projeto.
