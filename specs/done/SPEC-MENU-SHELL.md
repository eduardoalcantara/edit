# Especificação Técnica — Menu Shell (Turbo Vision / EDIT.EXE)

**Autor:** Cursor (Composer) + PO  
**Data:** 2026-06-07  
**Versão:** 1.0  
**Status:** Aprovada para implementação direta pelo Cursor

---

## 1. Objetivo

Definir e implementar o **subsistema de menus real** do Editor Linux: uma barra de menu interativa, com dropdowns estilo **Borland Turbo Pascal IDE / Turbo Vision**, ergonomia familiar do **EDIT.EXE** do Windows, e integração correta com mouse e teclado em TTY local e SSH.

Esta spec entrega apenas o **shell** (infraestrutura visual e comportamental). Comandos concretos de Arquivo, Editar, Exibir, Formatar etc. serão plugados depois pelas specs incrementais já existentes em `specs/to-do/`.

---

## 2. Fonte de verdade (ordem obrigatória)

Antes de implementar **qualquer** código ou spec derivada, consultar nesta ordem:

1. `PROJECT_RULES.md`
2. `docs/EDITOR_LINUX_MASTER_REQUIREMENTS.md`
3. `docs/REFERENCE_UX_TURBO_VISION.md`
4. `docs/especificacao_funcional_v1_editor_linux.md`
5. Spec incremental aplicável em `specs/to-do/` ou `specs/done/`
6. `PROJECT_STATUS.md` e `PROJECT_TIMELINE.md`

**Regra permanente:** não implementar comportamento, atalho ou layout que contradiga documentos já registrados. Em caso de conflito, parar e registrar o conflito em `PROJECT_TIMELINE.md` antes de codar.

---

## 3. Problema a corrigir

A implementação atual usa **linhas de texto estático** (`MENU_ARQUIVO`, `MENU_EDITAR`, `MENU_EXIBIR` renderizados como `Paragraph`). Isso **não é menu** — é decoração.

| Incorreto (atual) | Correto (esta spec) |
|-------------------|---------------------|
| Três linhas expandidas com todos os comandos | **Uma** barra horizontal: ` Arquivo  Editar  Exibir ` |
| Sem hit-test de mouse | Clique abre dropdown; clique em item executa ação |
| Atalhos inventados (`Ctrl+Alt+D` etc.) | Apenas atalhos documentados em `PROJECT_RULES` / specs |
| Menu misturado com cheat sheet | Status bar **somente** para contexto do editor |
| `menu.rs` com strings | `menus.rs` com modelo de componentes e estado |

---

## 4. Referências de UX (EDIT.EXE + Turbo Vision)

### 4.1 EDIT.EXE (Windows)

- Barra de menu compacta no topo, abaixo do título.
- Ativação por **mouse** (clique no nome do menu) ou **teclado** (`Alt` + letra mnemônica).
- Dropdown vertical com itens, separadores e atalhos alinhados à direita do rótulo.
- Item sob cursor/teclado destacado (inversão de cores).
- `Esc` fecha o menu; setas navegam; `Enter` confirma.
- Submenu indicado por seta `►` à direita; abre cascata lateral.

### 4.2 Borland Turbo Pascal IDE / Turbo Vision

- Componentes com **bordas** em caracteres pseudográficos (`┌─┐│`, etc.).
- Menu pull-down claramente **delimitado** do conteúdo principal.
- Foco visual forte: item ativo invertido; menu aberto distinto da barra fechada.
- Sistema orientado a objetos: barra → menu → item → submenu → ação.
- Modais e dropdowns **não** competem com a área de edição sem capturar foco explicitamente.

### 4.3 Adaptação Editor Linux

- Preservar legibilidade em SSH/TTY sem truecolor obrigatório.
- Teclado permanece caminho principal; mouse degrada graciosamente se indisponível.
- Não copiar literalmente API do Turbo Vision; **reproduzir semântica** de interação.

---

## 5. Escopo desta spec

### 5.1 Deve incluir

- Módulo `src/menus.rs` (ou `src/menus/mod.rs` se justificado).
- `MenuBar` com itens de topo configuráveis.
- Dropdown pull-down com borda, scroll se necessário, separadores.
- Submenu em cascata (painel lateral ao item pai).
- Navegação completa por teclado e mouse.
- Roteamento de eventos: menu aberto **captura** input antes do editor.
- Itens de demonstração mínimos (placeholders) para validar o shell.
- Remoção do padrão falso de linhas estáticas em `menu.rs` / `ui.rs`.
- Layout: barra de título + **barra de menu (1 linha)** + editor + status bar.
- Registro de retângulos de hit-test para mouse.
- Estilos semânticos via `theme.rs` (normal, hover, foco, desabilitado).

