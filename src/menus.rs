use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::clipboard::Clipboard;
use crate::encoding::{FileEncoding, Tabulation};
use crate::recent::{display_name, RecentFiles};
use crate::theme::ThemePalette;
use crate::view_state::{GuideColumn, ViewState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    NoOp,
    Quit,
    New,
    Open,
    Recent,
    Save,
    SaveAs,
    Close,
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
    ThemeCustom,
    ToggleSidePanel,
    ToggleTerminal,
    ToggleFooter,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    WordWrapOn,
    WordWrapOff,
    ShowSymbols,
    ShowSpaces,
    ShowTabs,
    ShowEol,
    ShowAll,
    Column80,
    Column120,
    Column160,
    ColumnUnlimited,
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
    PasteClip(usize),
    OpenRecent(usize),
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
    },
    SubMenu {
        label: &'static str,
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
        self.panel_areas.clear();
        self.item_hit_areas.clear();
    }

    pub fn open_top_menu(&mut self, index: usize, bar: &MenuBar) {
        if index >= bar.tops.len() {
            return;
        }
        self.open_top = Some(index);
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
    pub fn build(recent: &RecentFiles, view: &ViewState, enc: FileEncoding, tab: Tabulation, clip: &Clipboard) -> Self {
        Self {
            tops: vec![
                MenuTopItem {
                    label: " Arquivo ",
                    mnemonic: 'A',
                    children: file_menu(recent),
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

fn file_menu(recent: &RecentFiles) -> Vec<MenuNode> {
    let mut nodes = vec![
        item("Novo", Some("Ctrl+N"), ActionId::New, true, None),
        item("Abrir", Some("Ctrl+O"), ActionId::Open, true, None),
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
            )
        })
        .collect();
    if recent_children.is_empty() {
        recent_children.push(item("(vazio)", None, ActionId::NoOp, false, None));
    }
    nodes.push(MenuNode::SubMenu {
        label: "Recentes",
        children: recent_children,
    });
    nodes.extend([
        MenuNode::Separator,
        item("Salvar", Some("Ctrl+S"), ActionId::Save, true, None),
        item("Salvar Como", Some("Ctrl+Shift+S"), ActionId::SaveAs, true, None),
        MenuNode::Separator,
        item("Fechar", Some("Ctrl+W"), ActionId::Close, true, None),
        item("Sair", Some("Ctrl+Q"), ActionId::Quit, true, None),
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
            )
        })
        .collect();
    if paste_prev.is_empty() {
        paste_prev.push(item("(vazio)", None, ActionId::NoOp, false, None));
    }
    vec![
        item("Desfazer", Some("Ctrl+Z"), ActionId::Undo, true, None),
        item("Refazer", Some("Ctrl+Y"), ActionId::Redo, true, None),
        MenuNode::Separator,
        item("Recortar", Some("Ctrl+X"), ActionId::Cut, true, None),
        item("Copiar", Some("Ctrl+C"), ActionId::Copy, true, None),
        item("Colar", Some("Ctrl+V"), ActionId::Paste, true, None),
        MenuNode::SubMenu {
            label: "Colar Anterior",
            children: paste_prev,
        },
        MenuNode::Separator,
        item("Selecionar Tudo", Some("Ctrl+A"), ActionId::SelectAll, true, None),
        item("Buscar", Some("Ctrl+F"), ActionId::Find, true, None),
        item("Substituir", Some("Ctrl+H"), ActionId::Replace, true, None),
    ]
}

fn view_menu(view: &ViewState) -> Vec<MenuNode> {
    vec![
        MenuNode::SubMenu {
            label: "Zoom",
            children: vec![
                item("Zoom In", None, ActionId::ZoomIn, true, None),
                item("Zoom Out", None, ActionId::ZoomOut, true, None),
                item("Reset Zoom", None, ActionId::ZoomReset, true, None),
            ],
        },
        MenuNode::SubMenu {
            label: "Word Wrap",
            children: vec![
                item("Ativar", None, ActionId::WordWrapOn, true, Some(view.word_wrap)),
                item("Desativar", None, ActionId::WordWrapOff, true, Some(!view.word_wrap)),
            ],
        },
        MenuNode::SubMenu {
            label: "Mostrar",
            children: vec![
                item("Símbolos", None, ActionId::ShowSymbols, true, Some(view.show_symbols)),
                item("Espaços", None, ActionId::ShowSpaces, true, Some(view.show_spaces)),
                item("Tabs", None, ActionId::ShowTabs, true, Some(view.show_tabs)),
                item("Fim de linha", None, ActionId::ShowEol, true, Some(view.show_eol)),
                item("Tudo", None, ActionId::ShowAll, true, None),
            ],
        },
        MenuNode::SubMenu {
            label: "Painel Lateral",
            children: vec![
                item("Mostrar", None, ActionId::ToggleSidePanel, true, Some(view.side_panel)),
                item("Ocultar", None, ActionId::ToggleSidePanel, true, Some(!view.side_panel)),
            ],
        },
        MenuNode::SubMenu {
            label: "Terminal",
            children: vec![
                item("Mostrar", None, ActionId::ToggleTerminal, true, Some(view.terminal)),
                item("Ocultar", None, ActionId::ToggleTerminal, true, Some(!view.terminal)),
            ],
        },
        MenuNode::SubMenu {
            label: "Rodapé",
            children: vec![
                item("Mostrar", None, ActionId::ToggleFooter, true, Some(view.footer_visible)),
                item("Ocultar", None, ActionId::ToggleFooter, true, Some(!view.footer_visible)),
            ],
        },
        MenuNode::SubMenu {
            label: "Temas",
            children: vec![
                item("Escuro", None, ActionId::ThemeDark, true, Some(view.theme == crate::theme::ThemeId::Dark)),
                item("Claro", None, ActionId::ThemeLight, true, Some(view.theme == crate::theme::ThemeId::Light)),
                item(
                    "Azul Clássico",
                    None,
                    ActionId::ThemeClassicBlue,
                    true,
                    Some(view.theme == crate::theme::ThemeId::ClassicBlue),
                ),
                item("Personalizado", None, ActionId::ThemeCustom, true, None),
            ],
        },
        MenuNode::SubMenu {
            label: "Colunas",
            children: vec![
                item("80", None, ActionId::Column80, true, Some(view.guide_column == GuideColumn::Col80)),
                item("120", None, ActionId::Column120, true, Some(view.guide_column == GuideColumn::Col120)),
                item("160", None, ActionId::Column160, true, Some(view.guide_column == GuideColumn::Col160)),
                item("Ilimitado", None, ActionId::ColumnUnlimited, true, Some(view.guide_column == GuideColumn::Unlimited)),
            ],
        },
    ]
}

fn format_menu(enc: FileEncoding, tab: Tabulation) -> Vec<MenuNode> {
    vec![
        MenuNode::SubMenu {
            label: "Codificação",
            children: vec![
                checked_item("UTF-8", ActionId::EncodingUtf8, enc == FileEncoding::Utf8),
                checked_item("UTF-8 sem BOM", ActionId::EncodingUtf8NoBom, enc == FileEncoding::Utf8NoBom),
                checked_item("UTF-16 LE", ActionId::EncodingUtf16Le, enc == FileEncoding::Utf16Le),
                checked_item("UTF-16 BE", ActionId::EncodingUtf16Be, enc == FileEncoding::Utf16Be),
                checked_item("ISO-8859-1", ActionId::EncodingIso88591, enc == FileEncoding::Iso88591),
                checked_item("ANSI", ActionId::EncodingAnsi, enc == FileEncoding::Ansi),
            ],
        },
        MenuNode::SubMenu {
            label: "Tabulação",
            children: vec![
                checked_item("2 espaços", ActionId::TabSpaces2, tab == Tabulation::Spaces2),
                checked_item("4 espaços", ActionId::TabSpaces4, tab == Tabulation::Spaces4),
                checked_item("8 espaços", ActionId::TabSpaces8, tab == Tabulation::Spaces8),
                checked_item("Tab literal", ActionId::TabLiteral, tab == Tabulation::TabLiteral),
            ],
        },
    ]
}

fn checked_item(label: &'static str, action: ActionId, on: bool) -> MenuNode {
    item(label, None, action, true, Some(on))
}

fn item(
    label: impl Into<String>,
    shortcut: Option<&'static str>,
    action: ActionId,
    enabled: bool,
    checked: Option<bool>,
) -> MenuNode {
    MenuNode::Item {
        label: label.into(),
        shortcut,
        action,
        enabled,
        checked,
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
        if key.code == KeyCode::F(10) {
            state.open_top_menu(0, bar);
            return MenuEventResult::Consumed;
        }
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
            open_submenu(bar, state);
            MenuEventResult::Consumed
        }
        KeyCode::Left => {
            if state.focus_path.len() > 1 {
                state.focus_path.pop();
            } else {
                state.close();
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

fn open_submenu(bar: &MenuBar, state: &mut MenuState) {
    let Some(top) = state.open_top else {
        return;
    };
    let path = state.focus_path.clone();
    let Some(node) = node_at_path(&bar.tops[top].children, &path) else {
        return;
    };
    if matches!(node, MenuNode::SubMenu { .. }) {
        state.focus_path.push(first_enabled_index(
            submenu_at_path(&bar.tops[top].children, &path).unwrap_or(&[]),
        ));
    }
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

pub fn render(frame: &mut Frame, area: Rect, bar: &MenuBar, state: &mut MenuState, palette: ThemePalette) {
    state.bar_area = area;
    state.top_hit_areas.clear();

    let mut spans = Vec::new();
    let mut x = area.x;
    for (i, top) in bar.tops.iter().enumerate() {
        let width = top.label.len() as u16;
        state.top_hit_areas.push(Rect {
            x,
            y: area.y,
            width,
            height: 1,
        });
        let style = if state.open_top == Some(i) {
            palette.menu_top_active_style()
        } else {
            palette.menu_bar_style()
        };
        spans.push(Span::styled(top.label, style));
        x = x.saturating_add(width);
        spans.push(Span::styled(" ", palette.menu_bar_style()));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(palette.menu_bar_style()),
        area,
    );

    if let Some(top_idx) = state.open_top {
        render_panels(frame, bar, state, top_idx, area, palette);
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
        origin_x = origin_x.saturating_add(top.label.len() as u16 + 1);
    }

    let mut panel_x = origin_x;
    let mut panel_y = bar_area.y.saturating_add(1);
    let mut nodes = &bar.tops[top_idx].children[..];
    let mut path_prefix: Vec<usize> = vec![];

    loop {
        let (width, height, _lines) = measure_panel(nodes, palette);
        let area = Rect {
            x: panel_x,
            y: panel_y,
            width: width.max(20),
            height,
        };
        if area.y.saturating_add(area.height) > frame.area().height {
            break;
        }
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(palette.menu_border_style())
            .style(palette.menu_panel_style());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        state.panel_areas.push(area);

        let focused_leaf = state.focus_path.get(path_prefix.len()).copied();
        for (i, node) in nodes.iter().enumerate() {
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

            let focused = focused_leaf == Some(i);
            let line = format_menu_line(node, palette, inner.width as usize);
            let style = match node {
                MenuNode::Item { enabled: false, .. } => palette.menu_item_disabled_style(),
                _ if focused => palette.menu_item_focus_style(),
                _ => palette.menu_item_style(),
            };
            frame.render_widget(Paragraph::new(line).style(style), row_area);
        }

        let Some(focus_idx) = state.focus_path.get(path_prefix.len()).copied() else {
            break;
        };
        match nodes.get(focus_idx) {
            Some(MenuNode::SubMenu { children, .. }) => {
                path_prefix.push(focus_idx);
                panel_x = area.x.saturating_add(area.width);
                panel_y = area.y.saturating_add(focus_idx as u16);
                nodes = children;
            }
            _ => break,
        }
    }
}

fn measure_panel(nodes: &[MenuNode], palette: ThemePalette) -> (u16, u16, Vec<Line<'static>>) {
    let mut max_w = 0usize;
    for node in nodes {
        max_w = max_w.max(line_width(node));
    }
    let width = (max_w as u16).saturating_add(4);
    let height = nodes.len() as u16;
    let _ = palette;
    (width, height, vec![])
}

fn line_width(node: &MenuNode) -> usize {
    match node {
        MenuNode::Separator => 12,
        MenuNode::Item { label, shortcut, .. } => {
            label.len() + shortcut.map(|s| s.len() + 2).unwrap_or(0) + 4
        }
        MenuNode::SubMenu { label, .. } => label.len() + 4,
    }
}

fn format_menu_line(node: &MenuNode, palette: ThemePalette, width: usize) -> Line<'static> {
    match node {
        MenuNode::Separator => Line::from(Span::styled(
            "────────────",
            palette.menu_item_disabled_style(),
        )),
        MenuNode::Item { label, shortcut, checked, .. } => {
            let mark = checked.filter(|c| *c).map(|_| "✓ ").unwrap_or_default();
            let mut text = format!(" {mark}{label}");
            if let Some(sc) = shortcut {
                let pad = width.saturating_sub(text.len() + sc.len() + 1);
                text.push_str(&" ".repeat(pad));
                text.push(' ');
                text.push_str(sc);
            }
            Line::from(text)
        }
        MenuNode::SubMenu { label, .. } => Line::from(format!(" {label} ►")),
    }
}
