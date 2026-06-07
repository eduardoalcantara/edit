# Especificação para o Cursor — Menu Formatar: Codificação e Tabulação

**Autor:** Perplexity AI
**Data:** 2026-06-06
**Versão:** 1.0

## Objetivo

Definir a implementação incremental do menu **Formatar** para o Editor Linux, com foco em codificação de arquivos e tabulação, inspirada na praticidade de editores como o Notepad++.

## Escopo

### Deve incluir
- Submenu de codificação.
- Submenu de tabulação.
- Exibição do estado atual na interface.
- Integração com o editor atual sem abrir janela complexa de configuração.

### Não deve incluir
- Motor completo de conversão automática entre todos os encodings possíveis.
- Interface de configuração avançada.
- Detecção heurística complexa de encoding nesta etapa.

## Menu Formatar

O menu **Formatar** deve conter, no mínimo, dois submenus:

- **Codificação**.
- **Tabulação**.

## Submenu: Codificação

O submenu de codificação deve permitir ao usuário lidar com o arquivo atual de forma semelhante ao fluxo de editores textuais clássicos.

### Ações esperadas
- Mostrar a codificação atual do documento.
- Reabrir o arquivo como outra codificação.
- Converter o conteúdo do arquivo para outra codificação ao salvar.
- Expor opções comuns como:
  - ANSI.
  - UTF-8.
  - UTF-8 sem BOM.
  - UTF-16 LE.
  - UTF-16 BE.
  - ISO-8859-1 ou equivalente aplicável.

### Regra conceitual
- **Abrir como / Reinterpretar**: altera a forma como os bytes são lidos.
- **Converter / Salvar como**: preserva o texto visível e altera os bytes gravados.

### Requisitos de interface
- A codificação ativa deve aparecer na barra de status ou em área equivalente.
- O submenu precisa ser acessível sem sair da edição.
- O usuário deve perceber claramente se a mudança é temporária ou de conversão real.

## Submenu: Tabulação

O submenu de tabulação deve permitir escolher rapidamente o comportamento da tecla TAB.

### Opções obrigatórias
- 2 espaços.
- 4 espaços.
- 8 espaços.
- Tab literal.

### Regras
- A escolha deve afetar imediatamente a entrada da tecla TAB.
- O estado atual de tabulação deve ser visível na UI.
- A opção deve poder ser alterada por documento ou sessão.
- O usuário não deve precisar abrir uma janela de configuração avançada.

## Comportamento esperado

### Codificação
- A mudança de leitura ou conversão deve afetar apenas o arquivo atual, salvo se o usuário aplicar a mesma escolha globalmente depois.
- O editor deve evitar perda silenciosa de conteúdo ao trocar de encoding.
- Mudanças sensíveis devem ser acompanhadas de confirmação quando necessário.

### Tabulação
- Ao pressionar TAB, o editor deve inserir o valor configurado no submenu.
- A opção de tabulação deve ser refletida no comportamento de edição imediatamente.

## Critérios de aceite

A implementação será considerada válida quando:

- existir um menu Formatar visível;
- existirem os submenus Codificação e Tabulação;
- o usuário conseguir ver a codificação ativa;
- o usuário conseguir escolher entre 2, 4, 8 espaços ou tab literal;
- a escolha de tabulação afetar a tecla TAB;
- a documentação e o status do projeto forem atualizados.

## Observação para o Cursor

Esta spec deve ser implementada de forma incremental e coerente com o restante do editor. O Cursor não deve tentar resolver todos os casos de encoding do mundo de uma vez; o foco é entregar a base funcional e clara, com UX previsível.
