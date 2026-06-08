use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::clipboard::Clipboard;
use crate::encoding::{FileEncoding, Tabulation};
use crate::recent::{display_name, RecentFiles};
use crate::theme::ThemePalette;
use crate::view_state::{EditorBorder, EditorMargin, GuideColumn, ViewState};
use crate::workspace::Workspace;
use crate::widgets::panel::{self, cp437, PanelBorder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    NoOp,
    Quit,
    New,
    Open,
    Recent,
    Save,
    SaveAs,
    Rename,
    SaveAll,
    Close,
    CloseAll,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    PastePrevious,
    SelectAll,
    Find,
    Replace,
    ThemeDark,
    ThemeLight,
    ThemeClassicBlue,
    ThemeMatrix,
    ToggleTerminal,
    ToggleFooter,
    ShowMemoryToggle,
    LineNumbersToggle,
    WordWrapToggle,
    ShowSymbols,
    ShowSpaces,
    ShowTabs,
    ShowEol,
    ShowAll,
    Column80,
    Column120,
    Column160,
    ColumnUnlimited,
    MarginNone,
    MarginOneLine,
    MarginTwoLines,
    BorderToggle,
    EncodingUtf8,
    EncodingUtf8NoBom,
    EncodingUtf16Le,
    EncodingUtf16Be,
    EncodingIso88591,
    EncodingAnsi,
    TabSpaces2,
    TabSpaces4,
    TabSpaces8,
    TabLiteral,
    ConvertTabulation,
    PasteClip(usize),
    OpenRecent(usize),
    FocusTab(usize),
    ToggleCloseAllOnExit,
    TogglePersistUndo,
    SortFileName,
    SortFilePath,
    SortOpenedFirst,
    SortOpenedLast,
    SortStatus,
}

#[derive(Clone)]
pub enum MenuNode {
    Separator,
    Item {
        label: String,
        shortcut: Option<&'static str>,
        action: ActionId,
        enabled: bool,
        checked: Option<bool>,
        help: Option<&'static str>,
    },
    SubMenu {
        label: &'static str,
        help: Option<&'static str>,
        children: Vec<MenuNode>,
    },
}

#[derive(Clone)]
pub struct MenuTopItem {
    pub label: &'static str,
    pub mnemonic: char,
    pub children: Vec<MenuNode>,
}

pub struct MenuBar {
    pub tops: Vec<MenuTopItem>,
}

#[derive(Debug, Clone, Default)]
pub struct MenuState {
    pub open_top: Option<usize>,
    pub focus_path: Vec<usize>,
    /// Caminho de submenus abertos explicitamente (Right/Enter/clique).
    pub expanded_submenus: Vec<usize>,
    pub bar_area: Rect,
    pub top_hit_areas: Vec<Rect>,
    pub panel_areas: Vec<Rect>,
    pub item_hit_areas: Vec<(Vec<usize>, Rect)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEventResult {
    None,
    Consumed,
    Closed,
    Action(ActionId),
}

impl MenuState {
    pub fn is_open(&self) -> bool {
        self.open_top.is_some()
    }

    pub fn close(&mut self) {
        self.open_top = None;
        self.focus_path.clear();
        self.expanded_submenus.clear();
        self.panel_areas.clear();
        self.item_hit_areas.clear();
    }

    pub fn open_top_menu(&mut self, index: usize, bar: &MenuBar) {
        if index >= bar.tops.len() {
            return;
        }
        self.open_top = Some(index);
        self.expanded_submenus.clear();
        self.focus_path = vec![first_enabled_index(&bar.tops[index].children)];
        self.focus_path.retain(|&i| i != usize::MAX);
        if self.focus_path.is_empty() {
            self.focus_path.push(0);
        }
    }

    pub fn toggle_top(&mut self, index: usize, bar: &MenuBar) {
        if self.open_top == Some(index) {
            self.close();
        } else {
            self.open_top_menu(index, bar);
        }
    }
}

impl MenuBar {
    pub fn build(
        recent: &RecentFiles,
        view: &ViewState,
        enc: FileEncoding,
        tab: Tabulation,
        clip: &Clipboard,
        workspace: &Workspace,
    ) -> Self {
        Self {
            tops: vec![
                MenuTopItem {
                    label: " Arquivo ",
                    mnemonic: 'A',
                    children: file_menu(recent),
                },
                MenuTopItem {
                    label: " Abas ",
                    mnemonic: 'S',
                    children: tabs_menu(workspace),
                },
                MenuTopItem {
                    label: " Editar ",
                    mnemonic: 'E',
                    children: edit_menu(clip),
                },
                MenuTopItem {
                    label: " Exibir ",
                    mnemonic: 'X',
                    children: view_menu(view),
                },
                MenuTopItem {
                    label: " Formatar ",
                    mnemonic: 'F',
                    children: format_menu(enc, tab),
                },
            ],
        }
    }

