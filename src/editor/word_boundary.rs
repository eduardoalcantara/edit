//! Navegação inteligente por palavra (Ctrl+setas).
//!
//! Regras de segmentação, em ordem de prioridade:
//! 1. Separadores simbólicos e espaço em branco delimitam unidades.
//! 2. Transição minúscula → MAIÚSCULA inicia nova unidade (`camel` | `Case`).
//! 3. Bloco contíguo de maiúsculas seguido de minúscula: todas menos a última
//!    maiúscula formam uma unidade; a última maiúscula inicia a próxima com as
//!    minúsculas seguintes (`HTTP` | `Server`, `HTTP` | `Response`).
//! 4. Dígitos contíguos formam unidade própria (`version` | `2` | `Final`).

use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordDirection {
    Left,
    Right,
}

/// Retorna o índice de caractere da fronteira de palavra a partir de `position`.
///
/// - `WordDirection::Right` — início da próxima unidade à direita (pula separadores).
/// - `WordDirection::Left` — início da unidade atual ou, se já estiver no início
///   dela, início da unidade anterior.
pub fn get_next_word_boundary(text: &Rope, position: usize, direction: WordDirection) -> usize {
    let flat = text.to_string();
    let len = flat.chars().count();
    let position = position.min(len);
    match direction {
        WordDirection::Right => next_word_boundary(&flat, position),
        WordDirection::Left => prev_word_boundary(&flat, position),
    }
}

/// Índices de caractere onde cada unidade de palavra começa (ignora separadores).
pub fn word_starts(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut starts = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_separator(chars[i]) {
            i += 1;
            continue;
        }
        starts.push(i);
        i = end_of_unit(&chars, i);
    }
    starts
}

fn next_word_boundary(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    if pos >= n {
        return n;
    }

    if is_separator(chars[pos]) {
        let mut p = pos;
        while p < n && is_separator(chars[p]) {
            p += 1;
        }
        return p;
    }

    let unit_start = word_start_at_or_before(&chars, pos);
    let unit_end = end_of_unit(&chars, unit_start);
    let mut p = if pos < unit_end { unit_end } else { pos };
    while p < n && is_separator(chars[p]) {
        p += 1;
    }
    p
}

fn prev_word_boundary(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    if pos == 0 {
        return 0;
    }

    let n = chars.len();
    let pos = pos.min(n);

    if pos < n && is_separator(chars[pos]) {
        let mut p = pos;
        while p > 0 && is_separator(chars[p - 1]) {
            p -= 1;
        }
        return prev_word_boundary(text, p);
    }

    let cur_start = word_start_at_or_before(&chars, pos);
    if pos > cur_start {
        return cur_start;
    }

    if cur_start == 0 {
        return 0;
    }

    prev_word_start_before(&chars, cur_start)
}

fn prev_word_start_before(chars: &[char], before: usize) -> usize {
    if before == 0 {
        return 0;
    }
    let starts = word_starts_from_chars(chars);
    for start in starts.iter().rev() {
        if *start < before {
            return *start;
        }
    }
    0
}

fn word_start_at_or_before(chars: &[char], pos: usize) -> usize {
    if chars.is_empty() || pos == 0 {
        return 0;
    }

    let pos = pos.min(chars.len());
    let starts = word_starts_from_chars(chars);

    if pos < chars.len() && !is_separator(chars[pos]) && starts.contains(&pos) {
        return pos;
    }

    let probe = pos.saturating_sub(1);
    let mut i = probe;
    while i > 0 && is_separator(chars[i]) {
        i -= 1;
    }
    if is_separator(chars[i]) {
        return pos;
    }

    for start in starts.iter().rev() {
        if *start <= i {
            return *start;
        }
    }
    0
}

fn word_starts_from_chars(chars: &[char]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_separator(chars[i]) {
            i += 1;
            continue;
        }
        starts.push(i);
        i = end_of_unit(chars, i);
    }
    starts
}

/// Fim exclusivo da unidade que começa em `start`.
fn end_of_unit(chars: &[char], start: usize) -> usize {
    let n = chars.len();
    if start >= n {
        return start;
    }

    let ch = chars[start];
    if is_digit(ch) {
        let mut i = start + 1;
        while i < n && is_digit(chars[i]) {
            i += 1;
        }
        return i;
    }

    if is_lower(ch) {
        let mut i = start + 1;
        while i < n {
            if is_separator(chars[i]) || is_digit(chars[i]) {
                break;
            }
            if is_upper(chars[i]) {
                break;
            }
            if is_lower(chars[i]) {
                i += 1;
                continue;
            }
            break;
        }
        return i;
    }

    if is_upper(ch) {
        let mut j = start + 1;
        while j < n && is_upper(chars[j]) {
            j += 1;
        }
        if j < n && is_lower(chars[j]) {
            if j - start > 1 {
                return j - 1;
            }
            return end_of_lower_tail(chars, j);
        }
        return j;
    }

    start + 1
}

