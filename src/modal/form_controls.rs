//! Controles visuais compartilhados entre modais de formulário.

use crossterm::event::MouseEvent;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::theme::ThemePalette;
use crate::widgets::panel;

pub fn paint_label(frame: &mut Frame<'_>, area: Rect, text: &str, palette: ThemePalette) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(text).style(palette.status_style()),
        area,
    );
}

pub fn paint_button(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    focused: bool,
    palette: ThemePalette,
) {
    paint_button_with_mnemonic(frame, area, label, None, focused, palette);
}

/// Botão `[Label]` com letra mnemónica realçada (cor de hotkey do tema).
pub fn paint_button_with_mnemonic(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    mnemonic: Option<char>,
    focused: bool,
    palette: ThemePalette,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    use ratatui::text::{Line, Span};

    let base = palette.button_style(focused);
    let hot = Style::default()
        .fg(palette.menu_hotkey)
        .bg(if focused {
            ratatui::style::Color::White
        } else {
            palette.button_bg
        })
        .add_modifier(Modifier::BOLD);

    let mut spans = vec![Span::styled("[", base)];
    if let Some(m) = mnemonic {
        let target = m.to_ascii_lowercase();
        let mut done = false;
        for ch in label.chars() {
            if !done && ch.to_ascii_lowercase() == target {
                spans.push(Span::styled(ch.to_string(), hot));
                done = true;
            } else {
                spans.push(Span::styled(ch.to_string(), base));
            }
        }
        if !done {
            spans.push(Span::styled(format!("({m})"), hot));
            spans.push(Span::styled(label.to_string(), base));
        }
    } else {
        spans.push(Span::styled(label.to_string(), base));
    }
    spans.push(Span::styled("]", base));

    frame.render_widget(Paragraph::new(Line::from(spans)).style(base), area);
}

pub fn paint_text_input(
    frame: &mut Frame<'_>,
    area: Rect,
    input: &crate::modal::text_input::TextInput,
    focused: bool,
    palette: ThemePalette,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let mut field_style = palette.editor_text_style();
    if focused {
        field_style = field_style.add_modifier(Modifier::BOLD);
    }
    panel::fill_rect(frame, area, field_style);
    let display = if focused {
        input.display_focused()
    } else {
        input.display_unfocused()
    };
    frame.render_widget(Paragraph::new(display).style(field_style), area);
}

pub fn paint_field(
    frame: &mut Frame<'_>,
    area: Rect,
    text: &str,
    focused: bool,
    palette: ThemePalette,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let mut field_style = palette.editor_text_style();
    if focused {
        field_style = field_style.add_modifier(Modifier::BOLD);
    }
    panel::fill_rect(frame, area, field_style);
    let display = if focused {
        format!(" {text}▌")
    } else {
        format!(" {text}")
    };
    frame.render_widget(Paragraph::new(display).style(field_style), area);
}

pub fn button_width(label: &str) -> u16 {
    label.chars().count() as u16 + 2
}

pub fn rect_contains(r: Rect, mouse: &MouseEvent) -> bool {
    mouse.column >= r.x
        && mouse.column < r.x.saturating_add(r.width)
        && mouse.row >= r.y
        && mouse.row < r.y.saturating_add(r.height)
}

pub fn layout_button_row<'a>(
    x: u16,
    y: u16,
    max_width: u16,
    labels: &'a [&'static str],
) -> Vec<(usize, &'static str, Rect)> {
    let mut out = Vec::with_capacity(labels.len());
    let mut cx = x;
    for (i, label) in labels.iter().enumerate() {
        let w = button_width(label);
        if cx.saturating_add(w) > x.saturating_add(max_width) && cx > x {
            break;
        }
        out.push((
            i,
            *label,
            Rect {
                x: cx,
                y,
                width: w,
                height: 1,
            },
        ));
        cx = cx.saturating_add(w).saturating_add(1);
    }
    out
}
