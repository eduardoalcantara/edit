use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use ratatui::Frame;

use crate::app::App;
use crate::view_state::InputFocus;

use super::layer::{InputResult, LayerId, UiLayer};
use super::layout::UiLayout;
use super::layers::all_layers;

/// Compositor central: empilha camadas, pinta de baixo para cima, input de cima para baixo.
pub struct Compositor;

impl Compositor {
    pub fn paint(frame: &mut Frame<'_>, app: &mut App) {
        if app.modal.is_active() && app.menu_state.is_open() {
            app.menu_state.close();
        }

        let layout = UiLayout::compute(frame.area(), app);
        let palette = app.theme.palette();

        let mut layers: Vec<&dyn UiLayer> = all_layers()
            .into_iter()
            .filter(|layer| layer.is_visible(app))
            .collect();
        layers.sort_by_key(|layer| layer.id().z());

        for layer in layers {
            layer.paint(frame, app, layout, palette);
        }
    }

    pub fn dispatch(app: &mut App, event: Event) {
        let layout = UiLayout::compute(
            ratatui::layout::Rect {
                x: 0,
                y: 0,
                width: app.last_frame_width,
                height: app.last_frame_height,
            },
            app,
        );

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                Self::dispatch_key(app, key, layout);
            }
            Event::Mouse(mouse) => {
                Self::dispatch_mouse(app, mouse, layout);
            }
            Event::Resize(w, h) => {
                app.last_frame_width = w;
                app.last_frame_height = h;
                app.sync_terminal_pty_size();
            }
            _ => {}
        }
    }

    fn dispatch_key(app: &mut App, key: KeyEvent, layout: UiLayout) {
        if key.code == KeyCode::F(4) && key.modifiers.contains(KeyModifiers::ALT) {
            app.request_quit();
            return;
        }
        if matches!(key.code, KeyCode::Char('q' | 'Q')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            app.request_quit();
            return;
        }

        if !app.menu_state.is_open() && !app.modal.is_active() {
            if handle_global_chords(app, key) {
                return;
            }
            if handle_global_function_keys(app, key) {
                return;
            }
        }

        dispatch_key_to_layers(app, key, layout);
    }

    fn dispatch_mouse(app: &mut App, mouse: MouseEvent, layout: UiLayout) {
        let mut layers: Vec<&dyn UiLayer> = all_layers()
            .into_iter()
            .filter(|layer| layer.is_visible(app))
            .collect();
        layers.sort_by_key(|layer| std::cmp::Reverse(layer.id().z()));

        // Mouse usa hit-test por camada; foco do teclado não bloqueia clique no editor.
        let block_background = app.modal.is_active();
        for layer in layers {
            if block_background && !layer.captures_input(app) {
                continue;
            }
            if layer.on_mouse(mouse, app, layout) == InputResult::Consumed {
                return;
            }
        }
    }
}

fn dispatch_key_to_layers(app: &mut App, key: KeyEvent, layout: UiLayout) {
    let mut layers: Vec<&dyn UiLayer> = all_layers()
        .into_iter()
        .filter(|layer| layer.is_visible(app))
        .collect();
    layers.sort_by_key(|layer| std::cmp::Reverse(layer.id().z()));

    for layer in &layers {
        if layer.id() == LayerId::Modal && layer.on_key(key, app, layout) == InputResult::Consumed {
            return;
        }
    }
    if app.menu_state.is_open() {
        for layer in &layers {
            if layer.id() == LayerId::MenuDropdown
                && layer.on_key(key, app, layout) == InputResult::Consumed
            {
                return;
            }
        }
    }
    for layer in &layers {
        if layer.id() == LayerId::MenuBar && layer.on_key(key, app, layout) == InputResult::Consumed {
            return;
        }
    }

    let focus_id = if app.view.terminal && app.input_focus == InputFocus::Terminal {
        LayerId::Terminal
    } else {
        LayerId::Editor
    };
    for layer in &layers {
        if layer.id() == focus_id && layer.on_key(key, app, layout) == InputResult::Consumed {
            return;
        }
    }
}

/// Texto de ajuda à esquerda (hover do rodapé, menu/modal ou ação recente).
pub fn footer_help_left(app: &App) -> String {
    if let Some(hint) = &app.footer_hover_help {
        return hint.clone();
    }
    footer_help_default(app)
}

fn footer_help_default(app: &App) -> String {
    let mut layers: Vec<&dyn UiLayer> = all_layers()
        .into_iter()
        .filter(|layer| layer.is_visible(app))
        .collect();
    layers.sort_by_key(|layer| std::cmp::Reverse(layer.id().z()));

    for layer in layers {
        if let Some(hint) = layer.footer_hint(app) {
            return hint;
        }
    }
    app.status_message.clone()
}

/// Ação ao clicar em um grupo do rodapé direito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterClick {
    GoToLine,
    OpenFormatEncoding,
}

pub struct FooterSegment {
    pub text: String,
    pub help: &'static str,
    pub click: Option<FooterClick>,
}

