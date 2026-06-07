# PROJECT_RULES — Editor Linux (`edit`)

**Autor:** Perplexity AI  
**Data:** 2026-06-07  
**Versão:** 2.0

Regras estáveis do projeto. Para estado de implementação, ver `PROJECT_STATUS.md`; para histórico, `PROJECT_TIMELINE.md`; para detalhes de feature, specs em `specs/done/`.

---

## Regras gerais

- O editor deve ser previsível, simples e humano.
- A UX deve priorizar clareza e prevenção de erro.
- Modais são obrigatórios em ações destrutivas ou de risco.
- Atalhos devem ser consistentes e documentados.
- O comportamento deve ser estável entre sessões e estados.
- Não inventar padrões, atalhos, fluxos de UI ou arquitetura sem autorização explícita.
- **Antes de implementar:** ler e cruzar este arquivo, `docs/EDITOR_LINUX_MASTER_REQUIREMENTS.md`, specs aplicáveis em `specs/` e `PROJECT_STATUS.md`. Não codar com base em suposição.

---

## Identidade do produto

- Pacote e executável: **`edit`** (não `editor-linux`).
- Configuração persistente: **`edit.json`** na **mesma pasta** do executável (`edit` / `edit.exe`).
- Estrutura do JSON espelha a organização dos menus: **`arquivo`**, **`exibir`**, **`formatar`**.
- Migrar automaticamente `.edit/recent.json` e `.editor-linux/recent.json` na primeira execução, se `edit.json` não existir.
- Gravar configuração ao alterar opções persistidas, ao atualizar recentes e ao encerrar o programa.

---

## Regras de UI

- Menus devem ser visíveis e interativos (pull-down estilo Turbo Vision).
- Rodapé dedicado a **contexto e estado** do editor — não lista principal de F-keys.
- Help contextual do item de menu em foco aparece à **esquerda** do rodapé.
- Estado à **direita** do rodapé, nesta ordem: `Tam XXX/YYY | Pos XX/YY | modo | encoding | tab | Mem NMB` (memória só se toggle ativo).
- O tema deve ser sempre explícito e selecionável no menu Exibir → Temas.
- Painel lateral e terminal inferior devem poder ser alternados (placeholders permitidos até implementação completa).
- A interface não deve esconder ações essenciais atrás de gestos obscuros.
- Preservar legibilidade em terminais com suporte limitado; preferir UTF-8 onde o terminal suportar (`»`, `█`, `▀`, `√`).

---

## Regras de UI inspiradas no Turbo Vision

- Bordas visíveis em ASCII/pseudográficos para janelas, modais, painéis e áreas destacadas.
- **Editor e painéis internos:** borda `Plain`; **modais e menus dropdown:** borda `Double`.
- Títulos de janelas, menus e botões com contraste forte ou inversão semântica para foco e acionabilidade.
- Título do editor na borda: `[ nome do arquivo ]` (asterisco se dirty).
- Todo elemento clicável deve parecer acionável (hover em botões de modal).
- Modais claramente separados do conteúdo; sombra vertical `█`, horizontal `▀`.
- Título do modal na borda (ex.: `[ Sair ]`, `[ De ]` / `[ Para ]`).
- Menus permanentes; atalhos de teclado ao lado do rótulo em cinza (`menu_shortcut_style`).
- Itens booleanos do menu: **toggle único** com marcador `√` na margem esquerda (1 célula) — não submenus Ativar/Desativar.
- Opções mutuamente exclusivas (tema, colunas, margem, codificação, tabulação): estilo **radio** no menu.
- Submenus abrem **somente** com Right, Enter ou clique — não ao focar o item pai (`expanded_submenus`).
- Dropdown de menu renderizado **após** o editor (z-order opaco, linhas preenchidas até a largura).
- Barra de menu preenchida até a largura; separadores de submenu conectados às verticais (`╠══╣`).
- Comportamento visual lembra Turbo Vision / EDIT.EXE, sem copiar implementação literal.

---

## Regras de UX — atalhos globais

| Atalho | Ação |
|--------|------|
| `Ctrl+S` | Salvar |
| `Ctrl+O` | Abrir |
| `Ctrl+N` | Novo documento |
| `Ctrl+W` | Fechar documento |
| `Ctrl+Q` / `Alt+F4` | Sair (com confirmação se dirty; funciona mesmo com menu/modal aberto) |
| `Ctrl+T` | Alternar terminal inferior |
| `Ctrl+F` / `Ctrl+H` | Buscar / Substituir |
| `Ctrl+←/→` | Navegação inteligente por palavra |
| `Ctrl+Shift+←/→` | Seleção por palavra |
| `Alt` + arraste | Seleção retangular (bloco) |
| `Ctrl` + clique | Adicionar cursor (multi-cursor) |
| `F10` | Abrir menu Arquivo |

## Regras de UX — barra de menu

| Atalho | Menu |
|--------|------|
| `Alt+A` | Arquivo |
| `Alt+E` | Editar |
| `Alt+X` | Exibir |
| `Alt+F` | Formatar |

