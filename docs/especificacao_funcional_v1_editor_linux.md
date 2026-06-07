# Especificação Funcional v1 — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## 1. Objetivo

Definir a primeira versão funcional do Editor Linux do projeto Editor Minimalista, priorizando usabilidade, previsibilidade, segurança de edição e uma experiência compatível com usuários comuns de terminal.

## 2. Visão do Produto

O Editor Linux será um aplicativo de terminal com interface textual moderna, menus visíveis, suporte a mouse, atalhos previsíveis e comportamento inspirado na ergonomia do EDIT.EXE, evitando a lógica confusa de editores tradicionais que exigem memorização excessiva.

## 3. Princípios de UX

- A interface deve ser clara e padronizada.
- As ações principais devem estar visíveis em menus.
- Atalhos devem ser intuitivos e consistentes.
- O editor deve prevenir perda acidental de dados.
- O usuário deve poder operar com teclado, mouse ou ambos.

## 4. Stack Técnica

- Linguagem: Rust.
- UI TUI: Ratatui.
- Backend de terminal: Crossterm.
- Editor de texto base: tui-textarea ou componente próprio evolutivo.
- Persistência: sistema local de arquivos.
- Clipboard interno: histórico próprio dentro do editor.

## 5. Layout da Tela

A tela principal deve conter:

- Barra superior com título, nome do arquivo e estado da sessão.
- Barra de abas com arquivos abertos.
- Área central de edição.
- Painel lateral opcional com arquivos e navegação.
- Rodapé com atalhos e mensagens de status.
- Painel inferior opcional para terminal integrado.

## 6. Atalhos Básicos

### Arquivo
- Ctrl+N: novo arquivo.
- Ctrl+O: abrir arquivo.
- Ctrl+S: salvar.
- Ctrl+Shift+S: salvar como.
- Ctrl+W: fechar aba atual.
- Ctrl+Q: sair.

### Edição
- Ctrl+A: selecionar tudo.
- Ctrl+C: copiar.
- Ctrl+V: colar item mais recente.
- Ctrl+X: recortar.
- Ctrl+F: buscar.
- Ctrl+H: substituir.
- Ctrl+Alt+LeftMouse: seleção retangular por bloco.

### Terminal
- Ctrl+T: abrir ou fechar terminal inferior.

## 7. Menus

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

## 8. Temas

O editor deve suportar pelo menos os seguintes temas nativos:

- Escuro.
- Claro.
- Azul clássico inspirado no EDIT.EXE.

Também deve permitir tema personalizado por configuração do usuário. As cores devem ser configuradas por função semântica, não apenas por valor fixo, para permitir manutenção simples e coerente.

## 9. Abrir Recente

O menu Arquivo > Abrir Recente deve listar os últimos 10 arquivos abertos pelo usuário dentro do editor.

Regras:

- A lista deve ser ordenada do mais recente para o mais antigo.
- Arquivos ausentes ainda podem aparecer, mas devem ser marcados como indisponíveis.
- Deve existir opção para limpar o histórico.
- Abrir um item recente deve carregar o arquivo e atualizá-lo no topo da lista.

## 10. Clipboard Interno

O editor deve manter um histórico interno dos últimos 5 itens copiados ou recortados.

Regras:

- Ctrl+C adiciona o texto selecionado ao histórico.
- Ctrl+X adiciona o texto selecionado ao histórico e remove do buffer.
- Ctrl+V cola o item mais recente.
- Editar > Colar abre o histórico de colagem.
- Editar > Colar Anterior cola o item anterior ao item atual.
- O histórico deve ser preservado enquanto a sessão do editor estiver ativa, com persistência opcional posterior.

## 11. Modais

O editor deve usar caixas de diálogo modais para ações sensíveis.

Casos obrigatórios:

- Sair com alterações não salvas.
- Fechar aba com alterações não salvas.
- Sobrescrever arquivo existente.
- Descarta alterações.
- Cancelar operação em andamento.

Cada modal deve oferecer opções claras e diretas, com foco em evitar erro por distração.

## 12. Seleção Retangular

A seleção retangular deve permitir escolher um bloco por linhas e colunas.

Regras:

- O modo é ativado por Ctrl+Alt+LeftMouse.
- O bloco selecionado pode ser copiado, cortado ou editado.
- O comportamento deve ser consistente com editores visuais modernos.

## 13. Terminal Inferior

O editor deve oferecer um painel inferior com terminal embutido.

Regras:

- Atalho padrão: Ctrl+T.
- O terminal deve abrir no diretório do arquivo atual.
- O usuário deve poder executar comandos sem sair do editor.
- O painel deve poder ser fechado sem impactar o estado do arquivo aberto.

## 14. Regras de Salvamento

- O editor deve evitar perda de alterações.
- Fechar ou sair com conteúdo modificado deve gerar confirmação.
- O salvamento deve atualizar o arquivo atual e o histórico de recentes.
- O usuário deve sempre saber se o arquivo foi modificado.

## 15. Estado da Sessão

O editor deve manter, pelo menos durante a execução:

- arquivo atual;
- lista de abas abertas;
- tema ativo;
- histórico de recentes;
- histórico de clipboard;
- estado do terminal inferior;
- estado do modal atual.

## 16. Critérios de Aceite

A v1 será considerada válida quando:

- for possível editar, salvar, abrir e fechar arquivos;
- houver menus visíveis e atalhos básicos funcionando;
- existirem temas nativos e suporte a tema customizado;
- o menu Abrir Recente listar os últimos 10 arquivos;
- o histórico de clipboard interno guardar 5 itens;
- os modais de confirmação funcionarem corretamente;
- a seleção retangular estiver disponível;
- o terminal inferior puder ser aberto com Ctrl+T.

## 17. Próximos Passos

Após esta versão, a evolução natural deve focar em:

- persistência de layout.
- integração com projeto maior.
- refinamento de busca e substituição.
- suporte a plugins.
- possível sincronização com frontend Web futuro.