    pub fn top_index_by_mnemonic(&self, c: char) -> Option<usize> {
        self.tops
            .iter()
            .position(|t| t.mnemonic.eq_ignore_ascii_case(&c))
    }
}

/// Texto de ajuda do item/submenu atualmente em foco.
pub fn focused_help(bar: &MenuBar, state: &MenuState) -> Option<&'static str> {
    let top = state.open_top?;
    let node = node_at_path(&bar.tops[top].children, &state.focus_path)?;
    match node {
        MenuNode::Item { help, .. } | MenuNode::SubMenu { help, .. } => *help,
        MenuNode::Separator => None,
    }
}

fn tab_menu_shortcut(index: usize) -> Option<&'static str> {
    const SHORTCUTS: [&str; 10] = [
        "Alt+1", "Alt+2", "Alt+3", "Alt+4", "Alt+5", "Alt+6", "Alt+7", "Alt+8", "Alt+9", "Alt+0",
    ];
    SHORTCUTS.get(index).copied()
}

fn tabs_menu(workspace: &Workspace) -> Vec<MenuNode> {
    let mut nodes: Vec<MenuNode> = workspace
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            item(
                format!(" {}", tab.menu_label()),
                tab_menu_shortcut(i),
                ActionId::FocusTab(i),
                true,
                Some(i == workspace.active_index),
                "Foca esta aba aberta",
            )
        })
        .collect();

    if nodes.is_empty() {
        nodes.push(item(
            "(vazio)",
            None,
            ActionId::NoOp,
            false,
            None,
            "Nenhuma aba aberta",
        ));
    }

    nodes.push(MenuNode::Separator);
    nodes.push(item(
        "Fechar Todos",
        Some("Ctrl+Shift+W"),
        ActionId::CloseAll,
        true,
        None,
        "Fecha todas as abas abertas",
    ));
    nodes.push(toggle_item(
        "Fechar tudo ao sair",
        ActionId::ToggleCloseAllOnExit,
        workspace.fechar_tudo_ao_sair,
        "Descarta abas ao encerrar; desligado mantém a sessão entre execuções",
    ));
    nodes.push(toggle_item(
        "Salvar desfazer recentes",
        ActionId::TogglePersistUndo,
        workspace.salvar_desfazer_recentes,
        "Mantém até 5+ passos de desfazer por aba entre sessões; desligue para economizar disco",
    ));
    nodes.push(submenu(
        "Ordenar por",
        "Reordena abas abertas na sessão atual",
        vec![
            item(
                "Nome de Arquivo",
                None,
                ActionId::SortFileName,
                true,
                None,
                "Ordem alfabética pelo rótulo exibido",
            ),
            item(
                "Caminho",
                None,
                ActionId::SortFilePath,
                true,
                None,
                "Ordem alfabética pelo caminho completo",
            ),
            item(
                "Abertos Primeiro",
                None,
                ActionId::SortOpenedFirst,
                true,
                None,
                "Abas abertas há mais tempo no topo",
            ),
            item(
                "Abertos por Último",
                None,
                ActionId::SortOpenedLast,
                true,
                None,
                "Abas abertas recentemente no topo",
            ),
            item(
                "Status",
                None,
                ActionId::SortStatus,
                true,
                None,
                "Abas com alterações pendentes no topo",
            ),
        ],
    ));
    nodes
}

fn file_menu(recent: &RecentFiles) -> Vec<MenuNode> {
    let mut nodes = vec![
        item(
            "Novo",
            Some("Ctrl+N"),
            ActionId::New,
            true,
            None,
            "Cria um novo documento em branco",
        ),
        item(
            "Abrir",
            Some("Ctrl+O"),
            ActionId::Open,
            true,
            None,
            "Abre um arquivo existente do disco",
        ),
    ];
    let mut recent_children: Vec<MenuNode> = recent
        .paths()
        .iter()
        .enumerate()
        .map(|(i, p)| {
            item(
                format!(" {}", display_name(p)),
                None,
                ActionId::OpenRecent(i),
                true,
                None,
                "Abre um arquivo da lista de recentes",
            )
        })
        .collect();
    if recent_children.is_empty() {
        recent_children.push(item(
            "(vazio)",
            None,
            ActionId::NoOp,
            false,
            None,
            "Nenhum arquivo recente disponível",
        ));
    }
    nodes.push(submenu(
        "Recentes",
        "Lista de arquivos abertos recentemente",
        recent_children,
    ));
    nodes.extend([
        MenuNode::Separator,
        item(
            "Salvar",
            Some("Ctrl+S / F10"),
            ActionId::Save,
            true,
            None,
            "Salva o documento atual no arquivo em disco",
        ),
        item(
            "Salvar Como",
            Some("Ctrl+Shift+S"),
            ActionId::SaveAs,
            true,
            None,
            "Salva o documento com um novo nome ou caminho",
        ),
        item(
            "Renomear",
            Some("F2"),
            ActionId::Rename,
            true,
            None,
            "Renomeia o arquivo atual no disco",
        ),
        item(
            "Salvar Todos",
            Some("Ctrl+Alt+S"),
            ActionId::SaveAll,
            true,
            None,
            "Salva todas as abas com alterações pendentes",
        ),
        MenuNode::Separator,
        item(
            "Fechar",
            Some("Ctrl+W"),
            ActionId::Close,
            true,
            None,
            "Fecha o documento atual",
        ),
        item(
            "Sair",
            Some("Ctrl+Q / Alt+F4"),
            ActionId::Quit,
            true,
            None,
            "Encerra o editor",
        ),
    ]);
    nodes
}

