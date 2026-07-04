use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::encoding::Tabulation;
use crate::modal::buttons::CONVERT_TABULATION;
use crate::modal::dialog::{
    centered_dialog_rect, dialog_button_row_y, dialog_content_rect, hit_dialog_button,
    paint_dialog_buttons, paint_titled_dialog_content, Dialog, DialogKeyResult,
};
use crate::theme::ThemePalette;
use crate::widgets::panel;

pub const TAB_OPTIONS: [Tabulation; 4] = [
    Tabulation::Spaces2,
    Tabulation::Spaces4,
    Tabulation::Spaces8,
    Tabulation::TabLiteral,
];

const DIALOG_WIDTH: u16 = 54;
const LIST_ROWS: u16 = 4;
const LIST_GAP: u16 = 2;
/// Altura de cada caixa (borda + 4 opções + borda).
const LIST_BOX_HEIGHT: u16 = LIST_ROWS + 2;
const CONTENT_HEIGHT: u16 = 10;

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

#[derive(Debug, Clone, Copy)]
struct ConvertTabLayout {
    from_list: Rect,
    to_list: Rect,
    button_y: u16,
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

    pub fn refresh_body(&mut self) {}

    pub fn outer_rect(&self, area: Rect) -> Rect {
        let inner_h = CONTENT_HEIGHT.saturating_add(panel::DIALOG_MARGIN.0 + panel::DIALOG_MARGIN.1);
        centered_dialog_rect(area, DIALOG_WIDTH, inner_h.saturating_add(2))
    }

    pub fn paint(&self, frame: &mut Frame<'_>, area: Rect, palette: ThemePalette) {
        panel::render_drop_shadow(frame, area, palette);
        let content = paint_titled_dialog_content(
            frame,
            area,
            &self.dialog.title,
            palette,
        );
        let layout = self.layout(content);

        let intro = Paragraph::new(vec![
            Line::from("Informe como o arquivo está e como deve ficar."),
            Line::from(""),
        ])
        .style(palette.menu_panel_style());
        frame.render_widget(
            intro,
            Rect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: 2,
            },
        );

        draw_option_list(
            frame,
            layout.from_list,
            "De",
            self.from_idx,
            self.field_focus == Some(ConvertTabField::From),
            palette,
        );
        draw_option_list(
            frame,
            layout.to_list,
            "Para",
            self.to_idx,
            self.field_focus == Some(ConvertTabField::To),
            palette,
        );

        let blank_y = layout.from_list.y + LIST_BOX_HEIGHT;
        frame.render_widget(
            Paragraph::new("").style(palette.menu_panel_style()),
            Rect {
                x: content.x,
                y: blank_y,
                width: content.width,
                height: 1,
            },
        );

