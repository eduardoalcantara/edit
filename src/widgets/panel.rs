//! Painel modal opaco reutilizável (menus dropdown, diálogos).
//!
//! Bordas `Plain` para editor/painéis; `Double` para modais e menus.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::theme::ThemePalette;

/// Caracteres VGA/CP437 usados na UI (Turbo Vision).
pub mod cp437 {
    /// Indicador de submenu (UTF-8). Alternativas úteis:
    /// `»` U+00BB, `›` U+203A, `▸` U+25B8, `→` U+2192, `>` ASCII.
    pub const SUBMENU_ARROW: char = '\u{00BB}';
    /// Sombra vertical (CP437 byte 219 → U+2588 bloco cheio).
    pub const SHADOW_V: char = '\u{2588}';
    /// Sombra horizontal (CP437 byte 223 → U+2580 ▀ meio bloco superior).
    pub const SHADOW_H: char = '\u{2580}';
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelBorder {
    Plain,
    Double,
}

struct BoxChars {
    tl: char,
    tr: char,
    bl: char,
    br: char,
    h: char,
    v: char,
    sep_l: char,
    sep_r: char,
}

impl PanelBorder {
    fn chars(self) -> BoxChars {
        match self {
            PanelBorder::Plain => BoxChars {
                tl: '┌',
                tr: '┐',
                bl: '└',
                br: '┘',
                h: '─',
                v: '│',
                sep_l: '├',
                sep_r: '┤',
            },
            PanelBorder::Double => BoxChars {
                tl: '╔',
                tr: '╗',
                bl: '╚',
                br: '╝',
                h: '═',
                v: '║',
                sep_l: '╠',
                sep_r: '╣',
            },
        }
    }
}

/// Área interna (conteúdo) dentro de uma borda de 1 célula.
pub fn inner_rect(outer: Rect) -> Rect {
    Rect {
        x: outer.x.saturating_add(1),
        y: outer.y.saturating_add(1),
        width: outer.width.saturating_sub(2),
        height: outer.height.saturating_sub(2),
    }
}

/// Área de texto do editor após borda externa (antes das margens internas).
pub fn editor_content_rect(outer: Rect, border_visible: bool, terminal_below: bool) -> Rect {
    if outer.width == 0 || outer.height == 0 {
        return outer;
    }

    let top = 1u16;
    let bottom = match (border_visible, terminal_below) {
        (true, true) => 1,
        (true, false) => 1,
        (false, true) => 1,
        (false, false) => 0,
    };
    let left = if border_visible { 1 } else { 0 };
    let right = if border_visible { 1 } else { 0 };

    Rect {
        x: outer.x.saturating_add(left),
        y: outer.y.saturating_add(top),
        width: outer.width.saturating_sub(left + right),
        height: outer.height.saturating_sub(top + bottom),
    }
}

/// Desenha a moldura do editor (título no topo) e retorna a área de conteúdo.
pub fn render_editor_frame(
    frame: &mut Frame,
    outer: Rect,
    title: &str,
    fill: Style,
    border_style: Style,
    border_visible: bool,
    terminal_below: bool,
) -> Rect {
    if outer.width == 0 || outer.height == 0 {
        return outer;
    }

    fill_rect(frame, outer, fill);
    let c = PanelBorder::Plain.chars();

    if border_visible {
        render_titled_top_row(
            frame,
            outer.x,
            outer.y,
            outer.width as usize,
            title,
            c.tl,
            c.tr,
            c.h,
            border_style,
        );

        let side_end = outer.height.saturating_sub(1);
        for row in 1..side_end {
            let y = outer.y.saturating_add(row);
            render_cell(frame, outer.x, y, c.v, border_style);
            render_cell(
                frame,
                outer.x.saturating_add(outer.width.saturating_sub(1)),
                y,
                c.v,
                border_style,
            );
        }

        if terminal_below {
            render_separator_row(
                frame,
                outer,
                outer.y.saturating_add(outer.height.saturating_sub(1)),
                PanelBorder::Plain,
                border_style,
            );
        } else if outer.height >= 2 {
            render_hline(
                frame,
                outer.x,
                outer.y.saturating_add(outer.height.saturating_sub(1)),
                outer.width as usize,
                c.bl,
                c.h,
                c.br,
                border_style,
            );
        }
    } else {
        render_titled_top_row(
            frame,
            outer.x,
            outer.y,
            outer.width as usize,
            title,
            c.bl,
            c.br,
            c.h,
            border_style,
        );

        if terminal_below && outer.height >= 2 {
            render_plain_hline(
                frame,
                outer.x,
                outer.y.saturating_add(outer.height.saturating_sub(1)),
                outer.width as usize,
                c.h,
                border_style,
            );
        }
    }

    editor_content_rect(outer, border_visible, terminal_below)
}

/// Tamanho externo dado largura de conteúdo e número de linhas internas.
pub fn outer_size(content_width: usize, row_count: usize) -> (u16, u16) {
    (
        content_width.saturating_add(2) as u16,
        row_count.saturating_add(2) as u16,
    )
}

/// Preenche retângulo com espaços opacos.
pub fn fill_rect(frame: &mut Frame, area: Rect, style: Style) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let line = " ".repeat(area.width as usize);
    for row in 0..area.height {
        frame.render_widget(
            Paragraph::new(Span::styled(line.clone(), style)).style(style),
            Rect {
                x: area.x,
                y: area.y.saturating_add(row),
                width: area.width,
                height: 1,
            },
        );
    }
}

