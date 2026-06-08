//! Coluna de numeração de linhas no editor.

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::editor::engine::EditorEngine;
use crate::editor::wrap;
use crate::theme::ThemePalette;
use crate::view_state::EditorMargin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineGutterLayout {
    pub digits_width: usize,
    pub gap: usize,
    pub total_width: usize,
}

pub fn gutter_gap(margin: EditorMargin) -> usize {
    match margin {
        EditorMargin::None => 1,
        EditorMargin::OneLine => 2,
        EditorMargin::TwoLines => 4,
    }
}

pub fn digits_width(line_count: usize) -> usize {
    line_count.max(1).to_string().len()
}

pub fn layout(line_count: usize, margin: EditorMargin) -> LineGutterLayout {
    let digits_width = digits_width(line_count);
    let gap = gutter_gap(margin);
    LineGutterLayout {
        digits_width,
        gap,
        total_width: digits_width + gap,
    }
}

pub fn format_line_number(line_index: usize, digits_width: usize) -> String {
    format!("{:>width$}", line_index + 1, width = digits_width)
}

pub fn split_text_area(content: Rect, gutter: LineGutterLayout) -> (Rect, Rect) {
    let gutter_w = gutter.total_width.min(content.width as usize) as u16;
    let gutter_area = Rect {
        x: content.x,
        y: content.y,
        width: gutter_w,
        height: content.height,
    };
    let text_area = Rect {
        x: content.x.saturating_add(gutter_w),
        y: content.y,
        width: content.width.saturating_sub(gutter_w),
        height: content.height,
    };
    (gutter_area, text_area)
}

pub fn paint_gutter(
    frame: &mut Frame,
    gutter_area: Rect,
    layout: LineGutterLayout,
    engine: &EditorEngine,
    top_visual: usize,
    visible_h: usize,
    text_width: usize,
    show_tabs: bool,
    cursor_line: usize,
    palette: ThemePalette,
) {
    let dim = palette.line_number_style();
    let active = palette.line_number_active_style();
    let blank = " ".repeat(layout.total_width);
    for row in 0..visible_h {
        let y = gutter_area.y.saturating_add(row as u16);
        let cell = Rect {
            x: gutter_area.x,
            y,
            width: gutter_area.width,
            height: 1,
        };
        let Some(display_row) = wrap::build_display_row_at(engine, top_visual + row, text_width, show_tabs)
        else {
            frame.render_widget(Paragraph::new(blank.clone()).style(dim), cell);
            continue;
        };
        if display_row.seg_index != 0 {
            frame.render_widget(Paragraph::new(blank.clone()).style(dim), cell);
            continue;
        }
        let number = format_line_number(display_row.doc_line, layout.digits_width);
        let mut line = number;
        if layout.gap > 0 {
            line.push_str(&" ".repeat(layout.gap));
        }
        let style = if display_row.doc_line == cursor_line {
            active
        } else {
            dim
        };
        frame.render_widget(Paragraph::new(line).style(style), cell);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_width_grows_with_line_count() {
        assert_eq!(digits_width(1), 1);
        assert_eq!(digits_width(9), 1);
        assert_eq!(digits_width(10), 2);
        assert_eq!(digits_width(100), 3);
    }

    #[test]
    fn gutter_gap_follows_margin() {
        assert_eq!(gutter_gap(EditorMargin::None), 1);
        assert_eq!(gutter_gap(EditorMargin::OneLine), 2);
        assert_eq!(gutter_gap(EditorMargin::TwoLines), 4);
    }

    #[test]
    fn layout_aligns_hundreds() {
        let g = layout(100, EditorMargin::None);
        assert_eq!(g.digits_width, 3);
        assert_eq!(g.total_width, 4);
        assert_eq!(format_line_number(0, g.digits_width), "  1");
        assert_eq!(format_line_number(99, g.digits_width), "100");
    }
}
