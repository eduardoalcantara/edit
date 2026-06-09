pub fn features_text() -> &'static str {
    r#"FUNCIONALIDADES — edit

Editor de terminal estilo Turbo Vision / Turbo Pascal, com buffer ropey,
menus pull-down, modais e proteção contra perda de trabalho.


═══ EDIÇÃO DE TEXTO ═══

  • Buffer ropey (src/editor/) — insert/replace, Enter, undo/redo por aba
  • Seleção linear: Shift+setas, mouse, Ctrl+A
  • Seleção em bloco (Alt+arraste) e multi-cursor (Ctrl+clique) — parcial
  • Smart Word Navigation: Ctrl+←/→ e Ctrl+Shift+←/→ (camelCase, _, -, dígitos)
  • Tabulação literal ou por espaços (2/4/8); expansão visual de \t
  • Busca e substituição (Ctrl+F / Ctrl+H); F3 / Shift+F3 próxima/anterior
  • Ir para linha e coluna (Ctrl+G)
  • Números de linha, word wrap, colunas-guia (80/120/160/ilimitado)
  • Margem interna (0/1/2 linhas), borda visível ou mínima
  • Mostrar símbolos, espaços, tabs (»), fim de linha
  • Modo Replace não apaga quebras de linha; documento vazio = rope ""


═══ WORKSPACE / ABAS ═══

  • Até 10 abas; Ctrl+Tab / Ctrl+Shift+Tab / F4 / Shift+F4
  • Alt+1 … Alt+0 foco direto; menu Abas (Alt+S)
  • Novo (Ctrl+N): reutiliza aba pristine ou cria NovoN
  • Fechar aba (Ctrl+W); Fechar Todos (Ctrl+Shift+W) com confirmação
  • Salvar Todos (Ctrl+Alt+S); evicção da aba do final se exceder 10
  • Recentes (Arquivo): últimos 10 fechados; menu Abas = abertos
  • Sessão em .edit-session/; config edit.json v2 (arquivo.abas)
  • Toggles: Fechar tudo ao sair; Salvar desfazer recentes no disco


═══ ARQUIVOS ═══

  • Navegador estilo Turbo Pascal para Abrir / Salvar Como
    — lista de pastas e arquivos, filtro *.*, arquivos ocultos, barra status
  • Salvar (Ctrl+S / F10); Renomear no FS (F2) — modal de caminho simples
  • Codificação: UTF-8, UTF-8 sem BOM, UTF-16 LE/BE, ISO-8859-1, ANSI
  • Converter tabulação (De / Para) no modal dedicado
  • Confirmação ao sair/trocar aba com documento dirty


═══ TERMINAL INTEGRADO ═══

  • PTY real (portable-pty), emulador VT100, multi-sessão
  • Ctrl+T / Ctrl+' abre/foca/fecha; F6 alterna Editor ↔ Terminal
  • F7 envia seleção (ou linha atual) ao terminal
  • Sidebar: nova sessão, +/- altura painel, fechar painel/sessão
  • PgUp/PgDn rola scrollback; Ctrl+C copia seleção do scrollback
  • Cwd da nova sessão = pasta do arquivo da aba ativa


═══ INTERFACE E TEMAS ═══

  • Compositor de camadas: menu opaco, modais com sombra, rodapé contextual
  • Temas: Azul Clássico, Escuro, Claro, Matrix (Exibir → Temas)
  • Rodapé: help à esquerda; Tam/Pos/modo/encoding/tab/memória à direita
  • Zoom 1–3; consumo de memória opcional (~2s)
  • Menus Alt+A/E/X/F/H; toggles com √; opções exclusivas estilo radio


═══ PERSISTÊNCIA ═══

  • edit.json ao lado do executável (arquivo, exibir, formatar)
  • Migração automática de recent.json legado
  • Clipboard interno: 5 itens
  • Sem telemetria
"#
}

pub fn shortcuts_text() -> &'static str {
    r#"ATALHOS — edit

Referência completa. Atalhos globais funcionam mesmo com menu/modal aberto
quando indicado (ex.: Ctrl+Q, Alt+F4).


