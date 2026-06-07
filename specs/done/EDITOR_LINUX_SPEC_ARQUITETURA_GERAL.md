# Especificação Geral de Arquitetura — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 1.0

## Objetivo

Corrigir a direção do projeto e estabelecer uma arquitetura de **editor de texto de verdade**, com core próprio de edição, sem dependência de widgets genéricos que limitem o comportamento esperado. O editor deve aceitar os requisitos já definidos pelo projeto, incluindo menus clássicos, busca, seleção em bloco, histórico de edição, tabulação, codificação e renderização em terminal.

## Princípios do projeto

- O produto final é o alvo; não há filosofia de MVP neste projeto.
- A especificação é a fonte de verdade para o comportamento do editor.
- O core de edição deve ser independente da UI.
- A UI TUI deve apenas renderizar e encaminhar eventos.
- Não deve haver retrabalho estrutural para “deixar compilando”.
- Toda funcionalidade relevante precisa existir no core ou em camadas explicitamente definidas.

## Visão geral da arquitetura

A arquitetura deve ser dividida em três camadas principais:

### 1. Core / Engine
Responsável por manipular o texto, o cursor, a seleção, o histórico, a codificação e os comandos de edição.

### 2. Controller / Estado de Edição
Responsável por traduzir eventos de teclado e mouse em comandos de edição, administrar modo atual, cursores, seleção em bloco, múltiplas seleções, viewport e comportamento de navegação.

### 3. View / Renderização
Responsável por desenhar o estado do editor no terminal com `ratatui` e receber eventos via `crossterm`.

## Base tecnológica

### Crates recomendadas
- `ropey` para o buffer de texto principal.
- `ratatui` para renderização da interface.
- `crossterm` para entrada de teclado, mouse e controle de terminal.
- Crates auxiliares apenas quando justificadas por necessidade clara do editor.

### Motivo da escolha
- `ropey` é adequado como backing text-buffer para editores de texto e indexação robusta por caracteres e linhas.
- `ratatui` fornece a camada TUI para layout e desenho.
- `crossterm` cobre eventos e controle de terminal de forma compatível com um editor TUI.

## Camada Core / Engine

O core deve ser o centro da aplicação. Ele não deve conhecer widgets, frames, blocos de layout ou estilos específicos da UI. Ele deve apenas operar sobre o conteúdo, o cursor e os estados lógicos do editor.

### Responsabilidades do core
- Armazenar o texto do documento.
- Inserir, remover e substituir conteúdo.
- Manter posições de cursor.
- Manter seleção normal e seleção em bloco.
- Fornecer operações de busca e substituição.
- Registrar e restaurar histórico de edição.
- Aplicar tabulação conforme configuração atual.
- Ler, alterar e salvar o estado de codificação.
- Expor APIs para a UI consumir o estado atual.

### O core não deve
- Desenhar nada.
- Conhecer posição física do terminal.
- Conhecer menus visuais.
- Depender de tamanho de janela além do que for necessário para cálculos lógicos.

## Estrutura de dados principal

### EditorMode
```rust
pub enum EditorMode {
    Normal,
    BlockSelection,
    MultiCursor,
    Search,
    Replace,
}
```

### Cursor
```rust
pub struct Cursor {
    pub char_idx: usize,
    pub virtual_col: usize,
    pub anchor: Option<usize>,
}
```

### BlockSelectionState
```rust
pub struct BlockSelectionState {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}
```

### Viewport
```rust
pub struct Viewport {
    pub top_line: usize,
    pub left_col: usize,
    pub width: u16,
    pub height: u16,
}
```

### EditorEngine
```rust
pub struct EditorEngine {
    pub text: ropey::Rope,
    pub cursors: Vec<Cursor>,
    pub block_selection: Option<BlockSelectionState>,
    pub mode: EditorMode,
    pub viewport: Viewport,
}
```

## Estado de edição

O estado de edição deve controlar o comportamento lógico do editor em um nível acima do buffer.

### Responsabilidades do estado de edição
- Traduzir entrada do usuário em comandos.
- Gerenciar múltiplos cursores.
- Definir se a edição está em modo normal, bloco, múltiplo ou busca.
- Coordenar viewport e cursor visível.
- Intermediar ações entre UI e core.

