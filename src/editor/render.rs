use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::edit_mode::EditMode;
use crate::editor::cursor::{char_idx_to_line_col, SelectionMode};
use crate::editor::engine::EditorEngine;
use crate::theme::ThemePalette;
use crate::view_state::{EditorBorder, EditorMargin};
use crate::widgets::panel;

pub fn inner_area(outer: Rect, border: EditorBorder, terminal_below: bool) -> Rect {
    panel::editor_content_rect(outer, border == EditorBorder::Visible, terminal_below)
}

/// Área útil para texto após aplicar margens internas.
pub fn text_area(inner: Rect, margin: EditorMargin) -> Rect {
    let (top, bottom, left, right) = margin.insets();
    let top = top.min(inner.height as usize);
    let bottom = bottom.min(inner.height.saturating_sub(top as u16) as usize);
    let left = left.min(inner.width as usize);
    let right = right.min(inner.width.saturating_sub(left as u16) as usize);
    Rect {
        x: inner.x.saturating_add(left as u16),
        y: inner.y.saturating_add(top as u16),
        width: inner.width.saturating_sub((left + right) as u16),
        height: inner.height.saturating_sub((top + bottom) as u16),
    }
}

pub fn draw(
    engine: &mut EditorEngine,
    frame: &mut Frame,
    area: Rect,
    title: &str,
    palette: ThemePalette,
    margin: EditorMargin,
    border: EditorBorder,
    terminal_below: bool,
    show_cursor: bool,
) -> Rect {
    let frame_title = format!("[ {title} ]");
    let border_style = Style::default()
        .fg(palette.border)
        .bg(palette.editor_bg);

    let inner = panel::render_editor_frame(
        frame,
        area,
        &frame_title,
        palette.editor_text_style(),
        border_style,
        border == EditorBorder::Visible,
        terminal_below,
    );
    let content = text_area(inner, margin);

    engine.viewport.update_size(content);
    engine.ensure_visible();

    panel::fill_rect(frame, inner, palette.editor_text_style());

    let top = engine.viewport.top_line;
    let left = engine.viewport.left_col;
    let visible_h = content.height as usize;
    let visible_w = content.width as usize;
    let line_count = engine.text.len_lines().max(1);

    let text_style = palette.editor_text_style();
    for row in 0..visible_h {
        let doc_line = top + row;
        let line_area = Rect {
            x: content.x,
            y: content.y.saturating_add(row as u16),
            width: content.width,
            height: 1,
        };
        if doc_line >= line_count {
            frame.render_widget(Paragraph::new(" ").style(text_style), line_area);
            continue;
        }
        let line_text = engine.text.line(doc_line);
        let line_str = line_text.to_string();
        let display = if left < line_str.chars().count() {
            line_str.chars().skip(left).take(visible_w).collect::<String>()
        } else {
            String::new()
        };

        let spans = styled_line(engine, doc_line, left, &display, palette);
        frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
    }

    draw_cursors(frame, engine, content, top, left, palette, show_cursor);
    content
}

fn styled_line(
    engine: &EditorEngine,
    doc_line: usize,
    left_col: usize,
    display: &str,
    palette: ThemePalette,
) -> Vec<Span<'static>> {
    let normal = palette.editor_text_style();
    let selected = palette.selection_style();
    let mut spans = vec![Span::styled(display.to_string(), normal)];

    if engine.selection_mode == SelectionMode::Normal {
        if let Some(anchor) = engine.primary().anchor {
            let (a, b) = if anchor <= engine.primary().char_idx {
                (anchor, engine.primary().char_idx)
            } else {
                (engine.primary().char_idx, anchor)
            };
            let (r0, c0) = char_idx_to_line_col(&engine.text, a);
            let (r1, c1) = char_idx_to_line_col(&engine.text, b);
            if doc_line >= r0 && doc_line <= r1 {
                let line_len = engine.text.line(doc_line).len_chars();
                let sel_start = if doc_line == r0 { c0 } else { 0 };
                let sel_end = if doc_line == r1 { c1 } else { line_len };
                spans = highlight_range(display, left_col, sel_start, sel_end, normal, selected);
            }
        }
    }

    if engine.selection_mode == SelectionMode::Block {
        if let Some(block) = engine.block_selection {
            let (r0, c0, r1, c1) = block.normalized();
            if doc_line >= r0 && doc_line <= r1 {
                spans = highlight_range(display, left_col, c0, c1, normal, selected);
            }
        }
    }

    if !engine.search_pattern.is_empty() {
        let content = display.to_string();
        if let Some(pos) = content.find(&engine.search_pattern) {
            let before = &content[..pos];
            let mid = &content[pos..pos + engine.search_pattern.len()];
            let after = &content[pos + engine.search_pattern.len()..];
            let match_style = Style::default()
                .fg(palette.status)
                .bg(palette.editor_bg)
                .add_modifier(Modifier::BOLD);
            spans = vec![
                Span::styled(before.to_string(), normal),
                Span::styled(mid.to_string(), match_style),
                Span::styled(after.to_string(), normal),
            ];
        }
    }

    spans
}

fn highlight_range(
    display: &str,
    left_col: usize,
    sel_start: usize,
    sel_end: usize,
    normal: Style,
    selected: Style,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = display.chars().collect();
    let vis_start = sel_start.saturating_sub(left_col);
    let vis_end = sel_end.saturating_sub(left_col).min(chars.len());
    if vis_start >= chars.len() || vis_end <= vis_start {
        return vec![Span::styled(display.to_string(), normal)];
    }
    let before: String = chars[..vis_start].iter().collect();
    let mid: String = chars[vis_start..vis_end].iter().collect();
    let after: String = chars[vis_end..].iter().collect();
    vec![
        Span::styled(before, normal),
        Span::styled(mid, selected),
        Span::styled(after, normal),
    ]
}

fn draw_cursors(
    frame: &mut Frame,
    engine: &EditorEngine,
    content: Rect,
    top_line: usize,
    left_col: usize,
    palette: ThemePalette,
    show_cursor: bool,
) {
    if !show_cursor {
        return;
    }

    let cursors: Vec<_> = if engine.selection_mode == SelectionMode::Multi {
        engine.cursors.clone()
    } else {
        vec![*engine.primary()]
    };

    for (i, cursor) in cursors.iter().enumerate() {
        let (line, col) = char_idx_to_line_col(&engine.text, cursor.char_idx);
        if line < top_line || line >= top_line + content.height as usize {
            continue;
        }
        let vis_col = col.saturating_sub(left_col);
        if vis_col >= content.width as usize {
            continue;
        }
        let style = if i == 0 {
            palette.cursor_style_for_mode(engine.input_mode)
        } else {
            palette.cursor_style()
        };
        let x = content.x.saturating_add(vis_col as u16);
        let y = content.y.saturating_add((line - top_line) as u16);
        if engine.input_mode == EditMode::Replace && i == 0 {
            if let Some(ch) = engine.text.get_char(cursor.char_idx) {
                frame.render_widget(
                    Paragraph::new(ch.to_string()).style(style),
                    Rect {
                        x,
                        y,
                        width: 1,
                        height: 1,
                    },
                );
                continue;
            }
        }
        frame.set_cursor_position((x, y));
        let _ = style;
    }
}
