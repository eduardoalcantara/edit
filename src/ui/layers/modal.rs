use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};

use crate::modal::{ConfirmLayout, Modal};
use crate::theme::ThemePalette;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;
use crate::widgets::panel::{self, PanelBorder, DIALOG_MARGIN};

pub struct ModalLayer;

impl UiLayer for ModalLayer {
    fn id(&self) -> LayerId {
        LayerId::Modal
    }

    fn is_visible(&self, app: &crate::app::App) -> bool {
        app.modal.is_active()
    }

    fn captures_input(&self, app: &crate::app::App) -> bool {
        app.modal.is_active()
    }

    fn paint(
        &self,
        frame: &mut ratatui::Frame<'_>,
        app: &mut crate::app::App,
        _: UiLayout,
        palette: ThemePalette,
    ) {
        let area = centered_rect(62, 28, frame.area());
        panel::render_drop_shadow(frame, area, palette);

        match app.modal.clone() {
            Modal::Confirm {
                title,
                message,
                selected,
                layout,
                ..
            } => {
                let content = draw_titled_modal(frame, area, &title, palette);
                let msg_lines = wrapped_line_count(&message, content.width as usize);
                let msg_height = msg_lines.min(content.height as usize).max(1) as u16;
                frame.render_widget(
                    Paragraph::new(message.as_str())
                        .style(palette.menu_panel_style())
                        .wrap(Wrap { trim: true }),
                    Rect {
                        x: content.x,
                        y: content.y,
                        width: content.width,
                        height: msg_height,
                    },
                );
                let labels = match layout {
                    ConfirmLayout::OkCancel => &["OK", "Cancelar"][..],
                    ConfirmLayout::SaveDiscardCancel => {
                        &["Salvar", "Não Salvar", "Cancelar"][..]
                    }
                };
                let button_y = content.y.saturating_add(msg_height + 1);
                draw_dialog_buttons(frame, content, button_y, selected, labels, palette);
            }
            Modal::PathInput {
                title,
                prompt,
                input,
                ..
            } => {
                let content = draw_titled_modal(frame, area, &title, palette);
                let body = format!("{prompt}\n\n {input}▌");
                let body_lines = wrapped_line_count(&body, content.width as usize);
                let body_height = body_lines.min(content.height.saturating_sub(2) as usize).max(1) as u16;
                frame.render_widget(
                    Paragraph::new(body)
                        .style(palette.menu_panel_style())
                        .wrap(Wrap { trim: true }),
                    Rect {
                        x: content.x,
                        y: content.y,
                        width: content.width,
                        height: body_height,
                    },
                );
                let button_y = content.y.saturating_add(content.height.saturating_sub(1));
                draw_dialog_buttons(frame, content, button_y, 0, &["OK", "Cancelar"], palette);
            }
            Modal::Find {
                title,
                pattern,
                replacement,
                replace_mode,
            } => {
                let content = draw_titled_modal(frame, area, &title, palette);
                let body = if replace_mode {
                    format!("Texto:\n {pattern}\n\nSubstituir:\n {replacement}▌")
                } else {
                    format!("Texto:\n {pattern}▌")
                };
                let body_lines = wrapped_line_count(&body, content.width as usize);
                let body_height = body_lines.min(content.height.saturating_sub(2) as usize).max(1) as u16;
                frame.render_widget(
                    Paragraph::new(body)
                        .style(palette.menu_panel_style())
                        .wrap(Wrap { trim: true }),
                    Rect {
                        x: content.x,
                        y: content.y,
                        width: content.width,
                        height: body_height,
                    },
                );
                let button_y = content.y.saturating_add(content.height.saturating_sub(1));
                draw_dialog_buttons(frame, content, button_y, 0, &["OK", "Cancelar"], palette);
            }
            Modal::None => {}
        }
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        match &mut app.modal {
            Modal::Confirm {
                selected,
                layout,
                ..
            } => {
                let button_count = match layout {
                    ConfirmLayout::OkCancel => 2,
                    ConfirmLayout::SaveDiscardCancel => 3,
                };
                match key.code {
                    KeyCode::Enter => app.confirm_modal(),
                    KeyCode::Esc => app.cancel_modal(),
                    KeyCode::Left => *selected = selected.saturating_sub(1),
                    KeyCode::Right | KeyCode::Tab => {
                        *selected = (*selected + 1) % button_count;
                    }
                    _ => {}
                }
            }
            Modal::PathInput { input, .. } => match key.code {
                KeyCode::Enter => app.submit_path_input(),
                KeyCode::Esc => app.cancel_modal(),
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(ch);
                }
                _ => {}
            },
            Modal::Find {
                pattern,
                replacement,
                replace_mode,
                ..
            } => match key.code {
                KeyCode::Enter => app.submit_find(),
                KeyCode::Esc => app.cancel_modal(),
                KeyCode::Tab if *replace_mode => {}
                KeyCode::Backspace => {
                    pattern.pop();
                }
                KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if *replace_mode && key.modifiers.contains(KeyModifiers::SHIFT) {
                        replacement.push(ch);
                    } else {
                        pattern.push(ch);
                    }
                }
                _ => {}
            },
            Modal::None => return InputResult::Unhandled,
        }
        InputResult::Consumed
    }

    fn on_mouse(&self, mouse: MouseEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        if !app.mouse_enabled {
            return InputResult::Unhandled;
        }

        let frame = Rect {
            x: 0,
            y: 0,
            width: app.last_frame_width,
            height: app.last_frame_height,
        };
        let dialog = centered_rect(62, 28, frame);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = hit_dialog_button(&mouse, dialog, &app.modal) {
                    activate_modal_button(app, idx);
                }
            }
            MouseEventKind::Moved => {
                if let Some(idx) = hit_dialog_button(&mouse, dialog, &app.modal) {
                    if let Modal::Confirm { selected, .. } = &mut app.modal {
                        *selected = idx;
                    }
                }
            }
            _ => {}
        }
        InputResult::Consumed
    }

    fn footer_hint(&self, app: &crate::app::App) -> Option<String> {
        Some(match &app.modal {
            Modal::Confirm { layout, .. } => match layout {
                ConfirmLayout::SaveDiscardCancel => {
                    "←→ botões | Enter confirma | Esc cancela".to_string()
                }
                ConfirmLayout::OkCancel => "Enter OK | Esc cancelar".to_string(),
            },
            Modal::PathInput { .. } => "Enter confirma | Esc cancelar".to_string(),
            Modal::Find { replace_mode, .. } if *replace_mode => {
                "Enter substituir | Esc cancelar".to_string()
            }
            Modal::Find { .. } => "Enter buscar | Esc cancelar".to_string(),
            Modal::None => return None,
        })
    }
}