/// Margem interna padrão de diálogos: 1 linha (topo/base), 2 colunas (laterais).
pub const DIALOG_MARGIN: (u16, u16, u16, u16) = (1, 1, 2, 2);

/// Recua uma área pelas margens (topo, base, esquerda, direita).
pub fn inset_rect(area: Rect, top: u16, bottom: u16, left: u16, right: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(left),
        y: area.y.saturating_add(top),
        width: area.width.saturating_sub(left + right),
        height: area.height.saturating_sub(top + bottom),
    }
}

/// Limpa, preenche e desenha borda com título embutido na linha superior.
pub fn render_titled_frame(
    frame: &mut Frame,
    outer: Rect,
    title: &str,
    fill: Style,
    border_style: Style,
    border: PanelBorder,
) -> Rect {
    frame.render_widget(Clear, outer);
    fill_rect(frame, outer, fill);

    let c = border.chars();
    let w = outer.width as usize;
    let h = outer.height as usize;
    if w < 2 || h < 2 {
        return inner_rect(outer);
    }

    render_titled_top_row(
        frame,
        outer.x,
        outer.y,
        w,
        title,
        c.tl,
        c.tr,
        c.h,
        border_style,
    );

    for row in 1..h.saturating_sub(1) {
        let y = outer.y.saturating_add(row as u16);
        render_cell(frame, outer.x, y, c.v, border_style);
        render_cell(
            frame,
            outer.x.saturating_add(outer.width.saturating_sub(1)),
            y,
            c.v,
            border_style,
        );
    }

    render_hline(
        frame,
        outer.x,
        outer.y.saturating_add(outer.height.saturating_sub(1)),
        w,
        c.bl,
        c.h,
        c.br,
        border_style,
    );

    inner_rect(outer)
}

/// Limpa, preenche e desenha borda ASCII completa.
pub fn render_frame(
    frame: &mut Frame,
    outer: Rect,
    fill: Style,
    border_style: Style,
    border: PanelBorder,
) {
    frame.render_widget(Clear, outer);
    fill_rect(frame, outer, fill);
    draw_box(frame, outer, border, border_style);
}

/// Separador horizontal conectado às bordas verticais (T-junction).
pub fn render_separator_row(
    frame: &mut Frame,
    outer: Rect,
    row_y: u16,
    border: PanelBorder,
    style: Style,
) {
    let c = border.chars();
    let w = outer.width as usize;
    if w < 2 {
        return;
    }
    let line = format!(
        "{}{}{}",
        c.sep_l,
        c.h.to_string().repeat(w.saturating_sub(2)),
        c.sep_r
    );
    frame.render_widget(
        Paragraph::new(Span::styled(line, style)).style(style),
        Rect {
            x: outer.x,
            y: row_y,
            width: outer.width,
            height: 1,
        },
    );
}

/// Linha de conteúdo dentro do painel (entre as bordas verticais).
pub fn render_content_row(
    frame: &mut Frame,
    outer: Rect,
    row_index: u16,
    text: &str,
    style: Style,
) {
    let inner = inner_rect(outer);
    let row_y = inner.y.saturating_add(row_index);
    if row_y >= inner.y.saturating_add(inner.height) {
        return;
    }
    frame.render_widget(
        Paragraph::new(Span::styled(text.to_string(), style)).style(style),
        Rect {
            x: inner.x,
            y: row_y,
            width: inner.width,
            height: 1,
        },
    );
}

