# PROJECT_RULES — Editor Linux

**Autor:** Perplexity AI
**Data:** 2026-06-06
**Versão:** 1.2

## Regras gerais

- O editor deve ser previsível, simples e humano.
- A UX deve priorizar clareza e prevenção de erro.
- Modais são obrigatórios em ações destrutivas ou de risco.
- Atalhos devem ser consistentes e documentados.
- O comportamento deve ser estável entre sessões e estados.
- Não inventar padrões, atalhos, fluxos de UI ou arquitetura sem autorização explícita.

## Regras de UI

- Menus devem ser visíveis.
- Rodapé deve exibir atalhos úteis e estado atual.
- O tema deve ser sempre explícito.
- O painel lateral e o terminal inferior devem poder ser alternados.
- A interface não deve esconder ações essenciais atrás de gestos obscuros.
- O design deve preservar legibilidade em terminais com suporte limitado.

## Regras de UI inspiradas no Turbo Vision

- A interface deve usar bordas visíveis em caracteres ASCII/pseudográficos para janelas, modais, painéis e áreas destacadas.
- Títulos de janelas, menus e botões devem ter contraste forte ou inversão semântica de cores para indicar foco e acionabilidade.
- Todo elemento clicável ou acionável deve parecer clicável ou acionável.
- Modais devem ser claramente separados do conteúdo principal, com foco visual evidente.
- Menus devem ser permanentes e legíveis, preferencialmente com atalhos exibidos ao lado do rótulo.
- A barra de status deve ser dedicada a contexto e estado do editor, não a lista principal de atalhos.
- O design deve preservar legibilidade em terminais com suporte limitado, sem depender exclusivamente de cor para comunicar função.
- O comportamento visual deve lembrar interfaces clássicas do Turbo Vision, sem copiar literalmente sua implementação.

## Regras de UX

- `Ctrl+S` salva.
- `Ctrl+O` abre.
- `Ctrl+Q` sai com confirmação se houver alterações.
- `Ctrl+T` alterna o terminal inferior.
- `Alt` + arraste botão esquerdo ativa seleção retangular (bloco).
- `Ctrl` + clique botão esquerdo adiciona cursor (multi-cursor).
- Deve existir proteção contra perda de trabalho.

## Regras de domínio

- Histórico de recentes: 10 arquivos.
- Histórico de clipboard: 5 itens.
- Temas nativos mínimos: escuro, claro, azul clássico.
- O editor deve aceitar tema customizado.

## Regras de nomenclatura

- Arquivos e módulos devem usar nomes descritivos e curtos.
- Funções devem expressar ação clara.
- Estruturas de domínio devem refletir o papel real do componente.
- Evitar siglas obscuras quando houver nome mais claro.

## Regras de implementação

- Preferir pequenas unidades coesas.
- Separar estado, renderização e entrada de eventos.
- Não misturar lógica de UI com persistência sem necessidade.
- Toda decisão importante deve ser documentada.
- Não substituir stack ou arquitetura por preferência pessoal.
- Não remover proteções de modal, confirmação ou prevenção de perda de dados.
- **Antes de implementar:** ler e cruzar `PROJECT_RULES.md`, `docs/EDITOR_LINUX_MASTER_REQUIREMENTS.md`, specs aplicáveis em `specs/` e `PROJECT_STATUS.md`. Não codar com base em suposição ou padrões inventados.
- **Menus:** usar subsistema interativo (`menus.rs`) com pull-down estilo Turbo Vision / EDIT.EXE; **proibido** substituir menus por linhas de texto estático com rótulos de comandos.

## Atalhos de menu (barra)

- `Alt+A`: abrir menu Arquivo.
- `Alt+E`: abrir menu Editar.
- `Alt+X`: abrir menu Exibir.
- `Alt+F`: abrir menu Formatar.
- `F10`: abrir menu Arquivo (foco no primeiro item).
