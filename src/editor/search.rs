use ropey::Rope;

use crate::editor::tabs::visual_col_in_line;

pub fn find_next(text: &Rope, pattern: &str, from_char: usize) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    let byte_start = content
        .char_indices()
        .nth(from_char)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    if let Some(pos) = content[byte_start..].find(pattern) {
        let abs = byte_start + pos;
        return Some(content[..abs].chars().count());
    }
    content
        .find(pattern)
        .map(|p| content[..p].chars().count())
}

pub fn find_first(text: &Rope, pattern: &str) -> Option<usize> {
    find_next(text, pattern, 0)
}

pub fn find_last(text: &Rope, pattern: &str) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    content
        .rfind(pattern)
        .map(|p| content[..p].chars().count())
}

pub fn find_prev(text: &Rope, pattern: &str, from_char: usize) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    let char_count = content.chars().count();
    let from = from_char.min(char_count);
    let byte_end = content
        .char_indices()
        .nth(from)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    let search_end = byte_end.saturating_sub(1);
    if let Some(pos) = content[..=search_end].rfind(pattern) {
        return Some(content[..pos].chars().count());
    }
    content
        .rfind(pattern)
        .map(|p| content[..p].chars().count())
}

pub fn all_match_starts(text: &Rope, pattern: &str) -> Vec<usize> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let step = pattern.chars().count().max(1);
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut from = 0;
    loop {
        let Some(idx) = find_next(text, pattern, from) else {
            break;
        };
        if !seen.insert(idx) {
            break;
        }
        out.push(idx);
        from = idx.saturating_add(step);
        if from >= text.len_chars() {
            break;
        }
    }
    out
}

pub fn next_match_after_cursor(
    matches: &[usize],
    pattern_chars: usize,
    cursor: usize,
) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    for (i, &start) in matches.iter().enumerate() {
        let end = start.saturating_add(pattern_chars);
        if cursor >= start && cursor < end {
            return Some(matches[(i + 1) % matches.len()]);
        }
    }
    for &start in matches {
        if start >= cursor {
            return Some(start);
        }
    }
    Some(matches[0])
}

pub fn prev_match_before_cursor(
    matches: &[usize],
    pattern_chars: usize,
    cursor: usize,
) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }
    for i in (0..matches.len()).rev() {
        let start = matches[i];
        let end = start.saturating_add(pattern_chars);
        if cursor > start && cursor <= end {
            return Some(if i == 0 {
                *matches.last().unwrap_or(&start)
            } else {
                matches[i - 1]
            });
        }
    }
    for i in (0..matches.len()).rev() {
        if matches[i] < cursor {
            return Some(matches[i]);
        }
    }
    matches.last().copied()
}

/// Intervalos de coluna visual (na linha expandida) para cada ocorrência na linha.
pub fn line_match_visual_ranges(
    text: &Rope,
    pattern: &str,
    match_starts: &[usize],
    doc_line: usize,
    current_match: Option<usize>,
    line_str: &str,
    tab_width: usize,
) -> Vec<(usize, usize, bool)> {
    if pattern.is_empty() || doc_line >= text.len_lines() {
        return Vec::new();
    }
    let pattern_chars = pattern.chars().count();
    let line_start = text.line_to_char(doc_line);
    let line_chars = text.line(doc_line).len_chars();
    let line_end = line_start.saturating_add(line_chars);
    let mut out = Vec::new();
    for &start in match_starts {
        let end = start.saturating_add(pattern_chars);
        if end <= line_start || start >= line_end {
            continue;
        }
        let local_start = start.saturating_sub(line_start);
        let local_end = end.min(line_end) - line_start;
        let vis_start = visual_col_in_line(line_str, local_start, tab_width);
        let vis_end = visual_col_in_line(line_str, local_end, tab_width);
        let is_current = current_match == Some(start);
        out.push((vis_start, vis_end, is_current));
    }
    out
}

pub fn match_range_for_replace(
    text: &Rope,
    char_idx: usize,
    pattern: &str,
) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    let byte_start = content
        .char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    if let Some(rel) = content[byte_start..].find(pattern) {
        let start = byte_start + rel;
        let end = start + pattern.len();
        let start_char = content[..start].chars().count();
        let end_char = content[..end].chars().count();
        return Some((start_char, end_char));
    }
    if let Some(pos) = content.find(pattern) {
        let end = pos + pattern.len();
        let start_char = content[..pos].chars().count();
        let end_char = content[..end].chars().count();
        return Some((start_char, end_char));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rope(s: &str) -> Rope {
        Rope::from_str(s)
    }

    #[test]
    fn next_match_from_before_occurrence() {
        let text = rope("aaa foo bar foo");
        let matches = all_match_starts(&text, "foo");
        assert_eq!(matches, vec![4, 12]);
        assert_eq!(next_match_after_cursor(&matches, 3, 0), Some(4));
        assert_eq!(next_match_after_cursor(&matches, 3, 4), Some(12));
        assert_eq!(next_match_after_cursor(&matches, 3, 12), Some(4));
    }

    #[test]
    fn prev_match_from_inside_occurrence() {
        let text = rope("aaa foo bar foo");
        let matches = all_match_starts(&text, "foo");
        assert_eq!(prev_match_before_cursor(&matches, 3, 6), Some(12));
        assert_eq!(prev_match_before_cursor(&matches, 3, 13), Some(4));
        assert_eq!(prev_match_before_cursor(&matches, 3, 4), Some(12));
    }

    #[test]
    fn find_next_uses_char_index_not_byte() {
        let text = rope("áb foo");
        assert_eq!(find_next(&text, "foo", 2), Some(3));
    }
}
