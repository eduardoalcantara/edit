# PROJECT_RULES — Editor Linux

**Autor:** Perplexity AI  
**Data:** 2026-06-06  
**Versão:** 1.0

## Regras gerais

- O editor deve ser previsível, simples e humano.
- A UX deve priorizar clareza e prevenção de erro.
- Modais são obrigatórios em ações destrutivas ou de risco.
- Atalhos devem ser consistentes e documentados.
- O comportamento deve ser estável entre sessões e estados.

## Regras de UI

- Menus devem ser visíveis.
- Rodapé deve exibir atalhos úteis e estado atual.
- O tema deve ser sempre explícito.
- O painel lateral e o terminal inferior devem poder ser alternados.
- A interface não deve esconder ações essenciais atrás de gestos obscuros.

## Regras de UX

- `Ctrl+S` salva.
- `Ctrl+O` abre.
- `Ctrl+Q` sai com confirmação se houver alterações.
- `Ctrl+T` alterna o terminal inferior.
- `Ctrl+Alt+LeftMouse` ativa seleção retangular.
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
