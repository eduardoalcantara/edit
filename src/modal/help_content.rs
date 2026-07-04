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
  • Busca e substituição (Ctrl+F / Ctrl+R); F3 / Shift+F3 próxima/anterior
  • Ir para linha e coluna (Ctrl+G)
  • Números de linha, word wrap, colunas-guia (80/120/160/ilimitado)
  • Margem interna (0/1/2 linhas), borda visível ou mínima
  • Mostrar símbolos, espaços, tabs (»), fim de linha
  • Modo Replace não apaga quebras de linha; documento vazio = rope ""


═══ WORKSPACE / ABAS ═══

  • Até 10 abas; Ctrl+Tab / Ctrl+Shift+Tab / F4 / Shift+F4
  • Split horizontal: Exibir → Dividir editor; Ctrl+1 (único/esquerda) / Ctrl+2 (split/direita)
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
  • Menus Alt+A/E/X/F/S/H; toggles com √; opções exclusivas estilo radio
  • Split: rodapé mostra Editor 1 / Editor 2 conforme painel em foco


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
  Esc                 Limpa busca ativa; sem busca, sai do editor
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
  Ctrl+F / Ctrl+R     Buscar / Substituir
  Ctrl+G              Ir para linha...
  F3 / Shift+F3       Próxima / anterior ocorrência de busca
  Esc                 Limpa busca ativa no editor (sem modal aberto)


─── BUSCAR / SUBSTITUIR (modal) ───

  Esc (modal busca)   Fecha o diálogo (equivale a [Fechar])
  Limpar              Limpa campos e remove destaque no texto


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
  Ctrl+1              Editor único (ou foco esquerdo em split)
  Ctrl+2              Dividir editor / foco painel direito
  Alt+S               Menu Abas


─── MENUS (BARRA SUPERIOR) ───

  Alt+A               Arquivo
  Alt+E               Editar
  Alt+X               Exibir  (mnemônico X, não E)
  Alt+F               Formatar
  Alt+S               Abas
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


─── CAMPOS DE TEXTO (modais) ───

  ←/→ Home/End        Cursor; Shift+setas seleciona
  Backspace/Delete    Apagar caractere ou seleção
  Ctrl+C/X/V/A        Copiar / Recortar / Colar / Selecionar tudo


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

pub fn ascii_table_text() -> &'static str {
    r#"TABELA ASCII — edit

Referência rápida de caracteres de desenho (CP437 / Unicode equivalente).
Use Ctrl+C após selecionar trechos neste painel.


═══ BORDAS SIMPLES (CP437) ═══

  ┌  U+250C   ┐  U+2510   └  U+2514   ┘  U+2518
  ─  U+2500   │  U+2502   ├  U+251C   ┤  U+2524
  ┬  U+252C   ┴  U+2534   ┼  U+253C


═══ BORDAS DUPLAS ═══

  ╔  U+2554   ╗  U+2557   ╚  U+255A   ╝  U+255D
  ═  U+2550   ║  U+2551   ╠  U+2560   ╣  U+2563


═══ BLOCOS / SOMBRA (UI do editor) ═══

  █  U+2588 bloco cheio     ▀  U+2580 meio superior
  ░  U+2591 claro           ▒  U+2592 médio
  ▓  U+2593 escuro


═══ SÍMBOLOS ÚTEIS ═══

  √  U+221A check (menus)    »  U+00BB seta submenu
  •  U+2022 bullet           ·  U+00B7 ponto médio
  °  U+00B0 grau             ±  U+00B1 mais/menos
  ×  U+00D7 multiplicação    ÷  U+00F7 divisão


═══ DEC 128–175 (CP437 clássico, amostra) ═══

  128 Ç   129 ü   130 é   131 â   132 ä   133 à
  134 å   135 ç   136 ê   137 ë   138 è   139 ï
  140 î   141 ì   142 Ä   143 Å   144 É   145 æ
  146 Æ   147 ô   148 ö   149 ò   150 û   151 ù
  152 ÿ   153 Ö   154 Ü   155 ¢   156 £   157 ¥
  158 ₧   159 ƒ   160 á   161 í   162 ó   163 ú
  164 ñ   165 Ñ   166 ª   167 º   168 ¿   169 ⌐
  170 ¬   171 ½   172 ¼   173 «   174 »   175 ░


═══ DEC 176–223 (blocos CP437) ═══

  176 ░   177 ▒   178 ▓   179 │   180 ┤   181 ╡
  182 ╢   183 ╖   184 ╕   185 ╣   186 ║   187 ╗
  188 ╝   189 ╜   190 ╛   191 ┐   192 └   193 ┴
  194 ┬   195 ├   196 ─   197 ┼   198 ╞   199 ╟
  200 ╚   201 ╔   202 ╩   203 ╦   204 ╠   205 ═
  206 ╬   207 ╧   208 ╨   209 ╤   210 ╥   211 ╙
  212 ╘   213 ╒   214 ╓   215 ╫   216 ╪   217 ┘
  218 ┌   219 █   220 ▄   221 ▌   222 ▐   223 ▀


═══ DEC 224–255 (símbolos CP437) ═══

  224 α   225 ß   226 Γ   227 π   228 Σ   229 σ
  230 µ   231 τ   232 Φ   233 Θ   234 Ω   235 δ
  236 ∞   237 φ   238 ε   239 ∩   240 ≡   241 ±
  242 ≥   243 ≤   244 ⌠   245 ⌡   246 ÷   247 ≈
  248 °   249 ·   250 ·   251 √   252 ⁿ   253 ²
  254 ■   255 NBSP


Dica: em terminais UTF-8, prefira os pontos de código Unicode acima.
Em SSH monocromático, ative mnemônicos com parênteses em edit.json
(exibir.mnemonico_parenteses = "ligado").
"#
}
