# Spec — Fidelidade UX Turbo Vision / EDIT.EXE

**Autor:** PO (feedback visual) + Cursor  
**Data:** 2026-06-07  
**Versão:** 1.0  
**Referência:** `docs/REFERENCE_UX_TURBO_VISION.md`, capturas Turbo Pascal 7.0, screenshots do Editor Linux atual

## Objetivo

Fechar a lacuna entre a UI implementada e a referência Turbo Vision / EDIT.EXE. A implementação atual entrega **estrutura de menu**, mas não a **linguagem visual** nem os **controles** esperados.

---

## Prioridade crítica (bloqueia aceite visual)

### TV1 — Paleta e inversão de cores incorretas

| Atual | Esperado (Turbo Vision) |
|-------|-------------------------|
| Tema escuro cinza/preto dominante | Barra de menu **cinza claro**, texto **preto** |
| Destaque cyan no menu ativo | Item ativo do menu topo: **vermelho** ou **verde** sólido |
| Item focado: inversão genérica | Item focado no dropdown: **verde** ou **magenta** sobre cinza |
| Editor preto | Área de edição **azul** (`#0000AA`) com texto branco/amarelo |
| Hotkeys invisíveis | Primeira letra do menu em **vermelho** (`Alt+F` → **F**ile) |

**Módulos:** `src/theme.rs`, `src/menus.rs`  
**Critério de aceite:** screenshot lado a lado reconhecível como “estilo Borland”.

### TV2 — Menu pull-down transparente / sem profundidade

| Atual | Esperado |
|-------|----------|
| Texto do editor visível através do menu | Painel **100% opaco** (fundo cinza sólido) |
| Borda fina mal fechada | Borda **double-line** (`╔═╗`) fechada |
| Sem sombra | **Drop shadow** preta 1 célula à direita e abaixo |
| Separadores desalinhados | Linha `─` contínua na largura interna |

**Módulos:** `src/menus.rs`  
**Critério de aceite:** menu cobre completamente o conteúdo subjacente; sombra visível.

### TV3 — Rodapé ausente ou errado

| Atual | Esperado |
|-------|----------|
| Rodapé opcional (`footer_visible`); conteúdo técnico (UTF-8, bytes) | **Sempre visível**; faixa cinza claro |
| Sem atalhos F1–F10 | `F1 Help` `F2 Save` `F3 Open` … com **F-keys em vermelho** |
| Mensagem de status no rodapé | Linha contextual à direita (ex.: descrição do item de menu focado) |

**Referência:** `PROJECT_RULES.md` — barra de status = contexto, não cheat sheet permanente; Turbo Pascal mistura F-keys fixos + hint contextual.

**Módulos:** `src/ui.rs`, `src/view_state.rs`  
**Critério de aceite:** 1 linha de rodapé sempre presente; F1/F10 funcionais ou documentados.

### TV4 — Barra de título separada (não-TV)

| Atual | Esperado |
|-------|----------|
| Linha extra `Editor Linux │ Sem título* │ Tema` | **Sem** barra de app; título do documento **no frame da janela** do editor |
| Modo Insert/Normal no título do Block | Indicadores `Ln Col Insert` **dentro** da janela ou no rodapé |

**Módulos:** `src/ui.rs`, `src/editor.rs`  
**Critério de aceite:** layout 3 faixas: menu | janela editor | rodapé.

### TV5 — Modo Replace preso na coluna

| Atual | Esperado |
|-------|----------|
| Substitui 1 caractere; cursor **não avança** | Overtype: substituir e **avançar** para próxima coluna (comportamento EDIT.EXE) |

**Módulos:** `src/editor.rs`  
**Critério de aceite:** digitar sequência em Replace desloca cursor monotonicamente à direita.

### TV6 — Diálogos sem botões reais

| Atual | Esperado |
|-------|----------|
| Texto `[ Enter ] Sim    [ Esc ] Cancelar` | Botões renderizados `[ OK ]` `[ Cancel ]` em **bloco verde**, clicáveis com mouse |
| Modal path: campo texto cru | Caixa de input com borda; botões abaixo |
| Sem sombra no modal | Modal com borda double-line + sombra |

**Módulos:** `src/ui.rs`, `src/modal.rs`, `src/events.rs`  
**Critério de aceite:** Tab/←→ alterna botões; Enter aciona; clique mouse no botão.

### TV7 — Abrir arquivo: path manual, sem browser FS

| Atual | Esperado (EDIT.EXE) |
|-------|---------------------|
| Modal pede caminho digitado | Diálogo **Pick file** com árvore/lista de diretórios |
| Sem navegação `..`, drives | Painel dirs + painel arquivos + máscara `*.*` |
| Sem preview de seleção | Highlight de arquivo focado; Enter confirma |

**Módulos:** novo `src/file_picker.rs`  
**Critério de aceite:** abrir `.rs` navegando pastas sem digitar path completo.

---

## Prioridade alta (janela e chrome)

### TV8 — Janela do editor sem chrome Turbo Vision

- Falta botão fechar `[■]` canto superior esquerdo
- Falta indicador de zoom `[↕]` canto superior direito
- Falta scrollbars integradas ao frame
- Falta indicador `Ln:Col` no canto inferior esquerdo **do frame**

### TV9 — Diálogo Find/Replace estilo Borland

- Campos separados com labels amarelas
- Checkboxes: case sensitive, whole words, regex
- Radio: Forward/Backward, Global/Selected
- Botões `[ OK ]` `[ Cancel ]` `[ Help ]`

**Relacionado:** L5, L6 em `SPEC-LIMITACOES-PENDENTES.md`

### TV10 — Menu cascata: alinhamento e hit-test

- Submenu deslocado verticalmente pelo índice do item pai (correto em TV) — validar
- Bordas que “escapam” do retângulo (bug reportado nas capturas)

---

## Prioridade média

### TV11 — Tema “Azul Clássico” não replica VGA 16 cores

Ajustar para paleta fixa DOS: gray(7), blue(1), red(4), green(2), yellow(14), white(15).

### TV12 — Área de trabalho (desktop) azul atrás da janela

Workspace preenchido com azul; janela editor “flutua” com margem (opcional fase 2).

### TV13 — Status contextual ao navegar menu

Rodapé muda para descrever item focado (ex.: “Save the current file”).

---

## Mapa de dependências

```text
TV1 + TV2 + TV4  →  aparência base aceitável
TV3 + TV5        →  usabilidade mínima EDIT.EXE
TV6 + TV7        →  paridade de diálogos
TV8–TV10         →  polimento profissional
```

## Ordem de implementação sugerida

1. TV5 (Replace) — bug funcional  
2. TV1, TV2, TV4, TV3 — shell visual  
3. TV6 — botões modais  
4. TV7 — file picker  
5. TV8–TV13 — chrome e Find dialog  

## Relação com outros docs

| Doc | Relação |
|-----|---------|
| `SPEC-LIMITACOES-PENDENTES.md` | Limitações técnicas/funcionais |
| Este spec | **Gap visual e UX** vs referência |
| `docs/REFERENCE_UX_TURBO_VISION.md` | Princípios (este spec operacionaliza) |

## Critério global de conclusão

PO consegue colocar screenshot do Editor Linux ao lado do Turbo Pascal e apontar **paridade de estrutura** (menu, janela, rodapé, modal, botões), não apenas “tem menu dropdown”.
