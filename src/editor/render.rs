use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::editor::line_numbers;
use crate::editor::wrap;
use crate::edit_mode::EditMode;
use crate::editor::cursor::{char_idx_to_line_col, SelectionMode};
use crate::editor::engine::EditorEngine;
use crate::editor::tabs::{expand_tabs, tab_stop_width, visual_col_in_line};
use crate::theme::ThemePalette;
use crate::view_state::{EditorBorder, EditorMargin};
use crate::widgets::panel;

pub fn inner_area(outer: Rect, border: EditorBorder, terminal_block: Option<u16>) -> Rect {
    panel::editor_content_rect(outer, border == EditorBorder::Visible, terminal_block)
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

pub fn editor_viewport_rect(
    shell: Rect,
    border: EditorBorder,
    terminal_block: Option<u16>,
    margin: EditorMargin,
) -> Rect {
    let inner = panel::editor_content_rect(
        shell,
        border == EditorBorder::Visible,
        terminal_block,
    );
    text_area(inner, margin)
}

pub fn draw(
    engine: &mut EditorEngine,
    frame: &mut Frame,
    area: Rect,
    title: &str,
    palette: ThemePalette,
    margin: EditorMargin,
    border: EditorBorder,
    terminal_block: Option<u16>,
    text_viewport: Option<Rect>,
    show_cursor: bool,
    show_tabs: bool,
    show_line_numbers: bool,
) -> (Rect, Rect) {
    let border_style = Style::default()
        .fg(palette.border)
        .bg(palette.editor_bg);
    let title_style = Style::default()
        .fg(palette.editor_fg)
        .bg(palette.editor_bg)
        .add_modifier(Modifier::BOLD);

    let inner = panel::render_editor_frame(
        frame,
        area,
        title,
        palette.editor_text_style(),
        border_style,
        title_style,
        border == EditorBorder::Visible,
        terminal_block,
    );
    let content = text_viewport.unwrap_or_else(|| text_area(inner, margin));

    panel::fill_rect(frame, inner, palette.editor_text_style());

    let line_count = engine.text.len_lines().max(1);
    let (cursor_line, _) = char_idx_to_line_col(&engine.text, engine.primary().char_idx);
    let gutter_layout = if show_line_numbers {
        Some(line_numbers::layout(line_count, margin))
    } else {
        None
    };
    let (gutter_area, text_rect) = if let Some(gutter) = gutter_layout {
        line_numbers::split_text_area(content, gutter)
    } else {
        (Rect::default(), content)
    };

    engine.viewport.update_size(text_rect);
    engine.ensure_visible();

    let visible_h = text_rect.height as usize;
    let visible_w = text_rect.width as usize;
    let word_wrap = engine.word_wrap && visible_w > 0;
    let top_visual = if word_wrap {
        engine.viewport.top_visual_row
    } else {
        engine.viewport.top_line
    };
    let left = if word_wrap { 0 } else { engine.viewport.left_col };

    if let Some(gutter) = gutter_layout {
        line_numbers::paint_gutter(
            frame,
            gutter_area,
            gutter,
            engine,
            top_visual,
            visible_h,
            visible_w,
            show_tabs,
            cursor_line,
            palette,
        );
    }

    let top = engine.viewport.top_line;
    let text_style = palette.editor_text_style();
    let tab_width = tab_stop_width(engine.tabulation);
    for row in 0..visible_h {
        let line_area = Rect {
            x: text_rect.x,
            y: text_rect.y.saturating_add(row as u16),
            width: text_rect.width,
            height: 1,
        };
        if word_wrap {
            let Some(display_row) =
                wrap::build_display_row_at(engine, top_visual + row, visible_w, show_tabs)
            else {
                frame.render_widget(Paragraph::new(" ").style(text_style), line_area);
                continue;
            };
            let expanded = wrap::expanded_line(engine, display_row.doc_line, show_tabs);
            let display =
                wrap::segment_text(&expanded, display_row, visible_w, left);
            let mut line_str = engine.text.line(display_row.doc_line).to_string();
            line_str.truncate(line_str.trim_end_matches('\n').len());
            let spans = styled_line(
                engine,
                display_row.doc_line,
                &line_str,
                display_row.seg_start + left,
                &display,
                tab_width,
                palette,
            );
            frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
        } else {
            let doc_line = top + row;
            if doc_line >= line_count {
                frame.render_widget(Paragraph::new(" ").style(text_style), line_area);
                continue;
            }
            let mut line_str = engine.text.line(doc_line).to_string();
            line_str.truncate(line_str.trim_end_matches('\n').len());
            let expanded = expand_tabs(&line_str, tab_width, show_tabs);
            let display = if left < expanded.chars().count() {
                expanded.chars().skip(left).take(visible_w).collect::<String>()
            } else {
                String::new()
            };
            let spans = styled_line(
                engine,
                doc_line,
                &line_str,
                left,
                &display,
                tab_width,
                palette,
            );
            frame.render_widget(Paragraph::new(Line::from(spans)), line_area);
        }
    }

    draw_cursors(
        frame,
        engine,
        text_rect,
        top_visual,
        left,
        tab_width,
        palette,
        show_cursor,
        show_tabs,
        word_wrap,
    );
    engine.refresh_footer_size_stats(top_visual, visible_h, visible_w, show_tabs);
    (text_rect, content)
}

fn styled_line(
    engine: &EditorEngine,
    doc_line: usize,
    line_str: &str,
    left_vis: usize,
    display: &str,
    tab_width: usize,
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
                spans = highlight_range(
                    display,
                    left_vis,
                    visual_col_in_line(line_str, sel_start, tab_width),
                    visual_col_in_line(line_str, sel_end, tab_width),
                    normal,
                    selected,
                );
            }
        }
    }

    if engine.selection_mode == SelectionMode::Block {
        if let Some(block) = engine.block_selection {
            let (r0, vc0, r1, vc1) = block.normalized();
            if doc_line >= r0 && doc_line <= r1 {
                spans = highlight_block_range(
                    display,
                    left_vis,
                    vc0,
                    vc1,
                    normal,
                    selected,
                );
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

fn highlight_block_range(
    display: &str,
    left_vis: usize,
    vc0: usize,
    vc1: usize,
    normal: Style,
    selected: Style,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = display.chars().collect();
    let vis_start = vc0.saturating_sub(left_vis);
    let vis_end = vc1.saturating_sub(left_vis);
    if vis_end <= vis_start {
        return vec![Span::styled(display.to_string(), normal)];
    }
    let before: String = chars[..vis_start.min(chars.len())].iter().collect();
    let mid: String = if vis_start < chars.len() {
        chars[vis_start..vis_end.min(chars.len())].iter().collect()
    } else {
        String::new()
    };
    let pad_count = vis_end.saturating_sub(chars.len().max(vis_start));
    let after: String = if vis_end < chars.len() {
        chars[vis_end..].iter().collect()
    } else {
        String::new()
    };
    let mut spans = vec![Span::styled(before, normal)];
    if !mid.is_empty() || pad_count > 0 {
        spans.push(Span::styled(
            format!("{}{}", mid, " ".repeat(pad_count)),
            selected,
        ));
    }
    spans.push(Span::styled(after, normal));
    spans
}

fn highlight_range(
    display: &str,
    left_vis: usize,
    sel_start_vis: usize,
    sel_end_vis: usize,
    normal: Style,
    selected: Style,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = display.chars().collect();
    let vis_start = sel_start_vis.saturating_sub(left_vis);
    let vis_end = sel_end_vis.saturating_sub(left_vis).min(chars.len());
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
    top_visual: usize,
    left_vis: usize,
    tab_width: usize,
    palette: ThemePalette,
    show_cursor: bool,
    show_tabs: bool,
    word_wrap: bool,
) {
    if !show_cursor {
        return;
    }

    let cursors: Vec<_> = if engine.selection_mode == SelectionMode::Multi {
        engine.cursors.clone()
    } else {
        vec![*engine.primary()]
    };

    let w = content.width as usize;

    for (i, cursor) in cursors.iter().enumerate() {
        let (line, col) = char_idx_to_line_col(&engine.text, cursor.char_idx);
        let mut line_str = engine.text.line(line).to_string();
        line_str.truncate(line_str.trim_end_matches('\n').len());
        let vis_col = visual_col_in_line(&line_str, col, tab_width);

        let (screen_row, x_offset) = if word_wrap && w > 0 {
            let vrow = wrap::visual_row_for_cursor(engine, line, vis_col, w, show_tabs);
            if vrow < top_visual || vrow >= top_visual + content.height as usize {
                continue;
            }
            let display_row =
                match wrap::build_display_row_at(engine, vrow, w, show_tabs) {
                    Some(r) => r,
                    None => continue,
                };
            let row = vrow - top_visual;
            let x = vis_col.saturating_sub(display_row.seg_start).saturating_sub(left_vis);
            (row, x)
        } else {
            let top_line = top_visual;
            if line < top_line || line >= top_line + content.height as usize {
                continue;
            }
            let row = line - top_line;
            let x = vis_col.saturating_sub(left_vis);
            (row, x)
        };

        if x_offset >= w {
            continue;
        }
        let style = if i == 0 {
            palette.cursor_style_for_mode(engine.input_mode)
        } else {
            palette.cursor_style()
        };
        let x = content.x.saturating_add(x_offset as u16);
        let y = content.y.saturating_add(screen_row as u16);
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