fn edit_menu(clip: &Clipboard) -> Vec<MenuNode> {
    let mut paste_prev: Vec<MenuNode> = clip
        .entries()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            item(
                Clipboard::preview(i, t),
                None,
                ActionId::PasteClip(i),
                true,
                None,
                "Cola um item do histórico interno da área de transferência",
            )
        })
        .collect();
    if paste_prev.is_empty() {
        paste_prev.push(item(
            "(vazio)",
            None,
            ActionId::NoOp,
            false,
            None,
            "Nenhum item anterior na área de transferência",
        ));
    }
    vec![
        item(
            "Desfazer",
            Some("Ctrl+Z"),
            ActionId::Undo,
            true,
            None,
            "Desfaz a última alteração no texto",
        ),
        item(
            "Refazer",
            Some("Ctrl+Y"),
            ActionId::Redo,
            true,
            None,
            "Refaz a alteração desfeita",
        ),
        MenuNode::Separator,
        item(
            "Recortar",
            Some("Ctrl+X"),
            ActionId::Cut,
            true,
            None,
            "Recorta a seleção para a área de transferência",
        ),
        item(
            "Copiar",
            Some("Ctrl+C"),
            ActionId::Copy,
            true,
            None,
            "Copia a seleção para a área de transferência",
        ),
        item(
            "Colar",
            Some("Ctrl+V"),
            ActionId::Paste,
            true,
            None,
            "Cola o conteúdo da área de transferência",
        ),
        submenu(
            "Colar Anterior",
            "Cola um item anterior do histórico interno da área de transferência",
            paste_prev,
        ),
        MenuNode::Separator,
        item(
            "Selecionar Tudo",
            Some("Ctrl+A"),
            ActionId::SelectAll,
            true,
            None,
            "Seleciona todo o texto do documento",
        ),
        item(
            "Buscar",
            Some("Ctrl+F"),
            ActionId::Find,
            true,
            None,
            "Busca um texto no documento",
        ),
        item(
            "Substituir",
            Some("Ctrl+H"),
            ActionId::Replace,
            true,
            None,
            "Busca e substitui texto no documento",
        ),
    ]
}

fn view_menu(view: &ViewState) -> Vec<MenuNode> {
    vec![
        toggle_item(
            "Quebra de linha",
            ActionId::WordWrapToggle,
            view.word_wrap,
            "Alterna a quebra automática de linhas longas no editor",
        ),
        toggle_item(
            "Terminal",
            ActionId::ToggleTerminal,
            view.terminal,
            "Mostra ou oculta o terminal integrado (Ctrl+T / Ctrl+')",
        ),
        toggle_item(
            "Rodapé",
            ActionId::ToggleFooter,
            view.footer_visible,
            "Mostra ou oculta a barra de status na parte inferior",
        ),
        toggle_item(
            "Consumo de memória",
            ActionId::ShowMemoryToggle,
            view.show_memory,
            "Exibe o consumo total de memória do aplicativo no rodapé",
        ),
        toggle_item(
            "Números de linha",
            ActionId::LineNumbersToggle,
            view.show_line_numbers,
            "Exibe a numeração de linhas à esquerda do texto no editor",
        ),
        toggle_item(
            "Borda visível",
            ActionId::BorderToggle,
            view.border == EditorBorder::Visible,
            "Alterna a borda externa do editor (┌ ─ ┐ / apenas título no topo)",
        ),
        submenu(
            "Texto",
            "Exibe ou oculta caracteres invisíveis no editor",
            vec![
                toggle_item(
                    "Símbolos",
                    ActionId::ShowSymbols,
                    view.show_symbols,
                    "Mostra ou oculta símbolos especiais",
                ),
                toggle_item(
                    "Espaços",
                    ActionId::ShowSpaces,
                    view.show_spaces,
                    "Mostra ou oculta espaços",
                ),
                toggle_item(
                    "Tabs",
                    ActionId::ShowTabs,
                    view.show_tabs,
                    "Mostra ou oculta tabulações",
                ),
                toggle_item(
                    "Fim de linha",
                    ActionId::ShowEol,
                    view.show_eol,
                    "Mostra ou oculta marcadores de fim de linha",
                ),
                item(
                    "Tudo",
                    None,
                    ActionId::ShowAll,
                    true,
                    None,
                    "Alterna a exibição de todos os caracteres invisíveis",
                ),
            ],
        ),
        submenu(
            "Temas",
            "Seleciona a paleta de cores da interface",
            vec![
                radio_item(
                    "Escuro",
                    ActionId::ThemeDark,
                    view.theme == crate::theme::ThemeId::Dark,
                    "Aplica o tema escuro",
                ),
                radio_item(
                    "Claro",
                    ActionId::ThemeLight,
                    view.theme == crate::theme::ThemeId::Light,
                    "Aplica o tema claro",
                ),
                radio_item(
                    "Azul Clássico",
                    ActionId::ThemeClassicBlue,
                    view.theme == crate::theme::ThemeId::ClassicBlue,
                    "Aplica o tema azul clássico estilo Turbo Vision",
                ),
                radio_item(
                    "Matrix",
                    ActionId::ThemeMatrix,
                    view.theme == crate::theme::ThemeId::Matrix,
                    "Aplica o tema verde terminal estilo Matrix",
                ),
            ],
        ),
        submenu(
            "Colunas",
            "Define a quantidade de colunas de caracteres digitáveis por linha no editor",
            vec![
                radio_item(
                    "80",
                    ActionId::Column80,
                    view.guide_column == GuideColumn::Col80,
                    "Limita a linha a 80 colunas",
                ),
                radio_item(
                    "120",
                    ActionId::Column120,
                    view.guide_column == GuideColumn::Col120,
                    "Limita a linha a 120 colunas",
                ),
                radio_item(
                    "160",
                    ActionId::Column160,
                    view.guide_column == GuideColumn::Col160,
                    "Limita a linha a 160 colunas",
                ),
                radio_item(
                    "Ilimitado",
                    ActionId::ColumnUnlimited,
                    view.guide_column == GuideColumn::Unlimited,
                    "Remove o limite de colunas por linha",
                ),
            ],
        ),
        submenu(
            "Margem",
            "Define a distância entre a borda do editor e o texto, nos 4 eixos",
            vec![
                radio_item(
                    "Sem Margem",
                    ActionId::MarginNone,
                    view.margin == EditorMargin::None,
                    "Texto começa na linha 1 e coluna 1 logo após a borda",
                ),
                radio_item(
                    "Uma linha",
                    ActionId::MarginOneLine,
                    view.margin == EditorMargin::OneLine,
                    "1 linha acima/abaixo e 2 colunas à esquerda/direita",
                ),
                radio_item(
                    "Duas linhas",
                    ActionId::MarginTwoLines,
                    view.margin == EditorMargin::TwoLines,
                    "2 linhas acima/abaixo e 4 colunas à esquerda/direita",
                ),
            ],
        ),
    ]
}