### Observação importante
A viewport não faz parte do texto. Ela é apenas uma janela lógica sobre o texto existente.

## Modelo de buffer

O buffer principal deve ser baseado em `ropey` para permitir edição eficiente e segura em UTF-8.

### Requisitos do buffer
- Indexação por caracteres.
- Indexação por linhas.
- Operações seguras para texto UTF-8.
- Capacidade de lidar com arquivos grandes.
- Suficiente flexibilidade para busca, seleção e substituição.

### Regra essencial
Não tratar o buffer como uma `String` simples para toda operação de edição. O editor precisa de uma estrutura própria de navegação e atualização.

## Cursor e navegação

### Requisitos mínimos
- Movimento para esquerda, direita, cima e baixo.
- Início e fim de linha.
- Início e fim do documento.
- Preservação de `virtual_col` ao mover verticalmente.
- Respeito ao espaço virtual além do fim da linha real.

### Regras
- O cursor principal deve ser sempre `cursors[0]`.
- Outros cursores, quando existirem, não devem quebrar o fluxo principal.
- A navegação vertical deve ser estável mesmo em linhas de comprimentos diferentes.

## Seleção

O editor precisa suportar pelo menos três formas de seleção:

### 1. Seleção normal
Seleção linear com âncora e posição atual.

### 2. Seleção em bloco
Seleção retangular por linhas e colunas, compatível com `Ctrl+Alt+Mouse`.

### 3. MultiCursor
Vários cursores ativos para edição simultânea, se habilitado.

### Regras gerais de seleção
- A seleção deve ser visível na UI.
- Copiar, cortar e colar devem respeitar a seleção atual.
- A seleção em bloco deve preservar sua geometria retangular.
- A UI deve indicar quando o editor está em modo de bloco.

## Busca e substituição

### Requisitos
- Buscar texto no documento atual.
- Buscar próximo e anterior.
- Substituir texto pontualmente.
- Substituir em sequência, se o menu oferecer essa opção.
- Destacar ocorrências encontradas.

### Regras
- A busca deve operar sobre o buffer do core.
- O resultado da busca deve ser navegável.
- A interface de busca não deve misturar lógica de UI com lógica textual.

## Histórico de edição

O editor deve manter histórico de alterações para desfazer e refazer.

### Requisitos
- Undo.
- Redo.
- Pilha ou mecanismo equivalente de operações.
- Comportamento estável após edições, recortes e colagens.

### Regras
- O histórico deve registrar alterações significativas.
- Uma operação de undo deve restaurar o estado lógico anterior.
- Redo deve reaplicar a operação desfeita.

## Área de transferência

O editor deve possuir integração com clipboard e histórico próprio de colagens.

### Requisitos
- Copiar seleção.
- Recortar seleção.
- Colar conteúdo atual.
- Histórico dos últimos itens copiados/colados para o submenu Colar Anterior.

### Regras
- O submenu de colagem anterior deve mostrar 5 itens com preview dos primeiros 20 caracteres.
- Ao clicar em um item do submenu, o conteúdo deve colar imediatamente no cursor ativo.
- O histórico deve ser mantido em memória enquanto a sessão existir, e opcionalmente persistido se o projeto decidir depois.

## Tabulação

O comportamento da tecla TAB deve ser configurável no menu Formatar.

### Opções
- 2 espaços.
- 4 espaços.
- 8 espaços.
- Tab literal.

### Regras
- O comportamento deve afetar a tecla TAB imediatamente.
- O estado atual deve ser visível.
- A decisão deve poder ser por documento ou por sessão.

## Codificação

O editor deve ser capaz de ler, interpretar e salvar arquivos em codificações suportadas pelo menu Formatar.

### Requisitos
- Identificar a codificação atual do documento, quando possível.
- Reabrir como outra codificação.
- Converter para outra codificação ao salvar.
- Exibir a codificação ativa na interface.

### Regras
- A operação de abrir/reabrir não é a mesma que converter/salvar.
- O editor deve evitar perda silenciosa de conteúdo.
- Mudanças potencialmente destrutivas devem solicitar confirmação.

