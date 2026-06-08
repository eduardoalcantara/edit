//! Quebra visual de linhas (word wrap) para renderização e numeração.

use crate::editor::engine::EditorEngine;
use crate::editor::tabs::{expand_tabs, tab_stop_width};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayRow {
    pub doc_line: usize,
    /// Índice do segmento na linha lógica (0 = exibe nº de linha).
    pub seg_index: usize,
    /// Início do segmento em colunas visuais da linha expandida.
    pub seg_start: usize,
}

/// Quebra uma linha expandida em segmentos de no máximo `width` colunas.
pub fn segment_starts(expanded: &str, width: usize, word_wrap: bool) -> Vec<usize> {
    if width == 0 {
        return vec![0];
    }
    let len = expanded.chars().count();
    if !word_wrap || len <= width {
        return vec![0];
    }
    let chars: Vec<char> = expanded.chars().collect();
    let mut starts = vec![0usize];
    let mut i = 0usize;
    while i < len {
        let mut end = (i + width).min(len);
        if end >= len {
            break;
        }
        let mut break_at = end;
        for j in (i + 1..end).rev() {
            if chars[j] == ' ' {
                break_at = j + 1;
                break;
            }
        }
        if break_at <= i {
            break_at = end;
        }
        starts.push(break_at);
        i = break_at;
    }
    starts
}

pub fn expanded_line(engine: &EditorEngine, doc_line: usize, show_tabs: bool) -> String {
    let mut line_str = engine.text.line(doc_line).to_string();
    line_str.truncate(line_str.trim_end_matches('\n').len());
    expand_tabs(&line_str, tab_stop_width(engine.tabulation), show_tabs)
}

pub fn visual_rows_for_line(
    engine: &EditorEngine,
    doc_line: usize,
    width: usize,
    show_tabs: bool,
) -> Vec<DisplayRow> {
    let expanded = expanded_line(engine, doc_line, show_tabs);
    segment_starts(&expanded, width, engine.word_wrap)
        .into_iter()
        .enumerate()
        .map(|(seg_index, seg_start)| DisplayRow {
            doc_line,
            seg_index,
            seg_start,
        })
        .collect()
}

pub fn total_visual_rows(
    engine: &EditorEngine,
    width: usize,
    show_tabs: bool,
) -> usize {
    let line_count = engine.text.len_lines().max(1);
    if !engine.word_wrap || width == 0 {
        return line_count;
    }
    (0..line_count)
        .map(|line| visual_rows_for_line(engine, line, width, show_tabs).len())
        .sum()
}

pub fn build_display_row_at(
    engine: &EditorEngine,
    visual_index: usize,
    width: usize,
    show_tabs: bool,
) -> Option<DisplayRow> {
    if width == 0 && engine.word_wrap {
        return None;
    }
    if !engine.word_wrap {
        let line_count = engine.text.len_lines().max(1);
        return if visual_index < line_count {
            Some(DisplayRow {
                doc_line: visual_index,
                seg_index: 0,
                seg_start: 0,
            })
        } else {
            None
        };
    }
    let line_count = engine.text.len_lines().max(1);
    let mut remaining = visual_index;
    for doc_line in 0..line_count {
        let segs = visual_rows_for_line(engine, doc_line, width, show_tabs);
        if remaining < segs.len() {
            return Some(segs[remaining]);
        }
        remaining -= segs.len();
    }
    None
}

pub fn visual_row_for_cursor(
    engine: &EditorEngine,
    doc_line: usize,
    vis_col: usize,
    width: usize,
    show_tabs: bool,
) -> usize {
    if !engine.word_wrap || width == 0 {
        return doc_line;
    }
    let mut base = 0usize;
    for line in 0..doc_line {
        base += visual_rows_for_line(engine, line, width, show_tabs).len();
    }
    let segs = segment_starts(&expanded_line(engine, doc_line, show_tabs), width, true);
    let seg_index = segs
        .iter()
        .rposition(|&start| start <= vis_col)
        .unwrap_or(0);
    base + seg_index
}

pub fn segment_text(expanded: &str, row: DisplayRow, width: usize, left_col: usize) -> String {
    let chars: Vec<char> = expanded.chars().collect();
    let seg_start = row.seg_start;
    let seg_end = if row.seg_index + 1 < segment_starts(expanded, width, true).len() {
        segment_starts(expanded, width, true)[row.seg_index + 1]
    } else {
        chars.len()
    };
    let slice: String = chars[seg_start..seg_end].iter().collect();
    if left_col < slice.chars().count() {
        slice.chars().skip(left_col).take(width).collect()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::engine::EditorEngine;

    #[test]
    fn wrap_splits_long_line() {
        let starts = segment_starts("hello world foo", 8, true);
        assert_eq!(starts, vec![0, 6, 12]);
    }

    #[test]
    fn no_wrap_single_segment() {
        let starts = segment_starts("short", 80, false);
        assert_eq!(starts, vec![0]);
    }

    #[test]
    fn visual_row_for_cursor_on_wrapped_line() {
        let mut e = EditorEngine::new();
        e.word_wrap = true;
        e.load_text("abcdefghijklmnop");
        let row = visual_row_for_cursor(&e, 0, 10, 8, false);
        assert_eq!(row, 1);
    }
}