fn format_menu(enc: FileEncoding, tab: Tabulation) -> Vec<MenuNode> {
    vec![
        submenu(
            "Codificação",
            "Define a codificação de caracteres do arquivo",
            vec![
                radio_item(
                    "UTF-8",
                    ActionId::EncodingUtf8,
                    enc == FileEncoding::Utf8,
                    "Salva e abre o arquivo em UTF-8 com BOM",
                ),
                radio_item(
                    "UTF-8 sem BOM",
                    ActionId::EncodingUtf8NoBom,
                    enc == FileEncoding::Utf8NoBom,
                    "Salva e abre o arquivo em UTF-8 sem BOM",
                ),
                radio_item(
                    "UTF-16 LE",
                    ActionId::EncodingUtf16Le,
                    enc == FileEncoding::Utf16Le,
                    "Salva e abre o arquivo em UTF-16 little-endian",
                ),
                radio_item(
                    "UTF-16 BE",
                    ActionId::EncodingUtf16Be,
                    enc == FileEncoding::Utf16Be,
                    "Salva e abre o arquivo em UTF-16 big-endian",
                ),
                radio_item(
                    "ISO-8859-1",
                    ActionId::EncodingIso88591,
                    enc == FileEncoding::Iso88591,
                    "Salva e abre o arquivo em ISO-8859-1",
                ),
                radio_item(
                    "ANSI",
                    ActionId::EncodingAnsi,
                    enc == FileEncoding::Ansi,
                    "Salva e abre o arquivo usando a codificação ANSI do sistema",
                ),
            ],
        ),
        submenu(
            "Tabulação",
            "Define como a tecla Tab insere espaços ou tabulação literal",
            vec![
                radio_item(
                    "2 espaços",
                    ActionId::TabSpaces2,
                    tab == Tabulation::Spaces2,
                    "Insere 2 espaços ao pressionar Tab",
                ),
                radio_item(
                    "4 espaços",
                    ActionId::TabSpaces4,
                    tab == Tabulation::Spaces4,
                    "Insere 4 espaços ao pressionar Tab",
                ),
                radio_item(
                    "8 espaços",
                    ActionId::TabSpaces8,
                    tab == Tabulation::Spaces8,
                    "Insere 8 espaços ao pressionar Tab",
                ),
                radio_item(
                    "Tab literal",
                    ActionId::TabLiteral,
                    tab == Tabulation::TabLiteral,
                    "Insere o caractere de tabulação literal ao pressionar Tab",
                ),
                item(
                    "Converter Tabulação",
                    None,
                    ActionId::ConvertTabulation,
                    true,
                    None,
                    "Converte indentação informando De/Para (2, 4, 8 espaços ou Tab literal)",
                ),
            ],
        ),
    ]
}

fn toggle_item(label: &'static str, action: ActionId, on: bool, help: &'static str) -> MenuNode {
    item(label, None, action, true, Some(on), help)
}