fn footer_active_focus(app: &App) -> &'static str {
    if app.modal.is_active() {
        "Diálogo"
    } else if app.menu_state.is_open() {
        "Menu"
    } else if app.view.terminal && app.input_focus == InputFocus::Terminal {
        "Terminal"
    } else {
        "Editor"
    }
}

pub fn footer_focus_label(app: &App) -> String {
    footer_active_focus(app).to_string()
}

pub fn footer_segments(app: &App) -> Vec<FooterSegment> {
    let (ln, col) = app.editor.cursor_line_col();
    let visible = app.editor.visible_char_count();
    let lines = app.editor.line_count();
    let total = app.editor.total_char_count();
    let tab_total = app.workspace.tabs.len();
    let tab_current = if tab_total == 0 {
        0
    } else {
        app.workspace.active_index + 1
    };
    let mut segments = vec![
        FooterSegment {
            text: footer_focus_label(app),
            help: "Área que recebe o teclado neste momento",
            click: None,
        },
        FooterSegment {
            text: format!("Aba {tab_current}/{tab_total}"),
            help: "Aba ativa e total de abas abertas",
            click: None,
        },
        FooterSegment {
            text: format!("Tam {visible}/{lines}/{total}"),
            help: "Caracteres visíveis no viewport / linhas do arquivo / total de caracteres (inclui quebras de linha)",
            click: None,
        },
        FooterSegment {
            text: format!("Pos {ln}/{col}"),
            help: "Linha e coluna do cursor (base 1); clique para ir para linha",
            click: Some(FooterClick::GoToLine),
        },
        FooterSegment {
            text: app.editor.mode().label().to_string(),
            help: "Modo de edição atual",
            click: None,
        },
        FooterSegment {
            text: app.document.encoding.label().to_string(),
            help: "Codificação do arquivo; clique para abrir Formatar → Codificação",
            click: Some(FooterClick::OpenFormatEncoding),
        },
        FooterSegment {
            text: app.document.tabulation.footer_label().to_string(),
            help: "Configuração de tabulação",
            click: None,
        },
    ];
    if app.view.show_memory {
        if let Some(label) = app.memory.display_label() {
            segments.push(FooterSegment {
                text: label,
                help: "Consumo de memória do processo",
                click: None,
            });
        }
    }
    segments
}

