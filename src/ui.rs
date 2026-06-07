use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::modal::Modal;

const APP_NAME: &str = "Editor Linux";

pub fn draw(frame: &mut Frame, app: &mut App) {
    let mut constraints = vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ];
    if app.view.footer_visible {
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    draw_title_bar(frame, chunks[0], app);
    draw_menu_bar(frame, chunks[1], app);
    draw_editor(frame, chunks[2], app);
    if app.view.footer_visible {
        if let Some(chunk) = chunks.get(3) {
            draw_status_bar(frame, *chunk, app);
        }
    }

    if app.modal.is_active() {
        draw_modal(frame, app);
    }
}

fn draw_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let title = format!(
        " {} │ {} │ Tema: {} ",
        APP_NAME,
        app.document_title(),
        app.theme.label()
    );
    let header = Paragraph::new(Line::from(Span::styled(title, palette.header_style())))
        .style(palette.header_style());
    frame.render_widget(header, area);
}

fn draw_menu_bar(frame: &mut Frame, area: Rect, app: &mut App) {
    let palette = app.theme.palette();
    crate::menus::render(frame, area, &app.menu_bar, &mut app.menu_state, palette);
}

fn draw_editor(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(col) = app.view.guide_column.column() {
        let guide_x = area.x.saturating_add(col as u16);
        if guide_x < area.x.saturating_add(area.width) {
            let guide = Block::default().style(
                Style::default()
                    .fg(app.theme.palette().border)
                    .bg(app.theme.palette().editor_bg),
            );
            let guide_area = Rect {
                x: guide_x,
                y: area.y,
                width: 1,
                height: area.height,
            };
            frame.render_widget(guide, guide_area);
        }
    }
    frame.render_widget(app.editor.textarea(), area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let (line, col) = app.editor.cursor_line_col();
    let selection = app.editor.selection_label();
    let size = app.editor.byte_size();
    let session = if app.is_ssh_session { "SSH" } else { "Local" };
    let mouse = if app.mouse_enabled {
        "Mouse: ativo"
    } else {
        "Mouse: off"
    };
    let enc = app.document.encoding.label();
    let tab = app.document.tabulation.label();
    let view_flags = app.view.status_flags();

    let status = Line::from(vec![
        Span::styled(format!(" {enc} "), palette.accent_style()),
        Span::styled(format!("| Tab:{tab} "), palette.footer_style()),
        Span::styled(format!("| {view_flags} "), palette.footer_style()),
        Span::styled(format!("| Ln {line}, Col {col} "), palette.footer_style()),
        Span::styled(format!("| Sel: {selection} "), palette.footer_style()),
        Span::styled(format!("| {size} bytes "), palette.footer_style()),
        Span::styled(
            format!("| {} ", app.editor.mode().label()),
            palette.status_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("| {session} "), palette.footer_style()),
        Span::styled(format!("| {mouse} "), palette.footer_style()),
        Span::styled(
            if app.view.side_panel {
                "| Painel: on "
            } else {
                ""
            },
            palette.footer_style(),
        ),
        Span::styled(
            if app.view.terminal {
                "| Term: on "
            } else {
                ""
            },
            palette.footer_style(),
        ),
        Span::styled(format!("| {} ", app.status_message), palette.status_style()),
    ]);

    let footer = Paragraph::new(status)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(
                    Style::default()
                        .fg(palette.border)
                        .bg(palette.footer_bg),
                )
                .style(palette.footer_style()),
        )
        .style(palette.footer_style());

    frame.render_widget(footer, area);
}

fn draw_modal(frame: &mut Frame, app: &App) {
    let palette = app.theme.palette();
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    match &app.modal {
        Modal::Confirm {
            title,
            message,
            selected,
            ..
        } => {
            let options = "[ Enter ] Sim    [ Esc ] Cancelar";
            let focus = if *selected == 0 { "Sim" } else { "Cancelar" };
            let body = format!("{message}\n\n{options}\nFoco: {focus}");
            let dialog = Paragraph::new(body)
                .block(
                    Block::default()
                        .title(format!(" {title} "))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(palette.accent))
                        .style(palette.editor_text_style()),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left);
            frame.render_widget(dialog, area);
        }
        Modal::PathInput {
            title,
            prompt,
            input,
            ..
        } => {
            let body = format!("{prompt}\n\n{input}\n\n[ Enter ] Confirmar    [ Esc ] Cancelar");
            let dialog = Paragraph::new(body)
                .block(
                    Block::default()
                        .title(format!(" {title} "))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(palette.accent))
                        .style(palette.editor_text_style()),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(dialog, area);
        }
        Modal::Find {
            title,
            pattern,
            replacement,
            replace_mode,
        } => {
            let body = if *replace_mode {
                format!(
                    "Buscar:\n{pattern}\n\nSubstituir por:\n{replacement}\n\n[ Enter ] Substituir    [ Esc ] Cancelar"
                )
            } else {
                format!("Buscar:\n{pattern}\n\n[ Enter ] Buscar    [ Esc ] Cancelar")
            };
            let dialog = Paragraph::new(body)
                .block(
                    Block::default()
                        .title(format!(" {title} "))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(palette.accent))
                        .style(palette.editor_text_style()),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(dialog, area);
        }
        Modal::None => {}
    }
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
