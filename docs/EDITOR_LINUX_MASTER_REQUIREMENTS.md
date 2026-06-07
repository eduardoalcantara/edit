# Editor Linux — Requisitos Mestres

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## Propósito

Este documento consolida todos os requisitos definidos até agora para o Editor Linux. Ele serve como referência permanente para evitar perda de contexto e para originar specs incrementais de implementação para o Cursor.

## Visão do produto

O Editor Linux será um editor de terminal para usuários comuns, com UX previsível, menus visíveis, atalhos familiares, proteção contra perda de trabalho e compatibilidade com terminal local, SSH e TTY.

## Direção do projeto

- O Editor Linux é um repositório separado do editor Web.
- A implementação deve ser feita em Rust.
- A UI deve ser TUI moderna, com foco em clareza e padronização.
- O Cursor implementa o código localmente com base nas specs derivadas deste documento.
- O foco é usar specs incrementais, não tentar fechar o produto inteiro de uma vez.

## Stack escolhida

- Linguagem: Rust.
- UI TUI: Ratatui.
- Backend de terminal: Crossterm.
- Editor de texto base: `tui-textarea` ou equivalente evolutivo.
- Persistência: filesystem local.
- Configuração: arquivos simples e explícitos.

## Estrutura do repositório

O repositório deve conter, no mínimo:

- `specs/`
- `docs/`
- `scripts/`
- `tests/`
- `src/`

Arquivos de governança:

- `.cursorrules`
- `PROJECT_RULES.md`
- `PROJECT_STATUS.md`
- `PROJECT_TIMELINE.md`
- `README.md`

## Arquitetura de módulos

Os módulos iniciais esperados em `src/` são:

- `main.rs`
- `app.rs`
- `editor.rs`
- `ui.rs`
- `events.rs`
- `theme.rs`
- `menus.rs`
- `modal.rs`
- `clipboard.rs`
- `recent.rs`
- `terminal_pane.rs`

## Princípios de UX

- A interface deve ser clara e padronizada.
- Menus devem ser visíveis.
- Atalhos devem ser previsíveis.
- O usuário deve sempre saber em que modo está.
- A aplicação deve evitar perda acidental de dados.
- O teclado deve sempre funcionar como caminho principal.
- O mouse é um recurso importante, mas não pode ser obrigatório.

## Layout geral da interface

A interface deve conter:

- barra superior com nome do app, documento e tema;
- menu principal visível;
- barra de abas;
- área central de edição;
- barra de status inferior;
- painel lateral opcional;
- terminal inferior opcional;
- rodapé de ajuda, apenas quando apropriado.

## Menus obrigatórios

### Arquivo
- Novo.
- Abrir.
- Abrir Recente.
- Salvar.
- Salvar Como.
- Fechar.
- Sair.

### Editar
- Desfazer.
- Refazer.
- Recortar.
- Copiar.
- Colar.
- Colar Anterior.
- Selecionar Tudo.
- Buscar.
- Substituir.

### Exibir
- Tema Escuro.
- Tema Claro.
- Tema Azul Clássico.
- Tema Personalizado.
- Alternar Painel Lateral.
- Alternar Terminal Inferior.

## Atalhos consolidados

### Arquivo
- `Ctrl+N`: novo arquivo.
- `Ctrl+O`: abrir arquivo.
- `Ctrl+S`: salvar.
- `Ctrl+Shift+S`: salvar como.
- `Ctrl+W`: fechar aba.
- `Ctrl+Q`: sair.

### Edição
- `Ctrl+A`: selecionar tudo.
- `Ctrl+C`: copiar.
- `Ctrl+V`: colar item mais recente.
- `Ctrl+X`: recortar.
- `Ctrl+F`: buscar.
- `Ctrl+H`: substituir.
- `Ctrl+Shift+V`: abrir histórico de colagem.
- `Ctrl+Alt+LeftMouse`: seleção retangular.

### Navegação e produtividade
- `Ctrl+T`: alternar terminal inferior.
- `Alt+R`: abrir recentes.

## Sistema de temas

O editor deve ter pelo menos estes temas nativos:

