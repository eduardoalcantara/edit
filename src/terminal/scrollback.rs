//! Buffer de linhas do output do shell.

const DEFAULT_MAX_LINES: usize = 10_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum EscapeState {
    #[default]
    Normal,
    Esc,
    Csi,
    Osc,
    OscEsc,
}

pub struct Scrollback {
    lines: Vec<String>,
    partial: String,
    rewrite_from_start: bool,
    max_lines: usize,
    escape: EscapeState,
}

impl Scrollback {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            partial: String::new(),
            rewrite_from_start: false,
            max_lines: DEFAULT_MAX_LINES,
            escape: EscapeState::Normal,
        }
    }

    pub fn push_bytes(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);
        for ch in text.chars() {
            self.push_char(ch);
        }
    }

    fn push_char(&mut self, ch: char) {
        match self.escape {
            EscapeState::Normal => match ch {
                '\r' => {
                    self.rewrite_from_start = true;
                }
                '\n' => {
                    let line = strip_inline_ansi(std::mem::take(&mut self.partial));
                    self.rewrite_from_start = false;
                    self.push_line(line);
                }
                '\x08' | '\x7f' => {
                    self.partial.pop();
                }
                '\x1b' => {
                    self.escape = EscapeState::Esc;
                }
                c if c.is_control() && c != '\t' => {}
                c => {
                    if self.rewrite_from_start {
                        self.partial.clear();
                        self.rewrite_from_start = false;
                    }
                    self.partial.push(c);
                }
            },
            EscapeState::Esc => match ch {
                '[' => self.escape = EscapeState::Csi,
                ']' => self.escape = EscapeState::Osc,
                _ => self.escape = EscapeState::Normal,
            },
            EscapeState::Csi => {
                if ('@'..='~').contains(&ch) {
                    self.escape = EscapeState::Normal;
                }
            }
            EscapeState::Osc => match ch {
                '\x07' => self.escape = EscapeState::Normal,
                '\x1b' => self.escape = EscapeState::OscEsc,
                _ => {}
            },
            EscapeState::OscEsc => {
                self.escape = if ch == '\\' {
                    EscapeState::Normal
                } else {
                    EscapeState::Osc
                };
            }
        }
    }

    fn push_line(&mut self, line: String) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
        }
        self.lines.push(line);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len() + usize::from(!self.partial.is_empty())
    }

    pub fn committed_line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn committed_line(&self, index: usize) -> &str {
        self.lines[index].as_str()
    }

    pub fn partial_line(&self) -> String {
        strip_inline_ansi(self.partial.clone())
    }

    /// Linhas lógicas para exibição: une prompt commitado + input parcial (cmd.exe na 1ª linha).
    pub fn logical_lines(&self) -> Vec<String> {
        let partial = self.partial_line();
        let mut all = self.lines.clone();
        if partial.is_empty() {
            return all;
        }
        if let Some(last) = all.last() {
            if should_merge_prompt_with_input(last, &partial) {
                let merged = all.pop().expect("last exists");
                all.push(format!("{merged}{partial}"));
                return all;
            }
        }
        all.push(partial);
        all
    }

    /// Linhas visíveis do fim. Com `follow_tail`, ancora no prompt/input corrente.
    pub fn visible_tail(
        &self,
        height: usize,
        scroll_offset: usize,
        follow_tail: bool,
    ) -> Vec<String> {
        if height == 0 {
            return vec![];
        }

        let all = self.logical_lines();

        if scroll_offset == 0 && follow_tail {
            if all.is_empty() {
                return vec![String::new(); height];
            }
            let start = all.len().saturating_sub(height);
            let mut out: Vec<String> = all[start..].to_vec();
            while out.len() < height {
                out.insert(0, String::new());
            }
            if out.len() > height {
                out.drain(0..out.len() - height);
            }
            return out;
        }

        if all.is_empty() {
            return vec![String::new(); height];
        }
        let total = all.len();
        let end = total.saturating_sub(scroll_offset);
        let start = end.saturating_sub(height);
        let mut out: Vec<String> = all[start..end].to_vec();
        while out.len() < height {
            out.insert(0, String::new());
        }
        out
    }

    pub fn max_scroll_offset(&self, visible_height: usize) -> usize {
        self.line_count().saturating_sub(visible_height)
    }

    /// Linhas visíveis com índice global (linhas commitadas + linha parcial no fim).
    pub fn visible_tail_indexed(
        &self,
        height: usize,
        scroll_offset: usize,
        follow_tail: bool,
    ) -> Vec<(usize, String)> {
        self.visible_tail(height, scroll_offset, follow_tail)
            .into_iter()
            .enumerate()
            .map(|(row, text)| {
                let global = self.global_line_for_visible_row(
                    height,
                    scroll_offset,
                    follow_tail,
                    row,
                );
                (global, text)
            })
            .collect()
    }

    fn global_line_for_visible_row(
        &self,
        height: usize,
        scroll_offset: usize,
        follow_tail: bool,
        row: usize,
    ) -> usize {
        let all = self.logical_lines();
        if scroll_offset == 0 && follow_tail {
            let start = all.len().saturating_sub(height);
            return start + row;
        }
        let end = all.len().saturating_sub(scroll_offset);
        let start = end.saturating_sub(height);
        start + row
    }
}

