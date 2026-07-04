#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCommand {
    InsertChar(char),
    Backspace,
    Delete,
    MoveLeft { extend: bool },
    MoveRight { extend: bool },
    MoveWordLeft { extend: bool },
    MoveWordRight { extend: bool },
    MoveUp { extend: bool },
    MoveDown { extend: bool },
    Home { extend: bool },
    End { extend: bool },
    DocumentStart { extend: bool },
    DocumentEnd { extend: bool },
    PageUp,
    PageDown,
    /// Roda do mouse: negativo = subir, positivo = descer (em linhas).
    ScrollWheel { delta: i32 },
    SelectAll,
    CancelSelection,
    Undo,
    Redo,
    Tab,
    Paste(String),
    StartBlockSelect { line: usize, col: usize },
    UpdateBlockSelect { line: usize, col: usize },
    EndBlockSelect,
    AddCursor { line: usize, col: usize },
    SetCursor { line: usize, col: usize },
    Click { line: usize, col: usize },
    ExtendSelection { line: usize, col: usize },
}

impl EditorCommand {
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            EditorCommand::InsertChar(_)
                | EditorCommand::Backspace
                | EditorCommand::Delete
                | EditorCommand::Undo
                | EditorCommand::Redo
                | EditorCommand::Tab
                | EditorCommand::Paste(_)
        )
    }
}