- Escuro.
- Claro.
- Azul clássico inspirado no EDIT.EXE.

Também deve aceitar tema customizado via configuração.

## Status bar obrigatória

A barra inferior deve mostrar, no mínimo:

- encoding, padrão `UTF-8`;
- linha atual;
- coluna atual;
- estado de seleção;
- tamanho do documento;
- modo atual (`Insert` ou `Replace`);
- indicação de TTY/SSH quando relevante;
- estado de mouse quando relevante.

A barra de status é para contexto e estado, não para substituir menus.

## Modo de edição

- O modo padrão deve ser Insert.
- O modo ativo deve aparecer claramente na UI.
- A mudança para Replace deve ser visível quando existir.
- O cursor deve refletir o modo sempre que possível no terminal em uso.

## Persistência e arquivos

O editor deve suportar:

- abrir arquivo local;
- salvar arquivo local;
- salvar como;
- aviso de alterações não salvas;
- atualização do estado após salvar;
- atualização de recentes após abertura/salvamento.

## Recentes

O menu `Arquivo > Abrir Recente` deve listar os últimos 10 arquivos.

Regras:

- ordenação do mais recente para o mais antigo;
- itens ausentes podem ser mostrados como indisponíveis;
- o usuário pode limpar o histórico;
- abrir um item recente deve movê-lo para o topo.

## Clipboard interno

O editor deve manter histórico interno de pelo menos 5 itens.

Regras:

- `Ctrl+C` adiciona o texto selecionado ao histórico;
- `Ctrl+X` adiciona e remove do buffer;
- `Ctrl+V` cola o item mais recente;
- `Editar > Colar` abre o histórico;
- `Editar > Colar Anterior` cola o item anterior;
- o histórico deve permanecer consistente na sessão atual.

## Modais

O editor deve usar caixas de diálogo modais para ações sensíveis.

Casos obrigatórios:

- sair com alterações não salvas;
- fechar aba com alterações não salvas;
- sobrescrever arquivo existente;
- descartar alterações;
- cancelar operações de risco.

## Seleção retangular

A seleção retangular deve permitir selecionar por linhas e colunas.

Regras:

- ativação por `Ctrl+Alt+LeftMouse`;
- deve permitir copiar, recortar e editar blocos;
- deve ser coerente com editores visuais modernos.

## Terminal inferior

O painel inferior deve funcionar como terminal embutido.

Regras:

- alternar com `Ctrl+T`;
- abrir no diretório do arquivo atual;
- permitir executar comandos sem sair do editor;
- não destruir o estado da edição ao ser fechado.

## Compatibilidade Local, SSH e TTY

O editor deve funcionar em terminal local e em sessões remotas via SSH sempre que o terminal cliente e o servidor repassarem corretamente entrada de teclado, mouse e sequências ANSI. O comportamento com mouse, seleção e cores pode variar conforme o emulador de terminal, o cliente SSH e a configuração do ambiente, então o aplicativo deve degradar com elegância quando algum recurso não estiver disponível. O teclado deve continuar sendo o caminho principal para navegação, edição, salvamento e saída, garantindo utilidade mesmo em TTYs mais simples ou ambientes remotos restritos.

## Regras de implementação

- Não inventar comportamento fora das specs.
- Não alterar atalhos sem atualizar a documentação.
- Não misturar responsabilidades de módulos.
- Não esconder ações essenciais atrás de fluxos obscuros.
- Atualizar `PROJECT_STATUS.md` e `PROJECT_TIMELINE.md` sempre que houver mudanças relevantes.

## Modelo de trabalho

O trabalho será feito em fases incrementais:

1. Este documento consolida os requisitos.
2. Cada fase futura gera uma spec própria para o Cursor.
3. O Cursor implementa somente o que estiver naquela spec.
4. O Arquiteto/Supervisor revisa e consolida as mudanças.

## Estado atual do projeto

- A base compilável inicial já foi entregue.
- A organização visual precisa ser refinada.
- Menus e barra de status precisam ser formalizados.
- Persistência local ainda precisa ser expandida.
- O projeto agora deve avançar por specs incrementais derivadas deste documento.