fn draw_titled_modal(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    title: &str,
    palette: ThemePalette,
) -> Rect {
    let frame_title = format!("[ {title} ]");
    let inner = panel::render_titled_frame(
        frame,
        area,
        &frame_title,
        palette.menu_panel_style(),
        palette.menu_border_style(),
        PanelBorder::Double,
    );
    panel::fill_rect(frame, inner, palette.menu_panel_style());
    let (top, bottom, left, right) = DIALOG_MARGIN;
    panel::inset_rect(inner, top, bottom, left, right)
}

fn draw_dialog_buttons(
    frame: &mut ratatui::Frame<'_>,
    content: Rect,
    y: u16,
    selected: usize,
    labels: &[&str],
    palette: ThemePalette,
) {
    let mut x = content.x;
    for (i, label) in labels.iter().enumerate() {
        let width = label.chars().count() as u16 + 2;
        let area = Rect {
            x,
            y,
            width,
            height: 1,
        };
        let focused = i == selected;
        let text = format!("[{label}]");
        frame.render_widget(
            Paragraph::new(text).style(palette.button_style(focused)),
            area,
        );
        x = x.saturating_add(width.saturating_add(1));
    }
}

fn modal_content_rect(area: Rect) -> Rect {
    let inner = panel::inner_rect(area);
    let (top, bottom, left, right) = DIALOG_MARGIN;
    panel::inset_rect(inner, top, bottom, left, right)
}

fn confirm_button_y(area: Rect, message: &str) -> u16 {
    let content = modal_content_rect(area);
    let msg_lines = wrapped_line_count(message, content.width as usize).max(1) as u16;
    content.y.saturating_add(msg_lines + 1)
}

fn form_button_y(area: Rect) -> u16 {
    let content = modal_content_rect(area);
    content.y.saturating_add(content.height.saturating_sub(1))
}

fn wrapped_line_count(text: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let mut lines = 1usize;
    let mut col = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            lines += 1;
            col = 0;
            continue;
        }
        col += 1;
        if col >= width {
            lines += 1;
            col = 0;
        }
    }
    lines.max(1)
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn modal_button_labels(modal: &Modal) -> Option<&'static [&'static str]> {
    match modal {
        Modal::Confirm { layout, .. } => Some(match layout {
            ConfirmLayout::OkCancel => &["OK", "Cancelar"],
            ConfirmLayout::SaveDiscardCancel => &["Salvar", "Não Salvar", "Cancelar"],
        }),
        Modal::PathInput { .. } | Modal::Find { .. } => Some(&["OK", "Cancelar"]),
        Modal::None => None,
    }
}

fn hit_dialog_button(mouse: &MouseEvent, dialog: Rect, modal: &Modal) -> Option<usize> {
    let labels = modal_button_labels(modal)?;
    let content = modal_content_rect(dialog);
    let row_y = match modal {
        Modal::Confirm { message, .. } => confirm_button_y(dialog, message),
        Modal::PathInput { .. } | Modal::Find { .. } => form_button_y(dialog),
        Modal::None => return None,
    };
    if mouse.row != row_y {
        return None;
    }
    let mut x = content.x;
    for (i, label) in labels.iter().enumerate() {
        let width = label.chars().count() as u16 + 2;
        if mouse.column >= x && mouse.column < x.saturating_add(width) {
            return Some(i);
        }
        x = x.saturating_add(width.saturating_add(1));
    }
    None
}

fn activate_modal_button(app: &mut crate::app::App, index: usize) {
    match &mut app.modal {
        Modal::Confirm { selected, .. } => {
            *selected = index;
            app.confirm_modal();
        }
        Modal::PathInput { .. } => match index {
            0 => app.submit_path_input(),
            _ => app.cancel_modal(),
        },
        Modal::Find { .. } => match index {
            0 => app.submit_find(),
            _ => app.cancel_modal(),
        },
        Modal::None => {}
    }
}
