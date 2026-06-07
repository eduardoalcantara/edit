# Especificação para o Cursor — Menu Editar

**Autor:** Perplexity AI
**Data:** 2026-06-07
**Versão:** 1.0

## Objetivo

Definir a implementação incremental do menu **Editar** para o Editor Linux, incluindo histórico de edição, área de transferência, colagem anterior, busca, seleção e seleção em bloco, com comportamento inspirado em editores clássicos.

## Escopo

### Deve incluir
- Histórico de edição.
- Área de transferência.
- Submenu de colagem anterior com visualização dos itens.
- Busca e substituição.
- Seleção normal.
- Seleção em bloco.
- Operações clássicas de edição.

### Não deve incluir
- Transformações de texto avançadas ainda não aprovadas.
- Macros.
- Automação complexa de refatoração.

## Menu Editar

O menu **Editar** deve conter os comandos clássicos esperados de um editor de texto.

### Histórico de edição
- Desfazer `Ctrl+Z`.
- Refazer `Ctrl+Y`.

### Área de transferência
- Recortar `Ctrl+X`.
- Copiar `Ctrl+C`.
- Colar `Ctrl+V`.
- Colar Anterior `Ctrl+Shift+V`.

## Submenu: Colar Anterior

O comando **Colar Anterior** deve abrir um submenu com os itens mais recentes do histórico de clipboard, em vez de executar apenas uma ação direta.

### Regras do submenu
- Exibir os 5 itens mais recentes do histórico.
- Cada item deve mostrar os primeiros 20 caracteres do conteúdo, ou menos se o trecho for menor.
- O item mais recente deve aparecer no topo.
- Ao selecionar um item, o conteúdo correspondente deve ser colado imediatamente na posição ativa do editor.
- O submenu deve ser navegável por teclado e mouse.

### Regras de uso
- O submenu não deve exigir confirmação extra para colar.
- O preview textual deve ajudar o usuário a identificar rapidamente o conteúdo.

## Seleção
- Selecionar Tudo `Ctrl+A`.
- Cancelar Seleção `Esc`.
- Seleção em Bloco `Ctrl+Alt+Mouse`.

## Submenu: Seleção em Bloco

O editor deve suportar seleção retangular, em bloco, no estilo de editores como o Notepad++.

### Regras
- A seleção em bloco deve funcionar por linhas e colunas.
- A ativação deve ocorrer com `Ctrl+Alt+Mouse`.
- A seleção deve permitir copiar, recortar e colar blocos de texto.
- A região selecionada deve permanecer visualmente distinguível.
- O comportamento deve ser consistente no textarea principal.

## Busca e substituição
- Buscar `Ctrl+F`.
- Buscar Próximo `F3`.
- Buscar Anterior `Shift+F3`.
- Substituir `Ctrl+H`.

### Regras
- A busca deve operar sobre o documento atual.
- O próximo e o anterior devem navegar pelos resultados de forma previsível.
- A substituição deve ser contextual e segura.

## Comportamento esperado

### Clipboard
- `Ctrl+C` deve salvar a seleção atual no histórico de clipboard.
- `Ctrl+X` deve salvar e remover a seleção atual.
- `Ctrl+V` deve colar o item mais recente.
- O histórico de clipboard deve manter no mínimo 5 itens.

### Colar Anterior
- O submenu deve mostrar 5 entradas reutilizáveis com preview curto.
- A seleção de qualquer item deve colá-lo imediatamente.
- O item mais recente deve ser sempre o primeiro.

### Seleção em bloco
- A seleção retangular deve ser precisa e previsível.
- O editor deve deixar claro quando o modo de bloco está ativo.

## Critérios de aceite

A implementação será considerada válida quando:

- o menu Editar existir com os comandos clássicos;
- o submenu Colar Anterior mostrar 5 itens do histórico com preview curto;
- cada item do submenu puder ser colado diretamente;
- a seleção em bloco funcionar com `Ctrl+Alt+Mouse`;
- copiar, recortar e colar funcionarem com o histórico;
- a busca e substituição estiverem disponíveis;
- a documentação e o status do projeto forem atualizados.

## Observação para o Cursor

Esta spec deve ser implementada de forma incremental. O foco inicial deve ser a estrutura do menu, o histórico de clipboard e a seleção em bloco, deixando refinamentos avançados para fases posteriores.
