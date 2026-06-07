use crossterm::event::{KeyCode, KeyEvent};

use crate::encoding::Tabulation;
use crate::modal::buttons::CONVERT_TABULATION;
use crate::modal::dialog::{Dialog, DialogKeyResult};

pub const TAB_OPTIONS: [Tabulation; 4] = [
    Tabulation::Spaces2,
    Tabulation::Spaces4,
    Tabulation::Spaces8,
    Tabulation::TabLiteral,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertTabField {
    From,
    To,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertTabKeyResult {
    Consumed,
    Submit,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ConvertTabulationModal {
    pub dialog: Dialog,
    pub from_idx: usize,
    pub to_idx: usize,
    pub field_focus: Option<ConvertTabField>,
}

impl ConvertTabulationModal {
    pub fn new(current: Tabulation) -> Self {
        let idx = tab_index(current);
        Self {
            dialog: Dialog::form("Converter Tabulação", String::new(), &CONVERT_TABULATION),
            from_idx: idx,
            to_idx: idx,
            field_focus: Some(ConvertTabField::From),
        }
    }

    pub fn from_tab(&self) -> Tabulation {
        TAB_OPTIONS[self.from_idx]
    }

    pub fn to_tab(&self) -> Tabulation {
        TAB_OPTIONS[self.to_idx]
    }

    pub fn refresh_body(&mut self) {
        self.dialog.body = format_body(self.from_idx, self.to_idx, self.field_focus);
    }

    pub fn focused_help(&self) -> Option<&'static str> {
        if self.field_focus.is_some() {
            Some(match self.field_focus? {
                ConvertTabField::From => {
                    "Formato atual do arquivo (como tabs/indentação foram gravados)"
                }
                ConvertTabField::To => {
                    "Formato desejado; passa a ser a opção em Formatar → Tabulação"
                }
            })
        } else {
            self.dialog.focused_help()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ConvertTabKeyResult {
        if key.code == KeyCode::Esc {
            return ConvertTabKeyResult::Cancel;
        }

        if let Some(field) = self.field_focus {
            match key.code {
                KeyCode::Up | KeyCode::Down => {
                    self.field_focus = Some(match field {
                        ConvertTabField::From => ConvertTabField::To,
                        ConvertTabField::To => ConvertTabField::From,
                    });
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Left => {
                    self.step(field, -1);
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Right => {
                    self.step(field, 1);
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Tab => {
                    self.field_focus = None;
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Enter => {
                    self.field_focus = None;
                    ConvertTabKeyResult::Consumed
                }
                _ => ConvertTabKeyResult::Consumed,
            }
        } else {
            match self.dialog.handle_button_keys(key) {
                DialogKeyResult::Activate(_) => ConvertTabKeyResult::Submit,
                DialogKeyResult::Cancel => ConvertTabKeyResult::Cancel,
                DialogKeyResult::Consumed => ConvertTabKeyResult::Consumed,
                DialogKeyResult::Ignored => ConvertTabKeyResult::Consumed,
            }
        }
    }

    fn step(&mut self, field: ConvertTabField, delta: i32) {
        let idx = match field {
            ConvertTabField::From => &mut self.from_idx,
            ConvertTabField::To => &mut self.to_idx,
        };
        let len = TAB_OPTIONS.len() as i32;
        let next = (*idx as i32 + delta).rem_euclid(len);
        *idx = next as usize;
    }
}

fn tab_index(tab: Tabulation) -> usize {
    TAB_OPTIONS
        .iter()
        .position(|&t| t == tab)
        .unwrap_or(1)
}

fn format_body(from_idx: usize, to_idx: usize, focus: Option<ConvertTabField>) -> String {
    let from_mark = if focus == Some(ConvertTabField::From) {
        "►"
    } else {
        " "
    };
    let to_mark = if focus == Some(ConvertTabField::To) {
        "►"
    } else {
        " "
    };
    format!(
        "Informe como o arquivo está e como deve ficar.\n\
         A parada define tabs e indentação (ex.: YAML, Java, texto).\n\n\
         {from_mark} De:    {}\n\
         {to_mark} Para:  {}\n\n\
         ↑↓ linha   ←→ opção   Tab botões",
        TAB_OPTIONS[from_idx].convert_option_label(),
        TAB_OPTIONS[to_idx].convert_option_label(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn cycles_from_option_with_arrows() {
        let mut modal = ConvertTabulationModal::new(Tabulation::Spaces4);
        assert_eq!(modal.from_tab(), Tabulation::Spaces4);
        modal.handle_key(key(KeyCode::Right));
        assert_eq!(modal.from_tab(), Tabulation::Spaces8);
    }

    #[test]
    fn switches_field_with_up_down() {
        let mut modal = ConvertTabulationModal::new(Tabulation::Spaces2);
        modal.handle_key(key(KeyCode::Down));
        assert_eq!(modal.field_focus, Some(ConvertTabField::To));
    }
}
