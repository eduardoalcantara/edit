use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

const APP_NAME: &str = "Editor Linux";

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_editor(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let title = format!(
        " {} │ {} │ Tema: {} ",
        APP_NAME,
        app.document_title,
        app.theme.label()
    );
    let header = Paragraph::new(Line::from(Span::styled(
        title,
        palette.header_style(),
    )))
    .style(palette.header_style());
    frame.render_widget(header, area);
}

fn draw_editor(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(app.editor.textarea(), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let shortcuts = Line::from(vec![
        Span::styled(" Ctrl+S ", palette.accent_style().add_modifier(Modifier::BOLD)),
        Span::styled("Salvar  ", palette.footer_style()),
        Span::styled("Ctrl+O ", palette.accent_style().add_modifier(Modifier::BOLD)),
        Span::styled("Abrir  ", palette.footer_style()),
        Span::styled("Ctrl+Q ", palette.accent_style().add_modifier(Modifier::BOLD)),
        Span::styled("Sair", palette.footer_style()),
    ]);

    let status = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status_message),
            palette.status_style(),
        ),
        Span::styled(
            if app.mouse_enabled {
                " Mouse: ativo "
            } else {
                " Mouse: indisponível (teclado ativo) "
            },
            palette.footer_style(),
        ),
    ]);

    let footer = Paragraph::new(vec![shortcuts, status])
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(
                    ratatui::style::Style::default()
                        .fg(palette.border)
                        .bg(palette.footer_bg),
                )
                .style(palette.footer_style()),
        )
        .style(palette.footer_style());

    frame.render_widget(footer, area);
}
