# Referência de UX: Turbo Vision

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## Propósito

Registrar os elementos de UX do Turbo Vision que inspiram o Editor Linux, para servir como referência formal de design de interface, comportamento visual e padronização de interação.

## Visão geral

Turbo Vision foi um framework TUI clássico orientado a objetos, conhecido por levar ao terminal uma experiência parecida com interface gráfica, com janelas, menus, diálogos e controles bem definidos. [web:331][web:334]

## Características visuais marcantes

- Bordas desenhadas com caracteres ASCII ou pseudográficos de linha, formando janelas, modais e painéis de forma clara. [web:331][web:334][web:348]
- Janelas com aparência destacada e molduras visíveis, reforçando separação de áreas na tela. [web:331][web:350]
- Diálogos modais bem delimitados, com foco em confirmação e interação direta. [web:331][web:335]
- Menus pull-down e botões visuais que ajudavam o usuário a identificar ações clicáveis ou acionáveis. [web:331][web:335]

## Cores e contraste

Uma das marcas mais lembradas do Turbo Vision é o uso de cores invertidas ou fortemente contrastadas para títulos de janelas, menus e botões, sinalizando visualmente que esses elementos eram interativos. [web:354][web:357][web:366]

## Interação

- Suporte a mouse em ambiente MS-DOS, o que era avançado para a época. [web:331][web:335]
- Menu principal visível e navegação previsível. [web:331][web:334]
- Diálogos com confirmação e controles claros. [web:331][web:335]
- Sistema de eventos orientado a objetos, permitindo ações consistentes e extensíveis. [web:332][web:335]

## Princípios que queremos reaproveitar

Para o Editor Linux, os princípios úteis são:

- molduras visíveis em torno de áreas funcionais;
- títulos com destaque visual forte;
- menus claramente identificáveis;
- botões e comandos com aparência de elementos acionáveis;
- status bar informativa, não escondida;
- modais como mecanismo de segurança e clareza.

## Adaptação para o Editor Linux

Sim, podemos adotar essa lógica no nosso sistema:

- janelas e painéis com bordas em caracteres ASCII/pseudográficos;
- títulos com estilo visual forte, incluindo contraste ou inversão semântica;
- menus e botões com estilo diferenciado para indicar interatividade;
- modais com bordas e foco destacados;
- barra de status dedicada a contexto e estado.

A adaptação deve preservar legibilidade em TTY, SSH e terminais com suporte limitado, sem depender exclusivamente de cores para comunicação.

## Regra de design derivada

No Editor Linux, qualquer elemento acionável deve parecer acionável. Isso significa:

- botões e comandos devem ter estilo consistente;
- menus devem ser visíveis e distinguíveis;
- títulos de janelas e modais devem chamar atenção;
- o foco deve ser claro;
- o usuário deve reconhecer a interação sem adivinhar.

## Relação com o projeto

Esta referência existe para manter a direção de UX coerente com o que você quer construir: um editor de terminal humano, previsível, elegante e com linguagem visual clara.