fn footer_status_from_segments(segments: &[FooterSegment]) -> String {
    segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Grupos de estado alinhados à direita (foco, aba, tamanho, linha/coluna, modo, encoding, tab, memória).
pub fn footer_status_right(app: &App) -> String {
    footer_status_from_segments(&footer_segments(app))
}

fn footer_right_start(left: &str, right: &str, width: usize) -> usize {
    let right_len = right.chars().count();
    let left_len = left.chars().count();
    if width <= right_len {
        return 0;
    }
    if left_len + right_len >= width {
        width.saturating_sub(right_len)
    } else {
        left_len + (width - left_len - right_len)
    }
}

/// Retorna o texto de ajuda do grupo do rodapé sob o cursor, se houver.
pub fn footer_hover_at(app: &App, rel_col: usize, inner_width: usize) -> Option<String> {
    footer_segment_index_at(app, rel_col, inner_width).map(|i| {
        footer_segments(app)[i].help.to_string()
    })
}

/// Ação de clique no grupo do rodapé sob o cursor.
pub fn footer_click_at(app: &App, rel_col: usize, inner_width: usize) -> Option<FooterClick> {
    footer_segment_index_at(app, rel_col, inner_width)
        .and_then(|i| footer_segments(app)[i].click)
}

/// Retorna o índice do segmento (para testes).
pub fn footer_segment_index_at(app: &App, rel_col: usize, inner_width: usize) -> Option<usize> {
    let left = footer_help_default(app);
    let segments = footer_segments(app);
    let right = footer_status_from_segments(&segments);
    let right_start = footer_right_start(&left, &right, inner_width);
    if rel_col < right_start {
        return None;
    }
    let rel = rel_col - right_start;
    let mut pos = 0usize;
    for (i, segment) in segments.iter().enumerate() {
        let len = segment.text.chars().count();
        if rel >= pos && rel < pos + len {
            return Some(i);
        }
        pos += len;
        if i + 1 < segments.len() {
            pos += 3;
        }
    }
    None
}

/// Monta linha do rodapé: ajuda à esquerda, status à direita, com espaço entre eles.
pub fn compose_footer_line(left: &str, right: &str, width: usize) -> String {
    let right_len = right.chars().count();
    let left_len = left.chars().count();

    if width <= right_len {
        return right.chars().take(width).collect();
    }
    if left_len + right_len >= width {
        let left_max = width.saturating_sub(right_len);
        let trimmed: String = left.chars().take(left_max).collect();
        let pad = width.saturating_sub(trimmed.chars().count() + right_len);
        return format!("{trimmed}{}{right}", " ".repeat(pad));
    }
    let pad = width - left_len - right_len;
    format!("{left}{}{right}", " ".repeat(pad))
}

pub fn footer_inner(area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    ratatui::layout::Rect {
        x: area.x.saturating_add(1),
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    }
}

/// Atalhos globais que funcionam mesmo com foco no terminal.
fn handle_global_chords(app: &mut App, key: KeyEvent) -> bool {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    match key.code {
        KeyCode::Char('e' | 'E') => {
            app.focus_editor();
            true
        }
        KeyCode::Char('t' | 'T' | '\'') => {
            app.chord_terminal_toggle();
            true
        }
        KeyCode::Char('g' | 'G') => {
            app.request_go_to_line();
            true
        }
        _ => false,
    }
}

fn handle_global_function_keys(app: &mut App, key: KeyEvent) -> bool {
    use KeyCode;

    if key.modifiers.contains(KeyModifiers::CONTROL)
        || key.modifiers.contains(KeyModifiers::ALT)
    {
        return false;
    }

    match key.code {
        KeyCode::F(1) => {
            app.set_status("Ajuda: em breve");
            true
        }
        KeyCode::F(2) => {
            app.request_rename();
            true
        }
        KeyCode::F(3) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.find_prev();
            } else {
                app.find_next();
            }
            true
        }
        KeyCode::F(4) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.focus_tab_relative(-1);
            } else {
                app.focus_tab_relative(1);
            }
            true
        }
        KeyCode::F(6) => {
            app.toggle_input_focus();
            true
        }
        KeyCode::F(7) => {
            app.send_editor_text_to_terminal();
            true
        }
        KeyCode::F(10) => {
            app.request_save();
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn footer_includes_size_with_line_count() {
        let app = App::new(false);
        let status = footer_status_right(&app);
        assert!(status.contains("Tam "));
        let tam = status.split(" | ").find(|s| s.starts_with("Tam ")).unwrap();
        let parts: Vec<_> = tam.trim_start_matches("Tam ").split('/').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn footer_hover_at_finds_tam_group() {
        let app = App::new(false);
        let segments = footer_segments(&app);
        let right = footer_status_from_segments(&segments);
        let left = footer_help_default(&app);
        let width = left.chars().count() + right.chars().count() + 4;
        let start = footer_right_start(&left, &right, width);
        let tam_offset = right
            .split(" | ")
            .take(2)
            .map(|part| part.chars().count() + 3)
            .sum::<usize>();
        let help = footer_hover_at(&app, start + tam_offset, width);
        assert_eq!(
            help.as_deref(),
            Some("Caracteres visíveis no viewport / linhas do arquivo / total de caracteres (inclui quebras de linha)")
        );
    }

    #[test]
    fn footer_click_at_pos_opens_go_to_line() {
        let app = App::new(false);
        let segments = footer_segments(&app);
        let right = footer_status_from_segments(&segments);
        let left = footer_help_default(&app);
        let width = left.chars().count() + right.chars().count() + 4;
        let start = footer_right_start(&left, &right, width);
        let pos_offset = right
            .split(" | ")
            .take(3)
            .map(|part| part.chars().count() + 3)
            .sum::<usize>();
        let click = footer_click_at(&app, start + pos_offset, width);
        assert_eq!(click, Some(FooterClick::GoToLine));
    }

    #[test]
    fn footer_includes_tab_indicator() {
        let app = App::new(false);
        let status = footer_status_right(&app);
        assert!(status.contains("Aba 1/1"));
    }

    #[test]
    fn footer_includes_memory_when_enabled() {
        let mut app = App::new(false);
        app.view.show_memory = true;
        app.memory.set_cached_for_test(Some(128 * 1024 * 1024));

        let status = footer_status_right(&app);
        assert!(status.contains("Mem 128MB"));
    }

    #[test]
    fn footer_omits_memory_when_disabled() {
        let mut app = App::new(false);
        app.view.show_memory = false;
        app.memory.set_cached_for_test(Some(128 * 1024 * 1024));

        let status = footer_status_right(&app);
        assert!(!status.contains("Mem"));
    }

    #[test]
    fn footer_omits_memory_when_unavailable() {
        let mut app = App::new(false);
        app.view.show_memory = true;
        app.memory.set_cached_for_test(None);

        let status = footer_status_right(&app);
        assert!(!status.contains("Mem"));
    }

    #[test]
    fn footer_focus_label_shows_terminal() {
        let mut app = App::new(false);
        app.view.terminal = true;
        app.input_focus = InputFocus::Terminal;
        let status = footer_status_right(&app);
        assert!(status.starts_with("Terminal |"));
    }

    #[test]
    fn footer_focus_label_shows_editor_by_default() {
        let app = App::new(false);
        let status = footer_status_right(&app);
        assert!(status.starts_with("Editor |"));
    }

    #[test]
    fn footer_help_uses_status_message_without_layer_hint() {
        let mut app = App::new(false);
        app.view.terminal = true;
        app.input_focus = InputFocus::Terminal;
        app.set_status("Pronto");
        let help = footer_help_left(&app);
        assert_eq!(help, "Pronto");
    }
}