/// Item de seleção exclusiva (radio): check na opção ativa.
fn radio_item(label: &'static str, action: ActionId, selected: bool, help: &'static str) -> MenuNode {
    item(label, None, action, true, Some(selected), help)
}

fn submenu(label: &'static str, help: &'static str, children: Vec<MenuNode>) -> MenuNode {
    MenuNode::SubMenu {
        label,
        help: Some(help),
        children,
    }
}

fn item(
    label: impl Into<String>,
    shortcut: Option<&'static str>,
    action: ActionId,
    enabled: bool,
    checked: Option<bool>,
    help: &'static str,
) -> MenuNode {
    MenuNode::Item {
        label: label.into(),
        shortcut,
        action,
        enabled,
        checked,
        help: Some(help),
    }
}

fn first_enabled_index(nodes: &[MenuNode]) -> usize {
    for (i, node) in nodes.iter().enumerate() {
        match node {
            MenuNode::Item { enabled: true, .. } => return i,
            MenuNode::SubMenu { .. } => return i,
            _ => {}
        }
    }
    0
}

fn current_nodes<'a>(bar: &'a MenuBar, state: &MenuState) -> Option<&'a [MenuNode]> {
    let top = state.open_top?;
    let mut nodes = &bar.tops[top].children[..];
    for &idx in &state.focus_path {
        match nodes.get(idx)? {
            MenuNode::SubMenu { children, .. } => nodes = children,
            _ => break,
        }
    }
    Some(nodes)
}

fn submenu_at_path<'a>(nodes: &'a [MenuNode], path: &[usize]) -> Option<&'a [MenuNode]> {
    let mut current = nodes;
    for &idx in path {
        match current.get(idx)? {
            MenuNode::SubMenu { children, .. } => current = children,
            _ => return None,
        }
    }
    Some(current)
}

pub fn handle_key(bar: &MenuBar, state: &mut MenuState, key: KeyEvent) -> MenuEventResult {
    if key.modifiers.contains(KeyModifiers::CONTROL) || key.modifiers.contains(KeyModifiers::SHIFT) {
        if !state.is_open() {
            return MenuEventResult::None;
        }
    }

    if !state.is_open() {
        if key.modifiers.contains(KeyModifiers::ALT) {
            if let KeyCode::Char(c) = key.code {
                if let Some(idx) = bar.top_index_by_mnemonic(c) {
                    state.open_top_menu(idx, bar);
                    return MenuEventResult::Consumed;
                }
            }
        }
        return MenuEventResult::None;
    }

    match key.code {
        KeyCode::Esc => {
            state.close();
            MenuEventResult::Closed
        }
        KeyCode::Up => {
            move_focus(bar, state, -1);
            MenuEventResult::Consumed
        }
        KeyCode::Down => {
            move_focus(bar, state, 1);
            MenuEventResult::Consumed
        }
        KeyCode::Right => {
            let Some(top) = state.open_top else {
                return MenuEventResult::Consumed;
            };
            if matches!(
                node_at_path(&bar.tops[top].children, &state.focus_path),
                Some(MenuNode::SubMenu { .. })
            ) {
                open_submenu(bar, state);
            } else {
                switch_top_menu(bar, state, 1);
            }
            MenuEventResult::Consumed
        }
        KeyCode::Left => {
            if state.focus_path.len() > 1 {
                state.focus_path.pop();
                state.expanded_submenus.truncate(state.focus_path.len().saturating_sub(1));
            } else {
                switch_top_menu(bar, state, -1);
            }
            MenuEventResult::Consumed
        }
        KeyCode::Enter => activate_focused(bar, state),
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            if activate_by_mnemonic(bar, state, c) {
                MenuEventResult::Consumed
            } else {
                MenuEventResult::Consumed
            }
        }
        _ => MenuEventResult::Consumed,
    }
}

pub fn handle_mouse(bar: &MenuBar, state: &mut MenuState, mouse: MouseEvent) -> MenuEventResult {
    let MouseEventKind::Down(MouseButton::Left) = mouse.kind else {
        if state.is_open() {
            if let MouseEventKind::Moved = mouse.kind {
                if let Some(path) = hit_test_item(state, mouse.column, mouse.row) {
                    state.focus_path = path;
                    sync_expanded_submenus(state);
                    return MenuEventResult::Consumed;
                }
            }
        }
        return MenuEventResult::None;
    };

    for (i, rect) in state.top_hit_areas.iter().enumerate() {
        if rect_contains(*rect, mouse.column, mouse.row) {
            state.toggle_top(i, bar);
            return MenuEventResult::Consumed;
        }
    }

    if state.is_open() {
        if let Some((path, action)) = hit_test_action(bar, state, mouse.column, mouse.row) {
            state.focus_path = path;
            if let Some(action) = action {
                state.close();
                return MenuEventResult::Action(action);
            }
            open_submenu(bar, state);
            return MenuEventResult::Consumed;
        }
        state.close();
        return MenuEventResult::Closed;
    }

    MenuEventResult::None
}

