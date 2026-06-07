const RING_SIZE: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    ring: Vec<String>,
}

impl Clipboard {
    pub fn push(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        self.ring.retain(|t| t != &text);
        self.ring.insert(0, text.clone());
        if self.ring.len() > RING_SIZE {
            self.ring.truncate(RING_SIZE);
        }
        Self::set_system(&text);
    }

    pub fn latest(&self) -> Option<&str> {
        self.ring.first().map(String::as_str)
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.ring.get(index).map(String::as_str)
    }

    pub fn entries(&self) -> &[String] {
        &self.ring
    }

    /// Texto para colar: prioriza área de transferência do SO, depois ring interno.
    pub fn paste_text(&self) -> Option<String> {
        Self::get_system().or_else(|| self.latest().map(str::to_string))
    }

    pub fn set_system(text: &str) {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text.to_string());
        }
    }

    pub fn get_system() -> Option<String> {
        arboard::Clipboard::new().ok()?.get_text().ok()
    }

    pub fn preview(index: usize, text: &str) -> String {
        let chars: String = text.chars().take(20).collect();
        if text.chars().count() > 20 {
            format!("{}. {}", index + 1, chars)
        } else {
            format!("{}. {}", index + 1, chars)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_keeps_latest_first() {
        let mut clip = Clipboard::default();
        clip.push("a".to_string());
        clip.push("b".to_string());
        assert_eq!(clip.latest(), Some("b"));
        assert_eq!(clip.get(1), Some("a"));
    }
}