        paint_dialog_buttons(
            frame,
            content,
            layout.button_y,
            self.dialog.selected,
            self.dialog.buttons,
            palette,
            false,
        );
    }

    pub fn hit_button(&self, mouse: &MouseEvent, dialog: Rect) -> Option<usize> {
        hit_dialog_button(mouse, dialog, self.dialog.buttons)
    }

    pub fn hit_list(&self, mouse: &MouseEvent, outer: Rect) -> Option<(ConvertTabField, usize)> {
        let layout = self.layout(dialog_content_rect(outer));
        if point_in_list_row(&layout.from_list, mouse) {
            let row = mouse.row.saturating_sub(layout.from_list.y + 1);
            if row < LIST_ROWS {
                return Some((ConvertTabField::From, row as usize));
            }
        }
        if point_in_list_row(&layout.to_list, mouse) {
            let row = mouse.row.saturating_sub(layout.to_list.y + 1);
            if row < LIST_ROWS {
                return Some((ConvertTabField::To, row as usize));
            }
        }
        None
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
                KeyCode::Up => {
                    self.step(field, -1);
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Down => {
                    self.step(field, 1);
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Left => {
                    if field == ConvertTabField::To {
                        self.field_focus = Some(ConvertTabField::From);
                    }
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Right => {
                    if field == ConvertTabField::From {
                        self.field_focus = Some(ConvertTabField::To);
                    }
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Tab if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) => {
                    self.field_focus = Some(match field {
                        ConvertTabField::From => ConvertTabField::To,
                        ConvertTabField::To => ConvertTabField::From,
                    });
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::BackTab => {
                    self.field_focus = Some(match field {
                        ConvertTabField::From => ConvertTabField::To,
                        ConvertTabField::To => ConvertTabField::From,
                    });
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Tab => {
                    self.field_focus = match field {
                        ConvertTabField::From => Some(ConvertTabField::To),
                        ConvertTabField::To => None,
                    };
                    ConvertTabKeyResult::Consumed
                }
                KeyCode::Enter => {
                    self.field_focus = match field {
                        ConvertTabField::From => Some(ConvertTabField::To),
                        ConvertTabField::To => None,
                    };
                    ConvertTabKeyResult::Consumed
                }
                _ => ConvertTabKeyResult::Consumed,
            }
        } else {
            match self.dialog.handle_button_keys(key) {
                DialogKeyResult::Activate(_) => ConvertTabKeyResult::Submit,
                DialogKeyResult::Cancel => ConvertTabKeyResult::Cancel,
                DialogKeyResult::Consumed => ConvertTabKeyResult::Consumed,
                DialogKeyResult::Ignored => {
                    if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
                        self.field_focus = Some(ConvertTabField::From);
                        ConvertTabKeyResult::Consumed
                    } else if key.code == KeyCode::Up {
                        self.field_focus = Some(ConvertTabField::To);
                        ConvertTabKeyResult::Consumed
                    } else {
                        ConvertTabKeyResult::Consumed
                    }
                }
            }
        }
    }

    fn layout(&self, content: Rect) -> ConvertTabLayout {
        let list_width = content
            .width
            .saturating_sub(LIST_GAP)
            .saturating_div(2)
            .max(16);
        let lists_y = content.y + 2;
        let from_list = Rect {
            x: content.x,
            y: lists_y,
            width: list_width,
            height: LIST_BOX_HEIGHT,
        };
        let to_list = Rect {
            x: from_list.x + list_width + LIST_GAP,
            y: lists_y,
            width: list_width,
            height: LIST_BOX_HEIGHT,
        };
        ConvertTabLayout {
            from_list,
            to_list,
            button_y: dialog_button_row_y(content),
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

fn point_in_list_row(list: &Rect, mouse: &MouseEvent) -> bool {
    mouse.column >= list.x
        && mouse.column < list.x.saturating_add(list.width)
        && mouse.row >= list.y.saturating_add(1)
        && mouse.row < list.y.saturating_add(1 + LIST_ROWS)
}

fn draw_option_list(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    selected: usize,
    focused: bool,
    palette: ThemePalette,
) {
    let border_style = palette.convert_field_border_style(focused);
    let title_style = palette.convert_field_title_style(focused);
    let inner = panel::render_titled_frame(
        frame,
        area,
        title,
        palette.menu_panel_style(),
        border_style,
        title_style,
        false,
        panel::PanelBorder::Plain,
    );

    for row in 0..LIST_ROWS {
        let y = inner.y.saturating_add(row);
        let is_selected = row as usize == selected;
        let label = TAB_OPTIONS[row as usize].convert_option_label();
        let marker = if is_selected { '►' } else { ' ' };
        let text = format!(" {marker} {label}");
        let style = palette.convert_field_item_style(focused, is_selected);
        frame.render_widget(
            Paragraph::new(text).style(style),
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            },
        );
    }
}

fn tab_index(tab: Tabulation) -> usize {
    TAB_OPTIONS
        .iter()
        .position(|&t| t == tab)
        .unwrap_or(1)
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
    fn cycles_from_option_with_up_down() {
        let mut modal = ConvertTabulationModal::new(Tabulation::Spaces4);
        assert_eq!(modal.from_tab(), Tabulation::Spaces4);
        modal.handle_key(key(KeyCode::Down));
        assert_eq!(modal.from_tab(), Tabulation::Spaces8);
    }

    #[test]
    fn switches_field_with_left_right() {
        let mut modal = ConvertTabulationModal::new(Tabulation::Spaces2);
        modal.handle_key(key(KeyCode::Right));
        assert_eq!(modal.field_focus, Some(ConvertTabField::To));
        modal.handle_key(key(KeyCode::Left));
        assert_eq!(modal.field_focus, Some(ConvertTabField::From));
    }

    #[test]
    fn switches_field_with_tab() {
        let mut modal = ConvertTabulationModal::new(Tabulation::Spaces2);
        modal.handle_key(key(KeyCode::Tab));
        assert_eq!(modal.field_focus, Some(ConvertTabField::To));
        modal.handle_key(key(KeyCode::Tab));
        assert_eq!(modal.field_focus, None);
    }
}