fn activate_focused(bar: &MenuBar, state: &mut MenuState) -> MenuEventResult {
    let Some(top) = state.open_top else {
        return MenuEventResult::None;
    };
    let Some(node) = node_at_path(&bar.tops[top].children, &state.focus_path) else {
        return MenuEventResult::Consumed;
    };
    match node {
        MenuNode::Item { action, enabled: true, .. } => {
            state.close();
            MenuEventResult::Action(*action)
        }
        MenuNode::SubMenu { .. } => {
            open_submenu(bar, state);
            MenuEventResult::Consumed
        }
        _ => MenuEventResult::Consumed,
    }
}

fn activate_by_mnemonic(bar: &MenuBar, state: &MenuState, c: char) -> bool {
    let Some(_top) = state.open_top else {
        return false;
    };
    let Some(nodes) = current_nodes(bar, state) else {
        return false;
    };
    for (i, node) in nodes.iter().enumerate() {
        let label = match node {
            MenuNode::Item { label, .. } => label.as_str(),
            MenuNode::SubMenu { label, .. } => label,
            MenuNode::Separator => continue,
        };
        if label
            .trim()
            .chars()
            .next()
            .is_some_and(|ch| ch.eq_ignore_ascii_case(&c))
        {
            return true;
        }
        let _ = i;
    }
    false
}

fn switch_top_menu(bar: &MenuBar, state: &mut MenuState, delta: i32) {
    let Some(current) = state.open_top else {
        return;
    };
    let count = bar.tops.len();
    if count == 0 {
        return;
    }
    let next = (current as i32 + delta).rem_euclid(count as i32) as usize;
    state.open_top_menu(next, bar);
}

fn open_submenu(bar: &MenuBar, state: &mut MenuState) {
    let Some(top) = state.open_top else {
        return;
    };
    let path = state.focus_path.clone();
    let Some(node) = node_at_path(&bar.tops[top].children, &path) else {
        return;
    };
    if matches!(node, MenuNode::SubMenu { .. }) {
        state.expanded_submenus = state.focus_path.clone();
        state.focus_path.push(first_enabled_index(
            submenu_at_path(&bar.tops[top].children, &path).unwrap_or(&[]),
        ));
    }
}

fn sync_expanded_submenus(state: &mut MenuState) {
    let mut keep = 0;
    for (i, &expanded) in state.expanded_submenus.iter().enumerate() {
        if state.focus_path.get(i) == Some(&expanded) {
            keep = i + 1;
        } else {
            break;
        }
    }
    state.expanded_submenus.truncate(keep);
}

fn move_focus(bar: &MenuBar, state: &mut MenuState, delta: i32) {
    let Some(top) = state.open_top else {
        return;
    };
    let parent_path: Vec<usize> = if state.focus_path.len() > 1 {
        state.focus_path[..state.focus_path.len() - 1].to_vec()
    } else {
        vec![]
    };
    let nodes = if parent_path.is_empty() {
        &bar.tops[top].children
    } else {
        submenu_at_path(&bar.tops[top].children, &parent_path).unwrap_or(&[])
    };
    let selectable: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !matches!(n, MenuNode::Separator))
        .map(|(i, _)| i)
        .collect();
    if selectable.is_empty() {
        return;
    }
    let current_idx = *state.focus_path.last().unwrap_or(&0);
    let pos = selectable.iter().position(|&i| i == current_idx).unwrap_or(0);
    let new_pos = (pos as i32 + delta).rem_euclid(selectable.len() as i32) as usize;
    if parent_path.is_empty() {
        state.focus_path = vec![selectable[new_pos]];
    } else {
        state.focus_path.truncate(parent_path.len());
        state.focus_path.push(selectable[new_pos]);
    }
    sync_expanded_submenus(state);
}

fn node_at_path<'a>(nodes: &'a [MenuNode], path: &[usize]) -> Option<&'a MenuNode> {
    let mut current = nodes.get(*path.first()?)?;
    for &idx in path.iter().skip(1) {
        match current {
            MenuNode::SubMenu { children, .. } => current = children.get(idx)?,
            _ => return None,
        }
    }
    Some(current)
}

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn hit_test_item(state: &MenuState, x: u16, y: u16) -> Option<Vec<usize>> {
    state
        .item_hit_areas
        .iter()
        .find(|(_, r)| rect_contains(*r, x, y))
        .map(|(p, _)| p.clone())
}

fn hit_test_action(
    bar: &MenuBar,
    state: &MenuState,
    x: u16,
    y: u16,
) -> Option<(Vec<usize>, Option<ActionId>)> {
    for (path, rect) in &state.item_hit_areas {
        if rect_contains(*rect, x, y) {
            return Some((path.clone(), action_at_path(bar, state, path)));
        }
    }
    None
}

fn action_at_path(bar: &MenuBar, state: &MenuState, path: &[usize]) -> Option<ActionId> {
    let top = state.open_top?;
    let node = node_at_path(&bar.tops[top].children, path)?;
    match node {
        MenuNode::Item { action, enabled: true, .. } => Some(*action),
        _ => None,
    }
}

