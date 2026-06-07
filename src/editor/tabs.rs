use crate::encoding::Tabulation;

pub const LITERAL_TAB_WIDTH: usize = 8;

pub fn tab_stop_width(tabulation: Tabulation) -> usize {
    match tabulation {
        Tabulation::Spaces2 => 2,
        Tabulation::Spaces4 => 4,
        Tabulation::Spaces8 => 8,
        Tabulation::TabLiteral => LITERAL_TAB_WIDTH,
    }
}

pub fn visual_col_in_line(line: &str, char_col: usize, tab_width: usize) -> usize {
    let mut vis = 0;
    for (i, ch) in line.chars().enumerate() {
        if i >= char_col {
            break;
        }
        vis += tab_advance(vis, ch, tab_width);
    }
    vis
}

pub fn line_visual_len(line: &str, tab_width: usize) -> usize {
    visual_col_in_line(line, line.chars().count(), tab_width)
}

pub fn char_col_from_visual(line: &str, target_vis: usize, tab_width: usize) -> usize {
    let mut vis = 0;
    let mut char_col = 0;
    for ch in line.chars() {
        if vis >= target_vis {
            return char_col;
        }
        let advance = tab_advance(vis, ch, tab_width);
        if target_vis < vis + advance {
            return char_col;
        }
        vis += advance;
        char_col += 1;
    }
    char_col
}

/// Expande `\t` para a largura visual usada na pintura e no posicionamento do cursor.
pub fn expand_tabs(line: &str, tab_width: usize, show_tabs: bool) -> String {
    if !line.contains('\t') {
        return line.to_string();
    }
    let mut out = String::with_capacity(line.len());
    let mut col = 0;
    for ch in line.chars() {
        if ch == '\t' {
            let advance = tab_width - (col % tab_width);
            if show_tabs {
                out.push('»');
                for _ in 1..advance {
                    out.push(' ');
                }
            } else {
                for _ in 0..advance {
                    out.push(' ');
                }
            }
            col += advance;
        } else {
            out.push(ch);
            col += 1;
        }
    }
    out
}

fn tab_advance(visual_col: usize, ch: char, tab_width: usize) -> usize {
    if ch == '\t' {
        tab_width - (visual_col % tab_width)
    } else {
        1
    }
}

/// Converte tabulação usando paradas explícitas de origem e destino.
pub fn convert_tabulation_between(content: &str, from: Tabulation, to: Tabulation) -> String {
    if from == to {
        return content.to_string();
    }
    let from_w = tab_stop_width(from);
    let to_w = tab_stop_width(to);
    map_lines(content, |line| convert_line_between(line, from_w, to_w, to))
}

fn convert_line_between(line: &str, from_w: usize, to_w: usize, to: Tabulation) -> String {
    match to {
        Tabulation::TabLiteral => {
            let expanded = expand_tabs(line, from_w, false);
            leading_indent_to_tabs(&expanded, from_w)
        }
        _ => expand_tabs(line, to_w, false),
    }
}

/// Expande cada `\t` do documento em espaços até a parada `width` (2, 4 ou 8).
pub fn convert_tabs_to_spaces(content: &str, width: usize) -> String {
    map_lines(content, |line| expand_tabs(line, width, false))
}

/// Recolhe a indentação inicial (espaços e tabs) em `\t` na parada `width` (2, 4 ou 8).
pub fn convert_spaces_to_tabs(content: &str, width: usize) -> String {
    map_lines(content, |line| leading_indent_to_tabs(line, width))
}

fn map_lines(content: &str, mut convert: impl FnMut(&str) -> String) -> String {
    let mut result = String::new();
    let mut lines = content.split('\n').peekable();
    while let Some(line) = lines.next() {
        result.push_str(&convert(line));
        if lines.peek().is_some() || content.ends_with('\n') {
            result.push('\n');
        }
    }
    result
}