### 5.2 Não deve incluir ainda

- Implementação completa de todos os comandos de `SPEC-MENU-ARQUIVO-EXIBIR`, `SPEC-MENU-EDITAR`, `SPEC-MENU-FORMATACAO-TABULACAO`.
- Clipboard histórico, busca/substituição, recentes reais, terminal funcional.
- Menu **Formatar** (virá em spec própria; shell deve permitir registrar novo topo depois).
- Abas, painel lateral funcional, seleção retangular.

---

## 6. Layout obrigatório na tela

```
┌─ Editor Linux ─ documento.txt* ─ Tema: Escuro ─────────────────────┐  ← barra de título (1 linha)
│ Arquivo   Editar   Exibir                                          │  ← barra de menu (1 linha)
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│                     área central de edição                         │
│                                                                    │
├────────────────────────────────────────────────────────────────────┤
│ UTF-8 │ Ln 1, Col 1 │ Sel: 0 │ 0 bytes │ Insert │ Local │ ...    │  ← status bar (1 linha)
└────────────────────────────────────────────────────────────────────┘

Quando menu aberto (exemplo):
│ Arquivo   Editar   Exibir                                          │
│ ┌──────────────┐                                                   │
│ │ Novo    Ctrl+N│                                                   │
│ │ Abrir   Ctrl+O│                                                   │
│ │ ───────────── │                                                   │
│ │ Sair    Ctrl+Q│                                                   │
│ └──────────────┘                                                   │
```

**Regras de layout:**

- A barra de menu ocupa **exatamente 1 linha** quando fechada.
- Dropdown sob o item de topo ativo, alinhado à esquerda do item.
- Submenu cascata à **direita** do item pai, com indicador `►`.
- Status bar **nunca** substitui a barra de menu nem lista todos os comandos.

---

## 7. Modelo de componentes (Rust)

### 7.1 Tipos principais

```text
MenuBar
  └── Vec<MenuTopItem>          // Arquivo, Editar, Exibir

MenuTopItem
  ├── id: MenuId
  ├── label: &'static str
  ├── mnemonic: Option<char>    // ex.: 'A' para Alt+A → Arquivo
  └── root: MenuNode

MenuNode
  ├── Item(MenuEntry)
  ├── Separator
  └── SubMenu { label, children: Vec<MenuNode> }

MenuEntry
  ├── id: ActionId
  ├── label: &'static str
  ├── shortcut: Option<ShortcutDisplay>  // ex.: "Ctrl+S" — só exibição
  ├── enabled: bool
  └── checked: Option<bool>     // para toggles futuros (Exibir)

MenuState
  ├── open_top: Option<usize>   // índice do menu de topo aberto
  ├── focus_path: Vec<usize>    // caminho no tree (top, item, subitem…)
  ├── hover_path: Option<Vec<usize>>
  └── cascade_rects: Vec<Rect>   // cache para hit-test mouse
```

### 7.2 Identificadores de ação

`ActionId` é enum extensível. Nesta spec, ações reais mínimas:

| ActionId | Comportamento nesta spec |
|----------|--------------------------|
| `NoOp` | Item placeholder — status bar: "Em breve" |
| `CloseMenu` | Fecha dropdown |
| `Quit` | Delega para `App::request_quit()` (já existente) |

Demais ações (`Open`, `Save`, `ThemeDark`, etc.) serão registradas no shell como **itens visíveis desabilitados** ou placeholders até specs incrementais conectarem handlers.

**Proibido:** inventar atalhos de teclado globais novos nesta spec além dos já documentados em `PROJECT_RULES.md`.

### 7.3 Módulos e responsabilidades

| Módulo | Responsabilidade |
|--------|------------------|
| `menus.rs` | Definição da árvore, estado, hit-test, dispatch de ação |
| `ui.rs` | Orquestra layout; chama `menus::render` |
| `events.rs` | Se `MenuState::is_open()`, encaminha evento a `menus::handle_event` |
| `app.rs` | Expõe métodos que menus invocam (`request_quit`, etc.) |
| `theme.rs` | Estilos `menu_bar`, `menu_item_normal`, `menu_item_focus`, `menu_border` |