## Menus clássicos

A arquitetura deve suportar os seguintes menus como parte do produto final:

### Arquivo
- Novo.
- Abrir.
- Recentes.
- Salvar.
- Salvar Como.
- Fechar.
- Sair.

### Exibir
- Zoom.
- Word Wrap.
- Mostrar símbolos.
- Painel lateral.
- Terminal.
- Rodapé.
- Temas.
- Colunas.

### Editar
- Desfazer.
- Refazer.
- Recortar.
- Copiar.
- Colar.
- Colar Anterior.
- Seleção em Bloco.
- Buscar.
- Substituir.

### Formatar
- Codificação.
- Tabulação.

## Requisitos de UI

### View / Renderização
- A UI deve desenhar apenas o que o estado manda.
- A renderização deve ser responsiva ao tamanho do terminal.
- O cursor físico deve refletir o cursor lógico.
- A seleção deve ser destacada visualmente.
- Estados como foco, busca, bloco e painéis precisam aparecer claramente.

### Regra fundamental
A UI não pode ser dona da verdade do editor. Ela apenas exibe e envia eventos ao estado de edição e ao core.

## Viewport e rolagem

### Requisitos
- Rolagem vertical e horizontal.
- Ajuste automático para manter o cursor visível.
- Respeito a colunas e linhas visíveis.
- Suporte a zoom do terminal ou ajustes de densidade visual quando aplicável.

### Regras
- A viewport deve seguir o cursor sem “pular” de forma confusa.
- A rolagem horizontal deve existir quando a linha exceder a área visível.
- A rolagem não pode corromper a seleção ou o cursor.

## Fluxo de eventos

### Entrada do usuário
1. O usuário pressiona uma tecla, combinações de teclas ou usa o mouse.
2. O controlador converte isso em comando lógico.
3. O core executa a operação no buffer e no estado.
4. A UI renderiza o novo estado.

### Regras
- Mouse e teclado devem ser tratados de forma consistente.
- Menus devem gerar comandos e não alterar texto diretamente.
- A lógica de edição deve ser testável independentemente da UI.

## Estrutura interna sugerida do projeto

```text
src/
  app/
    mod.rs
    state.rs
    commands.rs
  editor/
    mod.rs
    engine.rs
    cursor.rs
    selection.rs
    history.rs
    search.rs
    clipboard.rs
    encoding.rs
    tabulation.rs
    viewport.rs
  ui/
    mod.rs
    layout.rs
    render.rs
    menus.rs
    status.rs
  input/
    mod.rs
    keyboard.rs
    mouse.rs
  io/
    mod.rs
    load_save.rs
```

## Critérios de aceite

A arquitetura será considerada correta quando:

- o editor não depender de `tui-textarea` para funções centrais;
- o core de texto for separado da UI;
- a seleção em bloco existir no core;
- busca, undo/redo, clipboard e tabulação estiverem ligados ao estado de edição;
- os menus clássicos forem suportados por comandos reais;
- a UI apenas refletir o estado do editor;
- o projeto puder evoluir sem retrabalho estrutural.

## Decisões e limites

### Decisões
- `ropey` será o backbone textual.
- `ratatui` será a camada de desenho.
- `crossterm` será a camada de eventos e terminal.
- O editor deve ser construído para comportamento final, não para demonstração provisória.

### Limites atuais
- Esta spec define a base arquitetural e não toda micro-interação visual.
- Extensões futuras, como plugins, snippets e macros, devem ser especificadas depois.

## Próximos documentos derivados

A partir desta arquitetura, os próximos documentos devem detalhar:

- Core de texto e operações de buffer.
- Modelo de seleção e box selection.
- Histórico e undo/redo.
- Busca e substituição.
- Clipboard e colagem anterior.
- Menus e status bar.
- Sistema de persistência e codificação.

## Fechamento

Este documento redefine o projeto para que ele seja implementado como um editor de texto real, com responsabilidade clara entre camadas e sem dependência de widgets limitadores. O objetivo é preservar a coerência técnica, evitar retrabalho e respeitar os requisitos já aprovados ao longo do projeto.