pub fn render_bar(frame: &mut Frame, area: Rect, bar: &MenuBar, state: &mut MenuState, palette: ThemePalette) {
    state.bar_area = area;
    state.top_hit_areas.clear();

    panel::fill_rect(frame, area, palette.menu_bar_style());

    let mut x = area.x;
    for (i, top) in bar.tops.iter().enumerate() {
        let width = top.label.len() as u16;
        let item_area = Rect {
            x,
            y: area.y,
            width,
            height: area.height.max(1),
        };
        state.top_hit_areas.push(item_area);

        let style = if state.open_top == Some(i) {
            palette.menu_top_active_style()
        } else {
            palette.menu_bar_style()
        };
        panel::fill_rect(frame, item_area, style);
        frame.render_widget(
            Paragraph::new(Line::from(menu_top_spans(top.label, top.mnemonic, style, palette))).style(style),
            item_area,
        );
        x = x.saturating_add(width);
    }

    if x < area.x.saturating_add(area.width) {
        panel::fill_rect(
            frame,
            Rect {
                x,
                y: area.y,
                width: area.x.saturating_add(area.width).saturating_sub(x),
                height: area.height.max(1),
            },
            palette.menu_bar_style(),
        );
    }
}

/// Dropdown renderizado **por cima** do editor (chamar após `draw_editor`).
pub fn render_dropdown(frame: &mut Frame, bar: &MenuBar, state: &mut MenuState, palette: ThemePalette) {
    if let Some(top_idx) = state.open_top {
        render_panels(frame, bar, state, top_idx, state.bar_area, palette);
    }
}

fn render_panels(
    frame: &mut Frame,
    bar: &MenuBar,
    state: &mut MenuState,
    top_idx: usize,
    bar_area: Rect,
    palette: ThemePalette,
) {
    state.panel_areas.clear();
    state.item_hit_areas.clear();

    let mut origin_x = bar_area.x;
    for (i, top) in bar.tops.iter().enumerate() {
        if i == top_idx {
            break;
        }
        origin_x = origin_x.saturating_add(top.label.len() as u16);
    }

    let mut panel_x = origin_x;
    let mut panel_y = bar_area.y.saturating_add(1);
    let mut nodes = &bar.tops[top_idx].children[..];
    let mut path_prefix: Vec<usize> = vec![];

    loop {
        let (width, mut height) = measure_panel(nodes);
        let max_h = frame
            .area()
            .height
            .saturating_sub(panel_y)
            .saturating_sub(1);
        height = height.min(max_h.max(3));
        let area = Rect {
            x: panel_x,
            y: panel_y,
            width: width.max(20),
            height,
        };
        if area.height < 3 || area.width < 3 {
            break;
        }
        panel::render_drop_shadow(frame, area, palette);
        panel::render_frame(
            frame,
            area,
            palette.menu_panel_style(),
            palette.menu_border_style(),
            PanelBorder::Double,
        );
        state.panel_areas.push(area);

        let inner = panel::inner_rect(area);
        let focused_leaf = state.focus_path.get(path_prefix.len()).copied();
        let inner_w = inner.width as usize;
        let visible_rows = inner.height as usize;
        for (i, node) in nodes.iter().enumerate().take(visible_rows) {
            let row_y = inner.y.saturating_add(i as u16);
            let row_area = Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: 1,
            };
            let mut item_path = path_prefix.clone();
            item_path.push(i);
            state.item_hit_areas.push((item_path.clone(), row_area));

            let style = match node {
                MenuNode::Item { enabled: false, .. } => palette.menu_item_disabled_style(),
                MenuNode::Separator => palette.menu_item_disabled_style(),
                _ if focused_leaf == Some(i) => palette.menu_item_focus_style(),
                _ => palette.menu_item_style(),
            };

            match node {
                MenuNode::Separator => {
                    panel::render_separator_row(
                        frame,
                        area,
                        row_y,
                        PanelBorder::Double,
                        palette.menu_border_style(),
                    );
                }
                _ => {
                    let focused = focused_leaf == Some(i);
                    let line = menu_node_line(
                        node,
                        inner_w,
                        style,
                        palette.menu_marker_style(focused),
                        palette.menu_shortcut_style(focused),
                    );
                    panel::render_content_line(frame, area, i as u16, line);
                }
            }
        }

        let Some(focus_idx) = state.expanded_submenus.get(path_prefix.len()).copied() else {
            break;
        };
        match nodes.get(focus_idx) {
            Some(MenuNode::SubMenu { children, .. }) => {
                path_prefix.push(focus_idx);
                panel_x = area.x.saturating_add(area.width);
                panel_y = inner.y.saturating_add(focus_idx as u16);
                nodes = children;
            }
            _ => break,
        }
    }
}

