use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::modal::Modal;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let palette = app.theme.palette();
    frame.render_widget(
        Block::default().style(palette.desktop_style()),
        frame.area(),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_menu_bar(frame, chunks[0], app);
    draw_editor(frame, chunks[1], app);
    draw_footer_bar(frame, chunks[2], app);

    if app.modal.is_active() {
        draw_modal(frame, app);
    }
}

fn draw_menu_bar(frame: &mut Frame, area: Rect, app: &mut App) {
    let palette = app.theme.palette();
    crate::menus::render(frame, area, &app.menu_bar, &mut app.menu_state, palette);
}

fn draw_editor(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    if let Some(col) = app.view.guide_column.column() {
        let guide_x = area.x.saturating_add(col as u16);
        if guide_x < area.x.saturating_add(area.width) {
            let guide_area = Rect {
                x: guide_x,
                y: area.y,
                width: 1,
                height: area.height,
            };
            frame.render_widget(
                Paragraph::new(" ").style(
                    Style::default()
                        .fg(palette.border)
                        .bg(palette.editor_bg),
                ),
                guide_area,
            );
        }
    }
    frame.render_widget(app.editor.textarea(), area);
}

fn draw_footer_bar(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let hint = footer_context_hint(app);

    let shortcuts = Line::from(vec![
        hotkey_span("F1", "Help", &palette),
        Span::raw("  "),
        hotkey_span("F2", "Save", &palette),
        Span::raw("  "),
        hotkey_span("F3", "Open", &palette),
        Span::raw("  "),
        hotkey_span("F10", "Menu", &palette),
        Span::styled(format!("  {hint}"), palette.footer_style()),
    ]);

    let footer = Paragraph::new(shortcuts)
        .style(palette.footer_style())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(
                    Style::default()
                        .fg(palette.border)
                        .bg(palette.footer_bg),
                ),
        );
    frame.render_widget(footer, area);
}

fn hotkey_span<'a>(
    key: &'a str,
    label: &'a str,
    palette: &crate::theme::ThemePalette,
) -> Span<'a> {
    Span::styled(
        format!("{key}-{label}"),
        palette.footer_hotkey_style().add_modifier(Modifier::BOLD),
    )
}

fn footer_context_hint(app: &App) -> String {
    if app.modal.is_active() {
        return app.status_message.clone();
    }
    if app.menu_state.is_open() {
        return "Setas navegam │ Enter seleciona │ Esc fecha".to_string();
    }
    format!(
        "{} │ {} │ Ln {} Col {} │ {}",
        app.document.encoding.label(),
        app.document.tabulation.label(),
        app.editor.cursor_line_col().0,
        app.editor.cursor_line_col().1,
        app.status_message
    )
}

fn draw_modal(frame: &mut Frame, app: &App) {
    let palette = app.theme.palette();
    let area = centered_rect(62, 42, frame.area());
    render_modal_shadow(frame, area, palette);
    frame.render_widget(Clear, area);

    match &app.modal {
        Modal::Confirm {
            title,
            message,
            selected,
            ..
        } => {
            let block = modal_block(title, palette);
            let inner = block.inner(area);
            frame.render_widget(block, area);
            fill_area(frame, inner, palette.menu_panel_style());

            let body = Paragraph::new(message.as_str())
                .style(palette.menu_panel_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(body, inner);

            draw_dialog_buttons(frame, area, *selected, &["OK", "Cancelar"], palette);
        }
        Modal::PathInput {
            title,
            prompt,
            input,
            ..
        } => {
            let block = modal_block(title, palette);
            let inner = block.inner(area);
            frame.render_widget(block, area);
            fill_area(frame, inner, palette.menu_panel_style());

            let body = format!("{prompt}\n\n {input}▌");
            let dialog = Paragraph::new(body)
                .style(palette.menu_panel_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(dialog, inner);

            draw_dialog_buttons(frame, area, 0, &["OK", "Cancelar"], palette);
        }
        Modal::Find {
            title,
            pattern,
            replacement,
            replace_mode,
        } => {
            let block = modal_block(title, palette);
            let inner = block.inner(area);
            frame.render_widget(block, area);
            fill_area(frame, inner, palette.menu_panel_style());

            let body = if *replace_mode {
                format!("Texto:\n {pattern}\n\nSubstituir:\n {replacement}▌")
            } else {
                format!("Texto:\n {pattern}▌")
            };
            let dialog = Paragraph::new(body)
                .style(palette.menu_panel_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(dialog, inner);

            draw_dialog_buttons(frame, area, 0, &["OK", "Cancelar"], palette);
        }
        Modal::None => {}
    }
}

fn modal_block(title: &str, palette: crate::theme::ThemePalette) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(palette.menu_border_style())
        .style(palette.menu_panel_style())
}

fn fill_area(frame: &mut Frame, area: Rect, style: Style) {
    for row in 0..area.height {
        frame.render_widget(
            Paragraph::new(Span::styled(" ".repeat(area.width as usize), style)).style(style),
            Rect {
                x: area.x,
                y: area.y.saturating_add(row),
                width: area.width,
                height: 1,
            },
        );
    }
}

fn draw_dialog_buttons(
    frame: &mut Frame,
    dialog: Rect,
    selected: usize,
    labels: &[&str],
    palette: crate::theme::ThemePalette,
) {
    let mut x = dialog.x.saturating_add(2);
    let y = dialog.y.saturating_add(dialog.height.saturating_sub(2));
    for (i, label) in labels.iter().enumerate() {
        let width = label.len() as u16 + 2;
        let area = Rect {
            x,
            y,
            width,
            height: 1,
        };
        let focused = i == selected;
        let text = format!("[{label}]");
        frame.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(palette.button_style(focused)),
            area,
        );
        x = x.saturating_add(width.saturating_add(2));
    }
}

fn render_modal_shadow(frame: &mut Frame, area: Rect, palette: crate::theme::ThemePalette) {
    let shadow = palette.shadow_style();
    frame.render_widget(
        Paragraph::new(Span::styled("█".repeat(area.height as usize), shadow)).style(shadow),
        Rect {
            x: area.x.saturating_add(area.width),
            y: area.y.saturating_add(1),
            width: 1,
            height: area.height,
        },
    );
    frame.render_widget(
        Paragraph::new(Span::styled("█".repeat(area.width as usize), shadow)).style(shadow),
        Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(area.height),
            width: area.width,
            height: 1,
        },
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