pub fn render_drop_shadow(frame: &mut Frame, area: Rect, palette: ThemePalette) {
    let v_style = palette.shadow_vertical_style();
    let h_style = palette.shadow_horizontal_style();
    let v = cp437::SHADOW_V.to_string();
    let h = cp437::SHADOW_H.to_string();
    if area.x.saturating_add(area.width) < frame.area().width {
        frame.render_widget(
            Paragraph::new(Span::styled(v.repeat(area.height as usize), v_style)).style(v_style),
            Rect {
                x: area.x.saturating_add(area.width),
                y: area.y.saturating_add(1),
                width: 1,
                height: area.height,
            },
        );
    }
    if area.y.saturating_add(area.height) < frame.area().height {
        frame.render_widget(
            Paragraph::new(Span::styled(h.repeat(area.width as usize), h_style)).style(h_style),
            Rect {
                x: area.x.saturating_add(2),
                y: area.y.saturating_add(area.height),
                width: area.width,
                height: 1,
            },
        );
    }
}

fn draw_box(frame: &mut Frame, area: Rect, border: PanelBorder, style: Style) {
    let c = border.chars();
    let w = area.width as usize;
    let h = area.height as usize;
    if w < 2 || h < 2 {
        return;
    }

    render_hline(frame, area.x, area.y, w, c.tl, c.h, c.tr, style);

    for row in 1..h.saturating_sub(1) {
        let y = area.y.saturating_add(row as u16);
        render_cell(frame, area.x, y, c.v, style);
        render_cell(frame, area.x.saturating_add(area.width.saturating_sub(1)), y, c.v, style);
    }

    render_hline(
        frame,
        area.x,
        area.y.saturating_add(area.height.saturating_sub(1)),
        w,
        c.bl,
        c.h,
        c.br,
        style,
    );
}

fn render_plain_hline(
    frame: &mut Frame,
    x: u16,
    y: u16,
    width: usize,
    mid: char,
    style: Style,
) {
    if width == 0 {
        return;
    }
    let line = mid.to_string().repeat(width);
    frame.render_widget(
        Paragraph::new(Span::styled(line, style)).style(style),
        Rect {
            x,
            y,
            width: width as u16,
            height: 1,
        },
    );
}

fn render_titled_top_row(
    frame: &mut Frame,
    x: u16,
    y: u16,
    width: usize,
    title: &str,
    left: char,
    right: char,
    h: char,
    style: Style,
) {
    if width == 0 {
        return;
    }
    if width == 1 {
        render_cell(frame, x, y, left, style);
        return;
    }

    let mut title_part = format!(" {title} ");
    let max_title = width.saturating_sub(2);
    if title_part.chars().count() > max_title {
        title_part = title_part.chars().take(max_title).collect();
    }
    let title_len = title_part.chars().count();
    let fill_len = width.saturating_sub(2).saturating_sub(title_len);
    let line = format!(
        "{left}{}{}{right}",
        title_part,
        h.to_string().repeat(fill_len)
    );
    frame.render_widget(
        Paragraph::new(Span::styled(line, style)).style(style),
        Rect {
            x,
            y,
            width: width as u16,
            height: 1,
        },
    );
}

fn render_hline(
    frame: &mut Frame,
    x: u16,
    y: u16,
    width: usize,
    left: char,
    mid: char,
    right: char,
    style: Style,
) {
    let line = format!(
        "{}{}{}",
        left,
        mid.to_string().repeat(width.saturating_sub(2)),
        right
    );
    frame.render_widget(
        Paragraph::new(Span::styled(line, style)).style(style),
        Rect {
            x,
            y,
            width: width as u16,
            height: 1,
        },
    );
}

fn render_cell(frame: &mut Frame, x: u16, y: u16, ch: char, style: Style) {
    frame.render_widget(
        Paragraph::new(Span::styled(ch.to_string(), style)).style(style),
        Rect {
            x,
            y,
            width: 1,
            height: 1,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn editor_content_rect_visible_full() {
        let outer = Rect::new(0, 0, 10, 5);
        let inner = editor_content_rect(outer, true, false);
        assert_eq!(inner, Rect::new(1, 1, 8, 3));
    }

    #[test]
    fn editor_content_rect_hidden_no_terminal() {
        let outer = Rect::new(0, 0, 10, 5);
        let inner = editor_content_rect(outer, false, false);
        assert_eq!(inner, Rect::new(0, 1, 10, 4));
    }

    #[test]
    fn editor_content_rect_hidden_with_terminal() {
        let outer = Rect::new(0, 0, 10, 5);
        let inner = editor_content_rect(outer, false, true);
        assert_eq!(inner, Rect::new(0, 1, 10, 3));
    }
}