fn split_leading_indent(line: &str) -> (&str, &str) {
    let end = line
        .char_indices()
        .take_while(|(_, ch)| matches!(ch, ' ' | '\t'))
        .map(|(i, ch)| i + ch.len_utf8())
        .last()
        .unwrap_or(0);
    line.split_at(end)
}

fn leading_indent_to_tabs(line: &str, width: usize) -> String {
    let (indent, rest) = split_leading_indent(line);
    if indent.is_empty() {
        return line.to_string();
    }
    let expanded = expand_tabs(indent, width, false);
    format!("{}{}", collapse_spaces_to_tabs(&expanded, width), rest)
}

fn collapse_spaces_to_tabs(spaces: &str, tab_width: usize) -> String {
    let mut out = String::new();
    let mut col = 0;
    let mut i = 0;
    let chars: Vec<char> = spaces.chars().collect();
    while i < chars.len() {
        if chars[i] != ' ' {
            out.push(chars[i]);
            col += 1;
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i] == ' ' {
            i += 1;
        }
        let mut remaining = i - start;
        while remaining > 0 {
            let to_stop = tab_width - (col % tab_width);
            if remaining >= to_stop {
                out.push('\t');
                col += to_stop;
                remaining -= to_stop;
            } else {
                for _ in 0..remaining {
                    out.push(' ');
                }
                col += remaining;
                remaining = 0;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_col_after_literal_tab() {
        assert_eq!(visual_col_in_line("testesdd\t", 9, 8), 16);
        assert_eq!(visual_col_in_line("testesdd\t", 8, 8), 8);
    }

    #[test]
    fn expand_tabs_fills_to_stop() {
        assert_eq!(expand_tabs("ab\tc", 8, false), "ab      c");
    }

    #[test]
    fn char_col_from_visual_roundtrip() {
        let line = "testesdd\t";
        let char_col = 9;
        let vis = visual_col_in_line(line, char_col, 8);
        assert_eq!(char_col_from_visual(line, vis, 8), char_col);
    }

    #[test]
    fn tabs_to_spaces_at_four() {
        assert_eq!(convert_tabs_to_spaces("a\tb", 4), "a   b");
    }

    #[test]
    fn spaces_to_tabs_at_four() {
        assert_eq!(convert_spaces_to_tabs("    4 espaços", 4), "\t4 espaços");
    }

    #[test]
    fn spaces_to_tabs_at_two() {
        assert_eq!(convert_spaces_to_tabs("  x", 2), "\tx");
    }

    #[test]
    fn tabs_to_spaces_leaves_plain_spaces() {
        assert_eq!(convert_tabs_to_spaces("    ok", 4), "    ok");
    }

    #[test]
    fn spaces_to_tabs_leaves_inline_tabs() {
        assert_eq!(convert_spaces_to_tabs("a\tb", 4), "a\tb");
    }

    #[test]
    fn convert_between_space_modes_expands_tabs() {
        use crate::encoding::Tabulation;
        assert_eq!(
            convert_tabulation_between("a\tb", Tabulation::TabLiteral, Tabulation::Spaces4),
            "a   b"
        );
    }

    #[test]
    fn convert_between_to_literal_tabs() {
        use crate::encoding::Tabulation;
        assert_eq!(
            convert_tabulation_between("    x", Tabulation::Spaces4, Tabulation::TabLiteral),
            "\tx"
        );
    }

    #[test]
    fn roundtrip_spaces_at_four() {
        let original = "Sem tabs\n    4 espaços\nok?";
        let tabs = convert_spaces_to_tabs(original, 4);
        assert_eq!(tabs, "Sem tabs\n\t4 espaços\nok?");
        let back = convert_tabs_to_spaces(&tabs, 4);
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_via_between() {
        use crate::encoding::Tabulation;
        let original = "Sem tabs\n    4 espaços\nok?";
        let tabs = convert_tabulation_between(original, Tabulation::Spaces4, Tabulation::TabLiteral);
        let back = convert_tabulation_between(&tabs, Tabulation::TabLiteral, Tabulation::Spaces4);
        assert_eq!(back, original);
    }
}
