use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tui_textarea::Input;

use crate::app::App;
use crate::block_select::BlockSelect;
use crate::cursors::CursorMode;
use crate::menus::{self, MenuEventResult};
use crate::modal::Modal;

pub fn poll(timeout: Duration) -> io::Result<bool> {
    event::poll(timeout)
}

pub fn read() -> io::Result<Event> {
    event::read()
}

pub fn dispatch(app: &mut App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if app.modal.is_active() {
                handle_modal_key(app, key);
            } else if app.menu_state.is_open() {
                handle_menu_key(app, key);
            } else {
                let menu_result = menus::handle_key(&app.menu_bar, &mut app.menu_state, key);
                if let MenuEventResult::Action(action) = menu_result {
                    app.dispatch_action(action);
                } else if menu_result == MenuEventResult::None {
                    handle_key(app, key);
                }
            }
        }
        Event::Mouse(mouse) if app.mouse_enabled && !app.modal.is_active() => {
            if app.menu_state.is_open() || is_menu_bar_click(app, &mouse) {
                handle_menu_mouse(app, mouse);
            } else {
                handle_mouse(app, mouse);
            }
        }
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn is_menu_bar_click(app: &App, mouse: &MouseEvent) -> bool {
    if !matches!(mouse.kind, MouseEventKind::Down(_)) {
        return false;
    }
    app.menu_state.top_hit_areas.iter().any(|r| {
        mouse.column >= r.x
            && mouse.column < r.x.saturating_add(r.width)
            && mouse.row == r.y
    })
}

fn handle_menu_key(app: &mut App, key: KeyEvent) {
    match menus::handle_key(&app.menu_bar, &mut app.menu_state, key) {
        MenuEventResult::Action(action) => app.dispatch_action(action),
        MenuEventResult::Closed => app.set_status("Menu fechado"),
        MenuEventResult::Consumed | MenuEventResult::None => {}
    }
}

fn handle_menu_mouse(app: &mut App, mouse: MouseEvent) {
    match menus::handle_mouse(&app.menu_bar, &mut app.menu_state, mouse) {
        MenuEventResult::Action(action) => app.dispatch_action(action),
        MenuEventResult::Closed => app.set_status("Menu fechado"),
        _ => {}
    }
}

fn handle_modal_key(app: &mut App, key: KeyEvent) {
    match &mut app.modal {
        Modal::Confirm { selected, .. } => match key.code {
            KeyCode::Enter => app.confirm_modal(),
            KeyCode::Esc => app.cancel_modal(),
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                *selected = 1 - *selected;
            }
            _ => {}
        },
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
            KeyCode::Tab if *replace_mode => {
                // focus toggle placeholder
            }
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
        Modal::None => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if alt && matches!(key.code, KeyCode::Char('r' | 'R')) {
        app.menu_state.open_top_menu(0, &app.menu_bar);
        return;
    }

    if alt && matches!(key.code, KeyCode::Char('f' | 'F')) {
        app.menu_state.open_top_menu(3, &app.menu_bar);
        return;
    }

    if key.code == KeyCode::F(3) {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            app.find_prev();
        } else {
            app.find_next();
        }
        return;
    }

    if ctrl && key.modifiers.contains(KeyModifiers::SHIFT) && matches!(key.code, KeyCode::Char('S' | 's')) {
        app.request_save_as();
        return;
    }

    if ctrl && key.modifiers.contains(KeyModifiers::SHIFT) && matches!(key.code, KeyCode::Char('V' | 'v')) {
        if let Some(text) = app.clipboard.get(1).or_else(|| app.clipboard.get(0)).map(str::to_string) {
            app.editor.paste(&text);
            app.set_status("Colar anterior");
        }
        return;
    }

    if ctrl {
        match key.code {
            KeyCode::Char('q' | 'Q') => app.request_quit(),
            KeyCode::Char('s' | 'S') => app.request_save(),
            KeyCode::Char('o' | 'O') => app.request_open(),
            KeyCode::Char('n' | 'N') => app.request_new_document(),
            KeyCode::Char('w' | 'W') => app.request_close(),
            KeyCode::Char('z' | 'Z') => app.editor.undo(),
            KeyCode::Char('y' | 'Y') => app.editor.redo(),
            KeyCode::Char('a' | 'A') => app.editor.select_all(),
            KeyCode::Char('f' | 'F') => app.modal = Modal::find("Buscar", &app.find_pattern),
            KeyCode::Char('h' | 'H') => {
                app.modal = Modal::find_replace("Substituir", &app.find_pattern, "");
            }
            KeyCode::Char('c' | 'C') => {
                if app.editor.copy_selection(&mut app.clipboard) {
                    app.set_status("Copiado");
                }
            }
            KeyCode::Char('x' | 'X') => {
                if app.editor.cut_selection(&mut app.clipboard) {
                    app.set_status("Recortado");
                }
            }
            KeyCode::Char('v' | 'V') => {
                if let Some(text) = app.clipboard.latest().map(str::to_string) {
                    app.editor.paste(&text);
                    app.set_status("Colado");
                }
            }
            _ => {
                let input: Input = key.into();
                app.editor.handle_input(input);
            }
        }
        return;
    }

    if key.code == KeyCode::Esc {
        app.editor.cancel_selection();
        return;
    }

    if key.code == KeyCode::Insert {
        app.toggle_edit_mode();
        return;
    }

    let input: Input = key.into();
    app.editor.handle_input(input);
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    let alt = mouse.modifiers.contains(KeyModifiers::ALT);
    let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if alt {
                let (row, col) = mouse_to_cursor(mouse);
                BlockSelect::on_press(app.editor.cursors_mut(), row, col, true);
                return;
            }
            if ctrl {
                let (row, col) = mouse_to_cursor(mouse);
                app.editor.cursors_mut().add_cursor(row, col);
                return;
            }
            if app.editor.cursors().mode != CursorMode::Normal {
                app.editor.cancel_selection();
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.editor.cursors().dragging_block {
                let (row, col) = mouse_to_cursor(mouse);
                BlockSelect::on_drag(app.editor.cursors_mut(), row, col);
                return;
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app.editor.cursors().dragging_block {
                BlockSelect::on_release(app.editor.cursors_mut());
                app.set_status(format!("Modo: {}", app.editor.cursors().mode_label()));
                return;
            }
        }
        _ => {}
    }

    let input: Input = mouse.into();
    app.editor.handle_input(input);
}

fn mouse_to_cursor(mouse: MouseEvent) -> (usize, usize) {
    let row = mouse.row.saturating_sub(3) as usize;
    let col = mouse.column as usize;
    (row, col)
}
