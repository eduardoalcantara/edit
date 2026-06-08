pub fn features_text() -> &'static str {
    r#"FEATURES — edit

Editor
  • Buffer ropey com undo/redo
  • Insert/Replace, seleção linear e por bloco
  • Busca e substituição (Ctrl+F / Ctrl+H)
  • Números de linha, word wrap, smart word navigation
  • Ir para linha (Ctrl+G)

Workspace
  • Até 10 abas; Ctrl+Tab / Alt+1…0
  • Menu Abas (Alt+S); Salvar Todos (Ctrl+Alt+S)
  • Persistência em edit.json e .edit-session/

Terminal
  • PTY integrado, emulador VT100
  • Multi-sessão; F6 foco; F7 envia texto do editor

Exibir e temas
  • Turbo Vision: Azul Clássico, Escuro, Claro, Matrix
  • Terminal, rodapé, memória, margem, borda

Arquivos
  • Navegador estilo Turbo Pascal (Abrir / Salvar Como)
  • Recentes, codificação e tabulação configuráveis
"#
}

pub fn shortcuts_text() -> &'static str {
    r#"ATALHOS — edit

Global
  Ctrl+Q / Alt+F4     Sair
  Esc                 Sair (editor, sem menu/modal)
  Ctrl+E              Foco editor
  Ctrl+T / Ctrl+'     Terminal: abre/foca/fecha
  Ctrl+G              Ir para linha

Arquivo
  Ctrl+N/O/S          Novo / Abrir / Salvar
  Ctrl+Shift+S        Salvar Como
  Ctrl+W              Fechar aba
  F10                 Salvar
  F2                  Renomear

Editar
  Ctrl+Z/Y            Desfazer / Refazer
  Ctrl+F/H            Buscar / Substituir
  Ctrl+G              Ir para linha

Navegação
  F3 / Shift+F3       Próxima / anterior busca
  F4 / Shift+F4       Próxima / anterior aba
  F6                  Foco Editor ↔ Terminal
  F7                  Enviar seleção ao terminal
  F1                  Ajuda (Features)

Abas
  Alt+1 … Alt+0       Foco aba 1–10
  Alt+S               Menu Abas
  Ctrl+Shift+W        Fechar Todos

Menus
  Alt+A/E/X/F/H       Arquivo / Editar / Exibir / Formatar / Ajuda
"#
}

pub fn about_text() -> String {
    format!(
        "SOBRE — edit\n\n\
         Editor TUI estilo Turbo Vision\n\
         Versão: {}\n\
         Pacote: edit\n\n\
         Configuração local: edit.json\n\
         Sessão: .edit-session/\n\
         Sem telemetria.\n",
        env!("CARGO_PKG_VERSION")
    )
}