**Remover ou esvaziar** `menu.rs` (strings estáticas) após migração.

---

## 8. Interação — teclado

### 8.1 Barra fechada

| Entrada | Comportamento |
|---------|---------------|
| `Alt` + mnemônico | Abre menu de topo correspondente; foco no primeiro item habilitado |
| `F10` | Abre menu **Arquivo** (primeiro da barra); foco no primeiro item |
| Atalhos globais (`Ctrl+S`, `Ctrl+O`, …) | Continuam funcionando **com barra fechada** (não remover) |

Mnemônicos iniciais (registrar em `PROJECT_RULES` se ainda não existirem):

| Menu | Mnemônico |
|------|-----------|
| Arquivo | `A` |
| Editar | `E` |
| Exibir | `X` |

### 8.2 Dropdown aberto

| Entrada | Comportamento |
|---------|---------------|
| `↑` / `↓` | Move foco entre itens (pula separadores e desabilitados) |
| `→` | Abre submenu cascata do item focado |
| `←` | Fecha submenu cascata ou fecha menu se já no nível raiz |
| `Enter` | Executa item focado ou abre submenu se tiver filhos |
| `Esc` | Fecha menu (um nível por vez ou tudo — preferir fechar tudo de uma vez) |
| Letra mnemônica do item | Atalho de aceleração dentro do menu aberto (estilo clássico) |

### 8.3 Prioridade de eventos

```text
1. Modal de risco (já existente) — maior prioridade
2. Menu aberto (esta spec)
3. Editor / atalhos globais
4. Mouse no editor
```

---

## 9. Interação — mouse

| Gesto | Comportamento |
|-------|---------------|
| Clique em item de topo | Abre dropdown; reposiciona se outro menu estava aberto |
| Clique novamente no mesmo topo | Fecha dropdown (toggle) |
| Clique fora do menu | Fecha dropdown |
| Clique em item do dropdown | Executa ação e fecha menu |
| Hover sobre item | Atualiza foco visual (se mouse habilitado) |
| Clique em item com submenu | Abre cascata à direita |

Se `mouse_enabled == false` (SSH/TTY restrito): menus funcionam **somente** por teclado, sem erro.

---

## 10. Renderização visual (Turbo Vision)

### 10.1 Barra de menu (fechada)

- Fundo: `theme.menu_bar_bg`.
- Itens de topo: espaçamento horizontal claro (` Arquivo  ` com padding).
- Item de topo **ativo/aberto**: estilo invertido ou sublinhado (`menu_top_active`).

### 10.2 Dropdown

- Borda: `BorderType::Plain` ou pseudográfico (`┌`, `─`, `│`, `└`).
- Largura: máximo entre itens + coluna de atalhos alinhada à direita.
- Item focado: inversão semântica (`menu_item_focus`).
- Item desabilitado: estilo atenuado, não selecionável.
- Separador: linha `─` ou `────────` dentro do painel.
- Submenu: sufixo ` ►` no rótulo.

### 10.3 Atalhos no menu

- Formato: `Rótulo` + padding + `Ctrl+S` alinhado à direita (estilo EDIT.EXE).
- Atalhos são **informativos** no rótulo; execução global continua via `events.rs`.

---

## 11. Árvore inicial de menus (shell + placeholders)

Estrutura mínima para validar cascata e navegação. Itens sem handler real ficam `enabled: false` ou `NoOp`.

### Arquivo

- Novo `Ctrl+N`
- Abrir `Ctrl+O`
- Recentes `Alt+R` *(atalho conforme MASTER_REQUIREMENTS)*
- ───
- Salvar `Ctrl+S`
- Salvar Como `Ctrl+Shift+S`
- ───
- Fechar `Ctrl+W`
- Sair `Ctrl+Q` → **handler real** (`request_quit`)

### Editar

- Desfazer `Ctrl+Z`
- Refazer `Ctrl+Y`
- ───
- Recortar `Ctrl+X`
- Copiar `Ctrl+C`
- Colar `Ctrl+V`
- Colar Anterior `Ctrl+Shift+V` ► *(submenu vazio placeholder: "Histórico vazio")*

### Exibir

- Temas ►
  - Escuro
  - Claro
  - Azul Clássico
  - Personalizado *(desabilitado)*