Mnemônico de Exibir é **X** (não E).

---

## Regras de UX — modais e confirmações

- Sair com documento dirty: modal com nome do arquivo; botões **[Salvar] [Não Salvar] [Cancelar]**.
- Trocar codificação: confirmação obrigatória; avisar se documento dirty.
- Converter tabulação: modal **De / Para** lado a lado; listas completas; foco sutil (borda preta/branca, sem fundo verde); Tab/Shift+Tab e ←/→ entre listas; botão **[Converter]**; opção **Para** vira tabulação ativa após confirmar.
- Botões de modal respondem a clique, hover e teclado; help do botão focado no rodapé.
- Modal aberto fecha menu dropdown automaticamente; modal e menu capturam input (`captures_input`).

---

## Regras de UX — editor e exibição

- Documento vazio: rope `""` (não `"\n"`); cursor em 0,0; sem placeholder.
- Modo Replace não apaga `\n`; linhas permanecem independentes.
- Enter cria nova linha.
- **Exibir → Borda:** visível (moldura completa) ou invisível (laterais/base ocultas; título mantido: `└ [ nome ] ─┘`).
- **Exibir → Margem:** sem / uma linha / duas linhas — padding interno no render.
- **Exibir → Colunas:** guia 80 / 120 / 160 / ilimitado.
- **Exibir → Mostrar:** símbolos, espaços, tabs (`»` onde há `\t`), fim de linha, tudo.
- **Exibir → Mostrar consumo de memória:** toggle (default ativo); amostragem leve (~2s).
- Tabulação literal e por espaços (2/4/8): expansão visual, cursor e scroll por coluna visual; parada 8 para Tab literal.
- Baseline dirty: documento novo ou aberto sem edição não dispara confirmação de saída (`EMPTY_DOCUMENT_TEXT`).

---

## Regras de domínio

| Item | Valor |
|------|-------|
| Recentes | 10 arquivos (`arquivo.recentes` em `edit.json`) |
| Clipboard interno | 5 itens |
| Temas | Escuro, Claro, Azul Clássico, Matrix |
| Codificações | UTF-8, UTF-8 sem BOM, UTF-16 LE/BE, ISO-8859-1, ANSI |
| Tabulação | 2 / 4 / 8 espaços ou Tab literal |
| Zoom | 1–3 |

Codificação e tabulação persistidas em `formatar` são **padrão** para novo documento; documento aberto mantém escolha da sessão até reset explícito.

---

## Regras de nomenclatura

- Arquivos e módulos: nomes descritivos e curtos.
- Funções: ação clara.
- Estruturas de domínio: refletem o papel real do componente.
- Evitar siglas obscuras quando houver nome mais claro.

---

## Regras de arquitetura

### Core de edição

- Buffer **`ropey`** em `src/editor/`; **proibido** reintroduzir `tui-textarea` ou widgets genéricos como donos do buffer.
- Toda mutação de texto passa por **`EditorEngine`** via **`EditorCommand`**.
- UI (`src/ui/`, `src/editor/render.rs`) **não** possui buffer de texto.

### Compositor de camadas (`src/ui/`)

- Trait **`UiLayer`**: pintura bottom→top, input top→bottom.
- Camadas: Desktop, Editor, Footer, MenuBar, MenuDropdown (overlay), Modal, Terminal (placeholder).
- Compositor unifica dispatch de paint e input; respeita `captures_input` por camada.

### Módulos principais

| Módulo | Responsabilidade |
|--------|------------------|
| `src/menus.rs` | Menu shell Turbo Vision; **proibido** menu estático com rótulos de comando |
| `src/modal/` | Shell `Dialog` reutilizável; presets em `buttons.rs` |
| `src/widgets/panel.rs` | Painéis, bordas ASCII, dropdown |
| `src/config.rs` | Load/save `edit.json` |
| `src/recent.rs` | Lista em memória (persistida via config) |
| `src/memory.rs` | Monitor RSS/working set (`sysinfo`) |
| `src/editor/tabs.rs` | Tab visual, conversão De/Para |
| `src/editor/word_boundary.rs` | Navegação por palavra (camelCase, separadores, dígitos) |

### Separação de concerns

- Preferir unidades pequenas e coesas.
- Separar estado, renderização e entrada de eventos.
- Não misturar lógica de UI com persistência sem necessidade — config serializa estado; App orquestra.
- Não substituir stack ou arquitetura por preferência pessoal.
- Não remover proteções de modal, confirmação ou prevenção de perda de dados.
- Toda decisão importante deve ser documentada em STATUS/TIMELINE ou spec.

---

## Fluxo de documentação

```
PROJECT_RULES.md  →  PROJECT_TIMELINE.md  →  specs/done/
        ↓
PROJECT_STATUS.md (estado atual)
```

Ao fechar uma feature: atualizar STATUS, TIMELINE e, se a regra for permanente, este arquivo.
