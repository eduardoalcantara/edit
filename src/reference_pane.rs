//! Painel direito de referência (ajuda, tabela ASCII) — conteúdo virtual read-only.

use crate::edit_mode::EditMode;
use crate::editor::Editor;
use crate::modal::help_content;
use crate::theme::ThemePalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    HelpFeatures,
    HelpShortcuts,
    AsciiTable,
}

impl ReferenceKind {
    pub fn title(self) -> &'static str {
        match self {
            ReferenceKind::HelpFeatures => "Funcionalidades",
            ReferenceKind::HelpShortcuts => "Atalhos",
            ReferenceKind::AsciiTable => "Tabela ASCII",
        }
    }

    pub fn content(self) -> &'static str {
        match self {
            ReferenceKind::HelpFeatures => help_content::features_text(),
            ReferenceKind::HelpShortcuts => help_content::shortcuts_text(),
            ReferenceKind::AsciiTable => help_content::ascii_table_text(),
        }
    }
}

pub struct ReferencePane {
    pub kind: ReferenceKind,
    pub editor: Editor,
    /// Aba real do painel direito antes de abrir a referência.
    pub stashed_right_tab: Option<usize>,
}

impl std::fmt::Debug for ReferencePane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferencePane")
            .field("kind", &self.kind)
            .field("stashed_right_tab", &self.stashed_right_tab)
            .finish_non_exhaustive()
    }
}

pub fn new_reference_editor(kind: ReferenceKind, palette: &ThemePalette) -> Editor {
    let mut editor = Editor::new(palette);
    editor.set_read_only(true);
    editor.engine_mut().load_text(kind.content());
    editor.set_mode(EditMode::Insert, palette);
    editor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeId;

    #[test]
    fn reference_editor_is_read_only() {
        let palette = ThemeId::Dark.palette();
        let editor = new_reference_editor(ReferenceKind::AsciiTable, &palette);
        assert!(editor.is_read_only());
        assert!(editor.content_string().contains("ASCII"));
    }
}
