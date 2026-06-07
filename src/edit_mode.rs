#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Insert,
    Replace,
}

impl EditMode {
    pub fn label(self) -> &'static str {
        match self {
            EditMode::Insert => "Insert",
            EditMode::Replace => "Replace",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            EditMode::Insert => EditMode::Replace,
            EditMode::Replace => EditMode::Insert,
        }
    }
}