fn menu_top_spans(
    label: &str,
    mnemonic: char,
    base: ratatui::style::Style,
    palette: ThemePalette,
) -> Vec<Span<'static>> {
    let trimmed = label.trim();
    let hot = palette.menu_hotkey_style();
    let mut out = vec![Span::styled(" ", base)];
    let mut pending = String::new();
    let mut mnemonic_used = false;

    for ch in trimmed.chars() {
        if !mnemonic_used && ch.eq_ignore_ascii_case(&mnemonic) {
            mnemonic_used = true;
            if !pending.is_empty() {
                out.push(Span::styled(std::mem::take(&mut pending), base));
            }
            out.push(Span::styled(ch.to_string(), hot));
        } else {
            pending.push(ch);
        }
    }
    if !pending.is_empty() {
        out.push(Span::styled(pending, base));
    }
    out.push(Span::styled(" ", base));
    out
}

fn measure_panel(nodes: &[MenuNode]) -> (u16, u16) {
    let mut max_w = 0usize;
    for node in nodes {
        max_w = max_w.max(line_width(node));
    }
    panel::outer_size(max_w, nodes.len())
}

fn item_right_text(node: &MenuNode) -> Option<String> {
    match item_right_slot(node)? {
        MenuRightSlot::Shortcut(s) => Some(s),
        MenuRightSlot::SubmenuArrow => Some(cp437::SUBMENU_ARROW.to_string()),
    }
}

enum MenuRightSlot {
    Shortcut(String),
    SubmenuArrow,
}

fn item_right_slot(node: &MenuNode) -> Option<MenuRightSlot> {
    match node {
        MenuNode::Item { shortcut, .. } => shortcut.map(|s| MenuRightSlot::Shortcut(s.to_string())),
        MenuNode::SubMenu { .. } => Some(MenuRightSlot::SubmenuArrow),
        MenuNode::Separator => None,
    }
}

/// Primeira coluna interna: espaço ou `√` (substitutivo, não extra).
fn menu_left_text(label: &str, checked: Option<bool>) -> String {
    match checked {
        Some(true) => format!("{}{label}", cp437::CHECK_ON),
        Some(false) | None => format!(" {label}"),
    }
}

fn menu_left_spans(
    label: &str,
    checked: Option<bool>,
    base_style: ratatui::style::Style,
    marker_style: ratatui::style::Style,
) -> Vec<Span<'static>> {
    match checked {
        Some(true) => vec![
            Span::styled(cp437::CHECK_ON.to_string(), marker_style),
            Span::styled(label.to_string(), base_style),
        ],
        Some(false) | None => vec![Span::styled(format!(" {label}"), base_style)],
    }
}

fn menu_node_line(
    node: &MenuNode,
    width: usize,
    base_style: ratatui::style::Style,
    marker_style: ratatui::style::Style,
    shortcut_style: ratatui::style::Style,
) -> Line<'static> {
    match node {
        MenuNode::Separator => Line::from(""),
        MenuNode::Item {
            label,
            checked,
            ..
        } => {
            let left_len = menu_left_text(label, *checked).chars().count();
            let left_spans = menu_left_spans(label, *checked, base_style, marker_style);
            let (right_text, right_style) = match item_right_slot(node) {
                Some(MenuRightSlot::Shortcut(s)) => (Some(s), shortcut_style),
                None | Some(MenuRightSlot::SubmenuArrow) => (None, base_style),
            };
            build_menu_row_line_spans(
                left_spans,
                left_len,
                right_text.as_deref(),
                width,
                base_style,
                right_style,
            )
        }
        MenuNode::SubMenu { label, .. } => {
            let left = format!(" {label}");
            let left_len = left.chars().count();
            let arrow = cp437::SUBMENU_ARROW.to_string();
            build_menu_row_line_spans(
                vec![Span::styled(left, base_style)],
                left_len,
                Some(&arrow),
                width,
                base_style,
                marker_style,
            )
        }
    }
}

fn build_menu_row_line_spans(
    mut left_spans: Vec<Span<'static>>,
    left_len: usize,
    right: Option<&str>,
    width: usize,
    base_style: ratatui::style::Style,
    right_style: ratatui::style::Style,
) -> Line<'static> {
    let right_part = match right {
        Some(text) => format!(" {text} "),
        None => String::new(),
    };
    let pad = width.saturating_sub(left_len + right_part.chars().count());
    left_spans.push(Span::styled(" ".repeat(pad), base_style));
    if let Some(text) = right {
        left_spans.push(Span::styled(" ", base_style));
        left_spans.push(Span::styled(text.to_string(), right_style));
        left_spans.push(Span::styled(" ", base_style));
    }
    Line::from(left_spans)
}

fn line_width(node: &MenuNode) -> usize {
    match node {
        MenuNode::Separator => 12,
        MenuNode::Item { label, checked, .. } => {
            menu_row_width(&menu_left_text(label, *checked), item_right_text(node).as_deref())
        }
        MenuNode::SubMenu { label, .. } => {
            menu_row_width(&format!(" {label}"), item_right_text(node).as_deref())
        }
    }
}

/// Linha de menu: ` {título}… {direita} ` com margem à direita antes da borda.
fn menu_row_width(left: &str, right: Option<&str>) -> usize {
    let right_len = right.map(|r| r.chars().count() + 2).unwrap_or(0);
    left.chars().count() + right_len + 1
}
