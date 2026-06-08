use ropey::Rope;

pub fn find_next(text: &Rope, pattern: &str, from_char: usize) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    let start = from_char.min(content.len());
    if let Some(pos) = content[start..].find(pattern) {
        return Some(content[..start + pos].chars().count());
    }
    content.find(pattern).map(|p| content[..p].chars().count())
}

pub fn find_prev(text: &Rope, pattern: &str, from_char: usize) -> Option<usize> {
    if pattern.is_empty() {
        return None;
    }
    let content = text.to_string();
    let char_count = content.chars().count();
    let from = from_char.min(char_count);
    let byte_pos = content
        .char_indices()
        .nth(from.saturating_sub(1))
        .map(|(i, _)| i)
        .unwrap_or(0);
    if let Some(pos) = content[..=byte_pos].rfind(pattern) {
        return Some(content[..pos].chars().count());
    }
    content.rfind(pattern).map(|p| content[..p].chars().count())
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

pub fn replace_at(text: &mut Rope, char_idx: usize, pattern: &str, replacement: &str) -> bool {
    let Some((start_char, end_char)) = match_range_for_replace(text, char_idx, pattern) else {
        return false;
    };
    text.remove(start_char..end_char);
    text.insert(start_char, replacement);
    true
}