fn should_merge_prompt_with_input(last_line: &str, partial: &str) -> bool {
    if partial.is_empty() {
        return false;
    }
    last_line.ends_with('>')
        || last_line.ends_with("$ ")
        || last_line.ends_with("% ")
}

fn strip_inline_ansi(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() == Some('[') {
                for pc in chars.by_ref() {
                    if ('@'..='~').contains(&pc) {
                        break;
                    }
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                while let Some(pc) = chars.next() {
                    if pc == '\x07' {
                        break;
                    }
                    if pc == '\x1b' && chars.next() == Some('\\') {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out.trim_end_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_committed_prompt_with_partial_input() {
        let mut sb = Scrollback::new();
        sb.push_bytes(b"banner\n");
        sb.push_bytes(b"D:\\proj\\edit>\n");
        sb.push_bytes(b"dir");
        let vis = sb.visible_tail(2, 0, true);
        assert_eq!(vis.last().map(String::as_str), Some("D:\\proj\\edit>dir"));
    }

    #[test]
    fn push_line_on_newline() {
        let mut sb = Scrollback::new();
        sb.push_bytes(b"hello\nworld");
        assert_eq!(sb.lines.len(), 1);
        assert_eq!(sb.partial, "world");
    }

    #[test]
    fn scroll_offset_shows_older_lines() {
        let mut sb = Scrollback::new();
        for i in 0..5 {
            sb.push_bytes(format!("line{i}\n").as_bytes());
        }
        let vis = sb.visible_tail(2, 1, false);
        assert_eq!(vis[1], "line3");
    }

    #[test]
    fn follow_tail_pins_prompt_on_last_row() {
        let mut sb = Scrollback::new();
        sb.push_bytes(b"banner1\nbanner2\nbanner3\nbanner4\n");
        sb.push_bytes(b"C:\\Users>");
        let vis = sb.visible_tail(4, 0, true);
        assert_eq!(vis[3], "C:\\Users>");
    }

    #[test]
    fn scroll_to_tail_offset_zero_shows_prompt() {
        let mut sb = Scrollback::new();
        for i in 0..12 {
            sb.push_bytes(format!("line{i}\n").as_bytes());
        }
        sb.push_bytes(b"D:\\proj>");
        let vis = sb.visible_tail(6, 0, true);
        assert_eq!(vis.last().map(String::as_str), Some("D:\\proj>"));
        let max = sb.max_scroll_offset(6);
        let vis_old = sb.visible_tail(6, max, false);
        assert_eq!(vis_old.first().map(String::as_str), Some("line0"));
    }

    #[test]
    fn scroll_delta_up_from_tail_increases_offset() {
        let mut sb = Scrollback::new();
        for i in 0..12 {
            sb.push_bytes(format!("line{i}\n").as_bytes());
        }
        let visible = 6usize;
        let max = sb.max_scroll_offset(visible);
        let offset = 0usize;
        let delta = 3isize;
        let next = ((offset as isize) + delta).clamp(0, max as isize) as usize;
        assert_eq!(next, 3);
        let vis = sb.visible_tail(visible, next, false);
        assert_eq!(vis[0], "line3");
    }

    #[test]
    fn osc_title_sequence_is_stripped() {
        let mut sb = Scrollback::new();
        sb.push_bytes(b"\x1b]0;Administrador: cmd\x07C:\\proj\\edit>");
        assert!(sb.partial.contains("C:\\proj\\edit>"));
        assert!(!sb.partial.contains("]0;"));
        assert!(!sb.partial.contains("Administrador"));
    }

    #[test]
    fn crlf_commits_line() {
        let mut sb = Scrollback::new();
        sb.push_bytes(b"hello\r\n");
        assert_eq!(sb.lines, vec!["hello".to_string()]);
        assert!(sb.partial.is_empty());
    }
}
