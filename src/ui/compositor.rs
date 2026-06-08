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

/// Texto de ajuda à esquerda (menu/modal/ação recente).
pub fn footer_help_left(app: &App) -> String {
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

pub fn footer_focus_group(app: &App) -> String {
    let active = footer_active_focus(app);
    let items = ["Editor", "Terminal", "Menu", "Diálogo"];
    let parts: Vec<String> = items
        .iter()
        .map(|item| {
            if *item == active {
                format!("[{item}]")
            } else {
                (*item).to_string()
            }
        })
        .collect();
    format!("Foco {}", parts.join(" "))
}

/// Grupos de estado alinhados à direita (tamanho, linha/coluna, aba, modo, encoding, tab, memória).
pub fn footer_status_right(app: &App) -> String {
    let (ln, col) = app.editor.cursor_line_col();
    let visible = app.editor.visible_char_count();
    let total = app.editor.total_char_count();
    let tab_total = app.workspace.tabs.len();
    let tab_current = if tab_total == 0 {
        0
    } else {
        app.workspace.active_index + 1
    };
    let mut segments = vec![
        format!("Aba {tab_current}/{tab_total}"),
        format!("Tam {visible}/{total}"),
        format!("Pos {ln}/{col}"),
        app.editor.mode().label().to_string(),
        app.document.encoding.label().to_string(),
        app.document.tabulation.footer_label().to_string(),
    ];
    segments.push(footer_focus_group(app));
    if app.view.show_memory {
        if let Some(label) = app.memory.display_label() {
            segments.push(label);
        }
    }
    segments.join(" | ")
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
        KeyCode::Char('t' | 'T' | '\'') => {
            app.toggle_terminal_panel();
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
    fn footer_focus_group_highlights_terminal() {
        let mut app = App::new(false);
        app.view.terminal = true;
        app.input_focus = InputFocus::Terminal;
        let status = footer_status_right(&app);
        assert!(status.contains("Foco"));
        assert!(status.contains("[Terminal]"));
    }

    #[test]
    fn footer_focus_group_highlights_editor_by_default() {
        let app = App::new(false);
        let status = footer_status_right(&app);
        assert!(status.contains("[Editor]"));
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
