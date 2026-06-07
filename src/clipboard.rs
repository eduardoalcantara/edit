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
        self.ring.insert(0, text);
        if self.ring.len() > RING_SIZE {
            self.ring.truncate(RING_SIZE);
        }
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

    pub fn preview(index: usize, text: &str) -> String {
        let chars: String = text.chars().take(20).collect();
        if text.chars().count() > 20 {
            format!("{}. {}", index + 1, chars)
        } else {
            format!("{}. {}", index + 1, chars)
        }
    }
}