- ───
- Painel Lateral ► *(placeholder)*
- Terminal Inferior `Ctrl+T` ► *(placeholder)*

**Nota:** Zoom, Word Wrap, Colunas etc. entram quando `SPEC-MENU-ARQUIVO-EXIBIR` for implementada **sobre** este shell.

---

## 12. Integração com specs futuras

| Spec | Como encaixa |
|------|--------------|
| `SPEC-MENU-ARQUIVO-EXIBIR.md` | Registra handlers em `ActionId`; expande árvore Exibir |
| `SPEC-MENU-EDITAR.md` | Popula submenu Colar Anterior; habilita busca/substituição |
| `SPEC-MENU-FORMATACAO-TABULACAO.md` | Adiciona topo **Formatar** via API `MenuBar::register_top()` |

API mínima exigida para extensão:

```rust
// menus.rs — contrato para specs futuras
pub fn register_action_handler(/* ... */);
pub fn set_item_enabled(action: ActionId, enabled: bool);
pub fn rebuild_menu_exibir(/* ... */);  // ou builder pattern
```

---

## 13. Ordem de implementação

1. Criar `menus.rs` com tipos, árvore inicial e `MenuState`.
2. Implementar `menus::render(frame, area, &MenuBar, &MenuState, theme)`.
3. Implementar `menus::handle_event(event, &mut MenuState, &MenuBar) -> MenuEventResult`.
4. Integrar hit-test mouse em `menus::mouse_at(x, y)`.
5. Alterar `ui.rs`: remover linhas estáticas; 1 linha de menu + dropdown overlay.
6. Alterar `events.rs`: prioridade modal → menu → editor.
7. Remover `menu.rs` (strings) ou converter em re-export temporário deprecado.
8. `cargo build` + teste manual teclado/mouse.
9. Atualizar `PROJECT_STATUS.md`, `PROJECT_TIMELINE.md`, corrigir relatório INITIAL COMPLETE (menus parciais).

---

## 14. Critérios de aceite

- [ ] Barra de menu **uma linha**, itens `Arquivo`, `Editar`, `Exibir` clicáveis/acessíveis.
- [ ] Dropdown pull-down com **borda** e item focado **invertido**.
- [ ] Submenu cascata com `►` funcional (Teclado + mouse).
- [ ] `Alt+A/E/X`, `F10`, setas, Enter, Esc funcionam conforme seção 8.
- [ ] Mouse abre/fecha/seleciona quando disponível; teclado sempre funciona.
- [ ] Menu aberto **bloqueia** input no editor até fechar.
- [ ] **Nenhuma** linha de texto estático listando todos os comandos.
- [ ] Status bar permanece dedicada a contexto (não cheat sheet).
- [ ] `Sair` via menu dispara fluxo existente de confirmação dirty.
- [ ] Atalhos globais documentados (`Ctrl+S`, `Ctrl+O`, …) **não** foram alterados nem inventados novos.
- [ ] Documentação do projeto atualizada.

---

## 15. Testes manuais mínimos

1. Abrir app → barra compacta visível, editor recebe teclas.
2. `Alt+A` → dropdown Arquivo; `↓`/`↑`; `Enter` em Sair → modal se dirty.
3. Clique em Exibir → Temas ► → navegar cascata.
4. `Esc` fecha menu em qualquer nível.
5. SSH sem mouse: menus 100% operáveis por teclado.
6. Com mouse: clique fora fecha menu.

---

## 16. Registro de decisões

| Decisão | Motivo |
|---------|--------|
| Inspirar Turbo Vision + EDIT.EXE | Direção PO/Arquiteto; docs `REFERENCE_UX_TURBO_VISION`, MASTER_REQUIREMENTS |
| Shell antes de features | Specs `SPEC-MENU-*` assumem menus reais |
| `F10` → Arquivo | Convenção clássica DOS/Windows |
| Mnemônicos Alt+A/E/X | Padrão EDIT.EXE; registrados nesta spec |
| Remover menu falso atual | Correção de implementação inválida |

---

## 17. Observação final

Esta spec **substitui** qualquer interpretação anterior que considerasse linhas de texto como "menus implementados". Após a entrega deste shell, as specs `SPEC-MENU-ARQUIVO-EXIBIR`, `SPEC-MENU-EDITAR` e demais devem ser implementadas **plugando ações** — não reimplementando layout de menu.
