//! Painel modal opaco reutilizável (menus dropdown, diálogos).
//!
//! Bordas `Plain` para editor/painéis; `Double` para modais e menus.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::theme::ThemePalette;

/// Formato padrão de título embutido na borda: `─[ info ]─` (sem espaço antes de `[`).
pub fn framed_title(label: &str) -> String {
    format!("[ {label} ]")
}

/// Caracteres VGA/CP437 usados na UI (Turbo Vision).
pub mod cp437 {
    /// Indicador de submenu. ASCII `>` funciona em PuTTY/SSH; `»` U+00BB se UTF-8 ok.
    pub const SUBMENU_ARROW: char = '>';
    /// Check de item ativado (estilo Turbo Vision / CP437 251 → U+221A).
    pub const CHECK_ON: char = '\u{221A}';

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

pub(crate) struct BoxChars {
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    pub h: char,
    pub v: char,
    pub sep_l: char,
    pub sep_r: char,
}

impl PanelBorder {
    pub(crate) fn chars(self) -> BoxChars {
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
/// `terminal_block`: linhas reservadas na base da shell (bloco terminal unificado).
pub fn editor_content_rect(outer: Rect, border_visible: bool, terminal_block: Option<u16>) -> Rect {
    if outer.width == 0 || outer.height == 0 {
        return outer;
    }

    let top = 1u16;
    let bottom = match terminal_block {
        Some(n) => n,
        None => u16::from(border_visible),
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
    title_style: Style,
    border_visible: bool,
    terminal_block: Option<u16>,
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
            title_style,
            false,
            true,
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

        if terminal_block.is_none() && outer.height >= 2 {
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
            title_style,
            false,
            true,
        );
    }

    editor_content_rect(outer, border_visible, terminal_block)
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
    title_style: Style,
    center_title: bool,
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
        title_style,
        center_title,
        false,
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

/// Linha de conteúdo com estilos por span (marcadores à direita com cor distinta).
pub fn render_content_line(
    frame: &mut Frame,
    outer: Rect,
    row_index: u16,
    line: Line<'_>,
) {
    let inner = inner_rect(outer);
    let row_y = inner.y.saturating_add(row_index);
    if row_y >= inner.y.saturating_add(inner.height) {
        return;
    }
    frame.render_widget(
        Paragraph::new(line),
        Rect {
            x: inner.x,
            y: row_y,
            width: inner.width,
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
    render_plain_row(frame, inner, row_index, text, style);
}

/// Linha de texto sem margem interna (área já é a região útil).
pub fn render_plain_row(
    frame: &mut Frame,
    area: Rect,
    row_index: u16,
    text: &str,
    style: Style,
) {
    if area.width == 0 || area.height == 0 || row_index >= area.height {
        return;
    }
    let w = area.width as usize;
    let display: String = text.chars().take(w).collect();
    frame.render_widget(
        Paragraph::new(Span::styled(display, style)).style(style),
        Rect {
            x: area.x,
            y: area.y.saturating_add(row_index),
            width: area.width,
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
        let h_count = area.width.saturating_sub(1) as usize;
        frame.render_widget(
            Paragraph::new(Span::styled(h.repeat(h_count), h_style)).style(h_style),
            Rect {
                x: area.x.saturating_add(2),
                y: area.y.saturating_add(area.height),
                width: h_count as u16,
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
    label: &str,
    left: char,
    right: char,
    h: char,
    border_style: Style,
    title_style: Style,
    center_title: bool,
    lead_line_before_title: bool,
) {
    if width == 0 {
        return;
    }
    if width == 1 {
        render_cell(frame, x, y, left, border_style);
        return;
    }

    let prefix_len = if lead_line_before_title && !center_title {
        1
    } else {
        0
    };
    let interior = width.saturating_sub(2);
    let max_inner = interior
        .saturating_sub(prefix_len)
        .saturating_sub(2);
    let mut inner = format!(" {label} ");
    if inner.chars().count() > max_inner {
        inner = inner.chars().take(max_inner).collect();
    }
    let inner_len = inner.chars().count();
    let title_core_len = 2 + inner_len;

    let (left_fill, right_fill) = if center_title {
        let total_fill = interior.saturating_sub(title_core_len);
        (total_fill / 2, total_fill - total_fill / 2)
    } else {
        (
            0,
            interior
                .saturating_sub(prefix_len + title_core_len),
        )
    };

    let mut spans = vec![Span::styled(left.to_string(), border_style)];
    if center_title {
        spans.push(Span::styled(h.to_string().repeat(left_fill), border_style));
    } else if lead_line_before_title {
        spans.push(Span::styled(h.to_string(), border_style));
    }
    spans.push(Span::styled("[".to_string(), border_style));
    spans.push(Span::styled(inner, title_style));
    spans.push(Span::styled("]".to_string(), border_style));
    spans.push(Span::styled(h.to_string().repeat(right_fill), border_style));
    spans.push(Span::styled(right.to_string(), border_style));

    frame.render_widget(
        Paragraph::new(Line::from(spans)),
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

pub(crate) fn render_cell(frame: &mut Frame, x: u16, y: u16, ch: char, style: Style) {
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
        let inner = editor_content_rect(outer, true, None);
        assert_eq!(inner, Rect::new(1, 1, 8, 3));
    }

    #[test]
    fn editor_content_rect_hidden_no_terminal() {
        let outer = Rect::new(0, 0, 10, 5);
        let inner = editor_content_rect(outer, false, None);
        assert_eq!(inner, Rect::new(0, 1, 10, 4));
    }

    #[test]
    fn editor_content_rect_with_terminal_block() {
        let outer = Rect::new(0, 0, 10, 12);
        let inner = editor_content_rect(outer, true, Some(6));
        assert_eq!(inner, Rect::new(1, 1, 8, 5));
    }

    #[test]
    fn horizontal_shadow_length_compensates_leading_offset() {
        let area = Rect::new(5, 3, 20, 8);
        let h_count = area.width.saturating_sub(1) as usize;
        assert_eq!(h_count, 19);
        assert_eq!(
            area.x.saturating_add(2) + h_count as u16 - 1,
            area.x.saturating_add(area.width)
        );
    }
}