fn end_of_lower_tail(chars: &[char], start: usize) -> usize {
    let n = chars.len();
    let mut i = start + 1;
    while i < n {
        if is_separator(chars[i]) || is_digit(chars[i]) {
            break;
        }
        if is_upper(chars[i]) {
            break;
        }
        if is_lower(chars[i]) {
            i += 1;
            continue;
        }
        break;
    }
    i
}

fn is_separator(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '_' | '-'
                | '.'
                | '/'
                | '\\'
                | ':'
                | '@'
                | '#'
                | '$'
                | '%'
                | '&'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '<'
                | '>'
                | '='
                | '+'
                | '*'
                | '?'
                | '!'
                | ';'
                | ','
                | '"'
                | '\''
                | '`'
        )
}

fn is_lower(ch: char) -> bool {
    ch.is_lowercase()
}

fn is_upper(ch: char) -> bool {
    ch.is_uppercase()
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn right_steps(text: &str, steps: usize) -> Vec<usize> {
        let mut pos = 0;
        let mut positions = vec![pos];
        for _ in 0..steps {
            pos = next_word_boundary(text, pos);
            positions.push(pos);
        }
        positions
    }

    fn left_steps(text: &str, start: usize, steps: usize) -> Vec<usize> {
        let mut pos = start;
        let mut positions = vec![pos];
        for _ in 0..steps {
            pos = prev_word_boundary(text, pos);
            positions.push(pos);
        }
        positions
    }

    #[test]
    fn word_starts_parse_http_response() {
        assert_eq!(word_starts("parseHTTPResponse"), vec![0, 5, 9]);
    }

    #[test]
    fn word_starts_user_profile_id() {
        assert_eq!(word_starts("user_profile_id"), vec![0, 5, 13]);
    }

    #[test]
    fn word_starts_easy_edit_turbo() {
        assert_eq!(word_starts("easy-edit-turbo"), vec![0, 5, 10]);
    }

    #[test]
    fn word_starts_camel_case_word() {
        assert_eq!(word_starts("camelCaseWord"), vec![0, 5, 9]);
    }

    #[test]
    fn word_starts_version2_final() {
        assert_eq!(word_starts("version2Final"), vec![0, 7, 8]);
    }

    #[test]
    fn ctrl_right_parse_http_response() {
        assert_eq!(right_steps("parseHTTPResponse", 3), vec![0, 5, 9, 17]);
    }

    #[test]
    fn ctrl_right_user_profile_id() {
        assert_eq!(right_steps("user_profile_id", 3), vec![0, 5, 13, 15]);
    }

    #[test]
    fn ctrl_right_easy_edit_turbo() {
        assert_eq!(right_steps("easy-edit-turbo", 3), vec![0, 5, 10, 15]);
    }

    #[test]
    fn ctrl_right_camel_case_word() {
        assert_eq!(right_steps("camelCaseWord", 3), vec![0, 5, 9, 13]);
    }

    #[test]
    fn ctrl_right_version2_final() {
        assert_eq!(right_steps("version2Final", 3), vec![0, 7, 8, 13]);
    }

    #[test]
    fn ctrl_left_from_end_parse_http_response() {
        let len = "parseHTTPResponse".chars().count();
        assert_eq!(left_steps("parseHTTPResponse", len, 3), vec![17, 9, 5, 0]);
    }

    #[test]
    fn ctrl_left_from_middle_of_word() {
        assert_eq!(prev_word_boundary("parseHTTPResponse", 7), 5);
        assert_eq!(prev_word_boundary("parseHTTPResponse", 3), 0);
    }

    #[test]
    fn ctrl_left_user_profile_id() {
        assert_eq!(left_steps("user_profile_id", 15, 3), vec![15, 13, 5, 0]);
    }

    #[test]
    fn rope_wrapper_matches_str() {
        let rope = Rope::from_str("parseHTTPResponse");
        assert_eq!(
            get_next_word_boundary(&rope, 0, WordDirection::Right),
            5
        );
        assert_eq!(
            get_next_word_boundary(&rope, 5, WordDirection::Left),
            0
        );
    }
}
