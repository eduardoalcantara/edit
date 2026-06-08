use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use ratatui::Frame;

use crate::app::App;

use super::layer::{InputResult, UiLayer};
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

        let mut layers: Vec<&dyn UiLayer> = all_layers()
            .into_iter()
            .filter(|layer| layer.is_visible(app))
            .collect();
        layers.sort_by_key(|layer| std::cmp::Reverse(layer.id().z()));

        let input_modal = layers.iter().any(|layer| layer.captures_input(app));
        for layer in layers {
            if input_modal && !layer.captures_input(app) {
                continue;
            }
            if layer.on_key(key, app, layout) == InputResult::Consumed {
                return;
            }
        }
        if input_modal {
            return;
        }
    }

    fn dispatch_mouse(app: &mut App, mouse: MouseEvent, layout: UiLayout) {
        let mut layers: Vec<&dyn UiLayer> = all_layers()
            .into_iter()
            .filter(|layer| layer.is_visible(app))
            .collect();
        layers.sort_by_key(|layer| std::cmp::Reverse(layer.id().z()));

        let input_modal = layers.iter().any(|layer| layer.captures_input(app));
        for layer in layers {
            if input_modal && !layer.captures_input(app) {
                continue;
            }
            if layer.on_mouse(mouse, app, layout) == InputResult::Consumed {
                return;
            }
        }
        if input_modal {
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
}
