use super::context::PdfRenderContext;
use super::types::extract_plain_text;
use crate::ast::{ElementNode, Node};
use printpdf::path::PaintMode;
use printpdf::{Color, Line, Mm, Point, Rect, Rgb};

pub fn render_table(el: &ElementNode, ctx: &mut PdfRenderContext) {
    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);
    if rows.is_empty() {
        return;
    }

    let is_borderless = el.classes.iter().any(|c| c == "borderless")
        || el
            .attributes
            .get("class")
            .map(|c| c.contains("borderless"))
            .unwrap_or(false)
        || el
            .inline_styles
            .get("border")
            .map(|b| b.contains("none"))
            .unwrap_or(false);

    let is_align_right = el.classes.iter().any(|c| c == "align-right")
        || el
            .attributes
            .get("class")
            .map(|c| c.contains("align-right"))
            .unwrap_or(false)
        || el
            .attributes
            .get("align")
            .map(|a| a == "right")
            .unwrap_or(false)
        || el
            .inline_styles
            .get("margin-left")
            .map(|m| m.contains("auto"))
            .unwrap_or(false);

    let (table_left, table_width) = if is_align_right {
        (ctx.left_margin + ctx.content_width * 0.52, ctx.content_width * 0.48)
    } else {
        (ctx.left_margin, ctx.content_width)
    };

    let mut max_cols = 1;
    for r in &rows {
        let cell_count = r
            .children
            .iter()
            .filter(|c| {
                if let Node::Element(cell) = c {
                    let tag = cell.tag.to_lowercase();
                    tag == "td" || tag == "th"
                } else {
                    false
                }
            })
            .count();
        if cell_count > max_cols {
            max_cols = cell_count;
        }
    }

    let col_width = table_width / (max_cols as f32);
    *ctx.cursor_y -= 3.0;

    for row in rows {
        let cells: Vec<&ElementNode> = row
            .children
            .iter()
            .filter_map(|c| {
                if let Node::Element(cell) = c {
                    let tag = cell.tag.to_lowercase();
                    if tag == "td" || tag == "th" {
                        Some(cell)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if cells.is_empty() {
            continue;
        }

        let is_header_row = !is_borderless && cells.iter().any(|c| c.tag.to_lowercase() == "th");
        let font = if is_header_row {
            ctx.fonts.helvetica_bold
        } else {
            ctx.fonts.helvetica_reg
        };

        let mut row_cell_lines: Vec<Vec<String>> = Vec::new();
        let mut max_lines = 1;
        for cell in &cells {
            let mut l = Vec::new();
            for c in &cell.children {
                if let Node::Element(p) = c {
                    let txt = extract_plain_text(p);
                    if !txt.trim().is_empty() {
                        l.push(txt.trim().to_string());
                    }
                }
            }
            if l.is_empty() {
                let full = extract_plain_text(cell);
                for part in full.split('\n') {
                    if !part.trim().is_empty() {
                        l.push(part.trim().to_string());
                    }
                }
            }
            if l.len() > max_lines {
                max_lines = l.len();
            }
            row_cell_lines.push(l);
        }

        let row_height_mm = (max_lines as f32 * 4.6).max(7.5);
        ctx.check_page_overflow(row_height_mm);

        if !is_borderless {
            if is_header_row {
                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    248.0 / 255.0,
                    250.0 / 255.0,
                    252.0 / 255.0,
                    None,
                )));
                let mut header_rect = Rect::new(
                    Mm(table_left),
                    Mm(*ctx.cursor_y - row_height_mm),
                    Mm(table_left + table_width),
                    Mm(*ctx.cursor_y),
                );
                header_rect.mode = PaintMode::Fill;
                ctx.current_layer.add_rect(header_rect);
            }

            ctx.current_layer.set_outline_color(Color::Rgb(Rgb::new(
                226.0 / 255.0,
                232.0 / 255.0,
                240.0 / 255.0,
                None,
            )));
            ctx.current_layer.set_outline_thickness(0.5);
            let border_line = Line {
                points: vec![
                    (
                        Point::new(Mm(table_left), Mm(*ctx.cursor_y - row_height_mm)),
                        false,
                    ),
                    (
                        Point::new(Mm(table_left + table_width), Mm(*ctx.cursor_y - row_height_mm)),
                        false,
                    ),
                ],
                is_closed: false,
            };
            ctx.current_layer.add_line(border_line);
        }

        for (col_idx, cell) in cells.iter().enumerate() {
            let lines = &row_cell_lines[col_idx];
            let cell_align = cell
                .attributes
                .get("align")
                .map(|s| s.as_str())
                .or_else(|| {
                    cell.children.iter().find_map(|c| {
                        if let Node::Element(p) = c {
                            p.attributes.get("align").map(|s| s.as_str())
                        } else {
                            None
                        }
                    })
                });

            let font_size = 9.0;
            let start_y = *ctx.cursor_y - 4.5;
            let base_cell_x = table_left + (col_idx as f32) * col_width;

            for (line_idx, line_txt) in lines.iter().enumerate() {
                let y_pos = start_y - (line_idx as f32) * 4.2;
                let text_w_mm = (line_txt.len() as f32) * font_size * 0.18;
                let cell_x = match cell_align {
                    Some("center") => base_cell_x + ((col_width - text_w_mm) / 2.0).max(1.5),
                    Some("right") => base_cell_x + (col_width - text_w_mm - 2.0).max(1.5),
                    _ => base_cell_x + 2.0,
                };

                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
                ctx.current_layer.use_text(line_txt, font_size, Mm(cell_x), Mm(y_pos), font);
            }
        }

        *ctx.cursor_y -= row_height_mm;
    }

    *ctx.cursor_y -= 4.0;
}

pub fn collect_rows<'a>(el: &'a ElementNode, rows: &mut Vec<&'a ElementNode>) {
    for child in &el.children {
        if let Node::Element(child_el) = child {
            let tag = child_el.tag.to_lowercase();
            if tag == "tr" {
                rows.push(child_el);
            } else if tag == "thead" || tag == "tbody" || tag == "tfoot" {
                collect_rows(child_el, rows);
            }
        }
    }
}
