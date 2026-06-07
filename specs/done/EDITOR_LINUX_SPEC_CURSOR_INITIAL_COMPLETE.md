# Especificação para o Cursor — Editor Linux Inicial Completa

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## Objetivo

Definir uma nova especificação de implementação para continuar o projeto Editor Linux a partir da V1 compilável, cobrindo funcionalidades ainda não implementadas e corrigindo a organização de menus, status bar, modo de edição e persistência local.

## Fonte de verdade

O Cursor deve seguir esta ordem:

1. `PROJECT_RULES.md`
2. Esta especificação.
3. `PROJECT_STATUS.md`
4. `PROJECT_TIMELINE.md`
5. Documentos arquivados em `docs/done/` apenas como referência.

## Escopo desta spec

### Implementar agora
- Menus visíveis com atalhos ao lado dos itens.
- Barra de status com encoding, linha, coluna, seleção, tamanho e modo.
- Indicador explícito de Insert/Replace.
- Abrir arquivo local.
- Salvar arquivo local.
- Confirmação para saída com alterações.
- Confirmação para sobrescrita de arquivo.
- Ajustes visuais do layout para ficar mais próximo de um editor tradicional.

### Não implementar ainda
- Drive.
- Clipboard histórico completo.
- Terminal inferior completo.
- Seleção retangular avançada.
- Plugins.
- Sincronização remota.

## Layout obrigatório

A interface deve conter:

- barra superior;
- menu principal;
- área central de edição;
- barra de status inferior;
- atalhos ao lado dos itens de menu;
- estado do editor na barra de status.

## Menus obrigatórios

### Arquivo
- Novo `Ctrl+N`
- Abrir `Ctrl+O`
- Abrir Recente
- Salvar `Ctrl+S`
- Salvar Como `Ctrl+Shift+S`
- Fechar `Ctrl+W`
- Sair `Ctrl+Q`

### Editar
- Desfazer `Ctrl+Z`
- Refazer `Ctrl+Y`
- Recortar `Ctrl+X`
- Copiar `Ctrl+C`
- Colar `Ctrl+V`
- Colar Anterior `Ctrl+Shift+V`
- Selecionar Tudo `Ctrl+A`
- Buscar `Ctrl+F`
- Substituir `Ctrl+H`

### Exibir
- Tema Escuro
- Tema Claro
- Tema Azul Clássico
- Alternar Painel Lateral
- Alternar Terminal Inferior

## Barra de status

A barra inferior deve mostrar:

- UTF-8.
- Linha.
- Coluna.
- Seleção.
- Tamanho do documento.
- Modo `Insert` ou `Replace`.
- Indicação de TTY/SSH quando relevante.
- Estado de mouse quando relevante.

## Modo de edição

O modo padrão é Insert.

Regras:

- O modo ativo precisa aparecer na UI.
- O cursor deve refletir o modo sempre que possível.
- A mudança de modo deve ser rastreável.

## Persistência local

A spec exige implementação de:

- abrir arquivo local;
- salvar arquivo local;
- aviso de alterações não salvas;
- atualização do estado após salvar.

## Modais obrigatórios

- sair com alterações não salvas;
- fechar documento com alterações não salvas;
- sobrescrever arquivo existente;
- cancelar ação de risco.

## Compatibilidade Local, SSH e TTY

O editor deve funcionar em terminal local e em sessões remotas via SSH sempre que o terminal cliente e o servidor repassarem corretamente entrada de teclado, mouse e sequências ANSI. O comportamento com mouse, seleção e cores pode variar conforme o emulador de terminal, o cliente SSH e a configuração do ambiente, então o aplicativo deve degradar com elegância quando algum recurso não estiver disponível. O teclado deve continuar sendo o caminho principal para navegação, edição, salvamento e saída, garantindo utilidade mesmo em TTYs mais simples ou ambientes remotos restritos.

## Ordem de implementação

1. Ajustar a estrutura visual para menus e status bar.
2. Implementar abertura e salvamento local.
3. Implementar atalhos de menu com rótulos visíveis.
4. Implementar modais de risco.
5. Atualizar documentação de projeto.

## Critérios de aceite

- menus visíveis com atalhos;
- barra de status com contexto do editor;
- modo Insert visível;
- abrir e salvar funcionando localmente;
- modais de risco funcionando;
- layout mais próximo do esperado.
