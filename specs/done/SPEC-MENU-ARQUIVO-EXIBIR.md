# Especificação para o Cursor — Menus Arquivo e Exibir

**Autor:** Perplexity AI
**Data:** 2026-06-06
**Versão:** 1.0

## Objetivo

Definir a implementação incremental dos menus **Arquivo** e **Exibir** para o Editor Linux, com comandos clássicos de editores textuais, priorizando clareza, consistência e UX previsível.

## Escopo

### Deve incluir
- Menu Arquivo com ações de arquivo.
- Menu Exibir com ações visuais e de apresentação.
- Itens com atalhos visíveis ao lado do rótulo, quando aplicável.
- Estado da UI refletido na barra de status ou em área equivalente.

### Não deve incluir
- Preferências avançadas em janelas complexas.
- Sistema completo de perfis por linguagem.
- Renderização gráfica fora da TUI.

## Menu Arquivo

O menu **Arquivo** deve conter os comandos clássicos esperados em editores de texto.

### Itens obrigatórios
- Novo `Ctrl+N`.
- Abrir `Ctrl+O`.
- Recentes.
- Salvar `Ctrl+S`.
- Salvar Como `Ctrl+Shift+S`.
- Fechar `Ctrl+W`.
- Sair `Ctrl+Q`.

### Regras
- **Novo** cria um documento em branco.
- **Abrir** seleciona um arquivo local.
- **Recentes** mostra os últimos arquivos usados.
- **Salvar** grava o documento atual.
- **Salvar Como** permite salvar com novo nome ou caminho.
- **Fechar** encerra a aba ou documento atual.
- **Sair** encerra o aplicativo com confirmação se houver alterações não salvas.

### Requisitos de UX
- Menus devem exibir os atalhos ao lado dos comandos.
- A ação de saída deve proteger contra perda de trabalho.
- O menu Recentes deve ser acessível sem precisar abrir diálogo extra complexo.

## Menu Exibir

O menu **Exibir** deve concentrar opções de apresentação, leitura visual e layout da interface.

### Submenu: Zoom
- Zoom In.
- Zoom Out.
- Reset Zoom.

### Regra de uso do Zoom
- O zoom deve ser entendido como ajuste de acessibilidade do terminal ou ajuste de densidade visual da interface, não como zoom gráfico interno de uma GUI.
- O editor deve permanecer funcional em qualquer nível de zoom suportado pelo terminal.

### Submenu: Word Wrap
- Ativar quebra automática.
- Desativar quebra automática.

### Regra de Word Wrap
- Quando ativado, a linha longa deve quebrar visualmente dentro da área de edição.
- O estado atual deve ser claramente visível.

### Submenu: Mostrar
- Símbolos.
- Espaços.
- Tabs.
- Fim de linha.
- Tudo.

### Regras de Mostrar
- A exibição de símbolos serve para revelar caracteres de formatação e controle.
- A ativação deve ser fácil de alternar e não exigir telas de configuração.

### Submenu: Painel Lateral
- Mostrar painel lateral.
- Ocultar painel lateral.

### Submenu: Terminal
- Mostrar terminal inferior.
- Ocultar terminal inferior.

### Submenu: Rodapé
- Mostrar rodapé.
- Ocultar rodapé.

### Submenu: Temas
- Escuro.
- Claro.
- Azul clássico.
- Personalizado.

### Submenu: Colunas
- 80.
- 120.
- 160.
- Ilimitado.

### Regras de Colunas
- A coluna funciona como guia visual opcional.
- Pode ser usada para referência de largura de linha, quebra ou alinhamento visual.
- Quando o limite estiver ativo, ele deve aparecer na UI.
- A opção “Ilimitado” desativa a referência visual fixa.

## Comportamento esperado

### Arquivo
- Criar, abrir, salvar e fechar devem funcionar de forma consistente.
- Recentes deve atualizar conforme o uso real.
- Sair sempre deve proteger o usuário de perda de dados não salvos.

### Exibir
- As opções visuais devem alterar a apresentação da interface sem modificar o conteúdo do arquivo.
- O estado de cada opção relevante deve ficar evidente na UI.
- O menu deve permanecer utilizável em terminal local e remoto.

## Critérios de aceite

A implementação será considerada válida quando:

- o menu Arquivo existir com os comandos clássicos;
- o menu Exibir existir com zoom, word wrap, mostrar, painel lateral, terminal, rodapé, temas e colunas;
- os atalhos de Arquivo estiverem visíveis;
- os estados visuais estiverem refletidos na interface;
- a proteção contra perda de dados estiver funcionando;
- a documentação do projeto for atualizada.

## Observação para o Cursor

Esta spec deve ser implementada de maneira incremental. O foco inicial deve ser criar os menus, expor os comandos e integrar o estado visual antes de avançar para refinamentos mais sofisticados.
