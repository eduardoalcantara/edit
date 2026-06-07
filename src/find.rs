use crate::modal::Modal;

#[derive(Debug, Clone, Default)]
pub struct FindState {
    pub pattern: String,
    pub replace_with: String,
}

pub fn find_next(lines: &[String], pattern: &str, row: usize, col: usize) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let total = lines.len().max(1);
    for offset in 0..total {
        let r = (row + offset) % total;
        let line = lines.get(r).map(String::as_str).unwrap_or("");
        let start_col = if offset == 0 { col + 1 } else { 0 };
        if let Some(pos) = line[start_col..].find(pattern) {
            return Some((r, start_col + pos));
        }
        if let Some(pos) = line.find(pattern) {
            return Some((r, pos));
        }
    }
    None
}

pub fn find_prev(lines: &[String], pattern: &str, row: usize, col: usize) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let total = lines.len().max(1);
    for offset in 0..total {
        let r = (row + total - offset) % total;
        let line = lines.get(r).map(String::as_str).unwrap_or("");
        let end_col = if offset == 0 {
            col.saturating_sub(1)
        } else {
            line.len().saturating_sub(1)
        };
        let slice_end = (end_col + pattern.len()).min(line.len());
        if slice_end > 0 {
            if let Some(pos) = line[..slice_end].rfind(pattern) {
                return Some((r, pos));
            }
        }
    }
    None
}

pub fn replace_one(lines: &mut [String], row: usize, col: usize, pattern: &str, replacement: &str) -> bool {
    let Some(line) = lines.get_mut(row) else {
        return false;
    };
    if let Some(pos) = line[col..].find(pattern) {
        let at = col + pos;
        line.replace_range(at..at + pattern.len(), replacement);
        true
    } else if let Some(pos) = line.find(pattern) {
        line.replace_range(pos..pos + pattern.len(), replacement);
        true
    } else {
        false
    }
}

pub fn open_find_modal() -> Modal {
    Modal::find("Buscar", "")
}

pub fn open_replace_modal() -> Modal {
    Modal::find_replace("Substituir", "", "")
}