─── GLOBAL ───

  Ctrl+Q / Alt+F4     Sair (confirma se dirty)
  Esc                 Sair (só com foco no editor, sem menu/modal)
  Ctrl+E              Foco no editor
  Ctrl+T / Ctrl+'     Editor: abre/foca terminal; Terminal: fecha painel
  F6                  Alterna foco Editor ↔ Terminal
  F7                  Envia seleção/linha ao terminal
  F1                  Ajuda → Funcionalidades
  F10                 Salvar aba ativa


─── ARQUIVO ───

  Ctrl+N              Novo documento
  Ctrl+O              Abrir (navegador de arquivos)
  Ctrl+S              Salvar
  Ctrl+Shift+S        Salvar Como
  Ctrl+Alt+S          Salvar Todos
  Ctrl+W              Fechar aba
  Ctrl+Shift+W        Fechar Todos
  F2                  Renomear arquivo no disco


─── EDITAR ───

  Ctrl+Z / Ctrl+Y     Desfazer / Refazer
  Ctrl+X / Ctrl+C / Ctrl+V   Recortar / Copiar / Colar
  Ctrl+A              Selecionar tudo
  Ctrl+F / Ctrl+H     Buscar / Substituir
  Ctrl+G              Ir para linha...
  F3 / Shift+F3       Próxima / anterior ocorrência de busca


─── NAVEGAÇÃO NO TEXTO ───

  Ctrl+← / Ctrl+→     Palavra anterior / próxima
  Ctrl+Shift+←/→      Selecionar por palavra
  Home / End          Início / fim da linha
  Ctrl+Home / End     Início / fim do documento
  PgUp / PgDn         Página acima / abaixo (editor)
  Alt+arraste         Seleção retangular (bloco)
  Ctrl+clique         Adicionar cursor (multi-cursor)


─── ABAS ───

  Ctrl+Tab            Próxima aba (se o host repassar)
  Ctrl+Shift+Tab      Aba anterior
  F4 / Shift+F4       Próxima / anterior aba (Windows-safe)
  Alt+1 … Alt+0       Foco aba na posição 1–10
  Alt+S               Menu Abas


─── MENUS (BARRA SUPERIOR) ───

  Alt+A               Arquivo
  Alt+S               Abas
  Alt+E               Editar
  Alt+X               Exibir  (mnemônico X, não E)
  Alt+F               Formatar
  Alt+H               Ajuda (Funcionalidades, Atalhos, Sobre)


─── TERMINAL (foco no PTY) ───

  Esc                 Devolve foco ao editor
  PgUp / PgDn         Rola scrollback
  Ctrl+C              Copia seleção do scrollback (ou envia ao PTY)
  Mouse               Arraste seleciona; roda rola scrollback

  Sidebar [n] nova sessão  [+]/- altura  [f] fecha painel  [q] fecha sessão


─── NAVEGADOR DE ARQUIVOS (modal Abrir/Salvar) ───

  Tab / Shift+Tab     Nome → Arquivos → Filtro → Ocultos → botões
  ↑/↓                 Navega lista; Enter abre pasta ou confirma arquivo
  F5                  Atualiza lista
  Esc / Cancelar      Fecha sem alterar
  Alt+O / Alt+C       Abrir/Salvar ou Cancelar
  Duplo-clique        Pasta: entra; arquivo (Abrir): confirma


─── MODAIS EM GERAL ───

  ←/→ ou Tab        Entre botões
  Enter               Ativa botão focado
  Esc                 Cancelar / Fechar
  Mouse               Clique e hover nos botões
"#
}

pub fn about_text() -> String {
    format!(
        "SOBRE — edit\n\n\
         Editor TUI estilo Turbo Vision para Linux e Windows.\n\
         Buffer ropey, menus pull-down, terminal PTY integrado,\n\
         workspace com até 10 abas e temas configuráveis.\n\n\
         Versão: {}\n\
         Pacote: edit\n\
         Autor: Perplexity AI (README do projeto)\n\n\
         Configuração: edit.json (mesma pasta do executável)\n\
         Sessão de abas: .edit-session/\n\
         Documentação: PROJECT_RULES.md, README.md\n\n\
         Licença e código-fonte conforme repositório do projeto.\n\
         Sem telemetria ou envio de dados.\n",
        env!("CARGO_PKG_VERSION")
    )
}
