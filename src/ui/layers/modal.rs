use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::modal::{convert_tab::ConvertTabKeyResult, DialogButtonAction, DialogKeyResult, Modal};
use crate::theme::ThemePalette;
use crate::ui::layer::{InputResult, LayerId, UiLayer};
use crate::ui::layout::UiLayout;

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
        match &app.modal {
            Modal::ConvertTabulation(modal) => {
                let area = modal.outer_rect(frame.area());
                modal.paint(frame, area, palette);
            }
            _ => {
                app.modal.refresh_body();
                let Some(dialog) = app.modal.dialog() else {
                    return;
                };
                let area = dialog.outer_rect(frame.area());
                dialog.paint(frame, area, palette);
            }
        }
    }

    fn on_key(&self, key: KeyEvent, app: &mut crate::app::App, _: UiLayout) -> InputResult {
        match &mut app.modal {
            Modal::Confirm { dialog, .. } => match dialog.handle_button_keys(key) {
                DialogKeyResult::Activate(_) => {
                    app.confirm_modal();
                    InputResult::Consumed
                }
                DialogKeyResult::Cancel => {
                    app.cancel_modal();
                    InputResult::Consumed
                }
                DialogKeyResult::Consumed => InputResult::Consumed,
                DialogKeyResult::Ignored => InputResult::Consumed,
            },
            Modal::PathInput { dialog, input, .. } => {
                match dialog.handle_button_keys(key) {
                    DialogKeyResult::Activate(_) => {
                        app.submit_path_input();
                        return InputResult::Consumed;
                    }
                    DialogKeyResult::Cancel => {
                        app.cancel_modal();
                        return InputResult::Consumed;
                    }
                    DialogKeyResult::Consumed => return InputResult::Consumed,
                    DialogKeyResult::Ignored => {}
                }
                match key.code {
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(ch);
                    }
                    _ => {}
                }
                InputResult::Consumed
            }
            Modal::Find {
                dialog,
                pattern,
                replacement,
                replace_mode,
                ..
            } => {
                match dialog.handle_button_keys(key) {
                    DialogKeyResult::Activate(_) => {
                        app.submit_find();
                        return InputResult::Consumed;
                    }
                    DialogKeyResult::Cancel => {
                        app.cancel_modal();
                        return InputResult::Consumed;
                    }
                    DialogKeyResult::Consumed => return InputResult::Consumed,
                    DialogKeyResult::Ignored => {}
                }
                match key.code {
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
                }
                InputResult::Consumed
            }
            Modal::ConvertTabulation(modal) => match modal.handle_key(key) {
                ConvertTabKeyResult::Submit => {
                    app.submit_convert_tabulation();
                    InputResult::Consumed
                }
                ConvertTabKeyResult::Cancel => {
                    app.cancel_modal();
                    InputResult::Consumed
                }
                ConvertTabKeyResult::Consumed => InputResult::Consumed,
            },
            Modal::None => InputResult::Unhandled,
        }
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

        match &mut app.modal {
            Modal::ConvertTabulation(modal) => {
                let outer = modal.outer_rect(frame);
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some((field, idx)) = modal.hit_list(&mouse, outer) {
                            modal.field_focus = Some(field);
                            match field {
                                crate::modal::convert_tab::ConvertTabField::From => {
                                    modal.from_idx = idx;
                                }
                                crate::modal::convert_tab::ConvertTabField::To => {
                                    modal.to_idx = idx;
                                }
                            }
                        } else if let Some(idx) = modal.hit_button(&mouse, outer) {
                            activate_button(app, idx);
                        }
                    }
                    MouseEventKind::Moved => {
                        if let Some((field, idx)) = modal.hit_list(&mouse, outer) {
                            modal.field_focus = Some(field);
                            match field {
                                crate::modal::convert_tab::ConvertTabField::From => {
                                    modal.from_idx = idx;
                                }
                                crate::modal::convert_tab::ConvertTabField::To => {
                                    modal.to_idx = idx;
                                }
                            }
                        } else if let Some(idx) = modal.hit_button(&mouse, outer) {
                            modal.field_focus = None;
                            modal.dialog.set_selected(idx);
                        }
                    }
                    _ => {}
                }
                InputResult::Consumed
            }
            _ => {
                let Some(dialog) = app.modal.dialog() else {
                    return InputResult::Consumed;
                };
                let outer = dialog.outer_rect(frame);

                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(idx) = dialog.hit_button(&mouse, outer) {
                            activate_button(app, idx);
                        }
                    }
                    MouseEventKind::Moved => {
                        if let Some(idx) = dialog.hit_button(&mouse, outer) {
                            app.modal.dialog_mut().map(|dialog| dialog.set_selected(idx));
                        }
                    }
                    _ => {}
                }
                InputResult::Consumed
            }
        }
    }

    fn footer_hint(&self, app: &crate::app::App) -> Option<String> {
        match &app.modal {
            Modal::ConvertTabulation(modal) => modal.focused_help().map(str::to_string),
            _ => app
                .modal
                .dialog()
                .and_then(|dialog| dialog.focused_help())
                .map(str::to_string),
        }
    }
}

fn activate_button(app: &mut crate::app::App, index: usize) {
    app.modal.dialog_mut().map(|dialog| dialog.set_selected(index));
    let action = app
        .modal
        .dialog()
        .and_then(|dialog| dialog.selected_action());
    match (&app.modal, action) {
        (Modal::Confirm { .. }, _) => app.confirm_modal(),
        (Modal::ConvertTabulation(_), Some(DialogButtonAction::Primary)) => {
            app.submit_convert_tabulation();
        }
        (_, Some(DialogButtonAction::Primary)) => match &app.modal {
            Modal::PathInput { .. } => app.submit_path_input(),
            Modal::Find { .. } => app.submit_find(),
            _ => {}
        },
        _ => app.cancel_modal(),
    }
}
