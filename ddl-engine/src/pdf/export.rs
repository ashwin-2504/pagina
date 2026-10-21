use crate::ast::{Document, ElementNode, Node};
use printpdf::*;
use std::io::Cursor;

pub fn export_pdf(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let title = doc.props.metadata.get("title").map(|s| s.as_str()).unwrap_or("Pagina Document");
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let (mut width_mm, mut height_mm) = if is_letter { (215.9, 279.4) } else { (210.0, 297.0) };
    if is_landscape {
        std::mem::swap(&mut width_mm, &mut height_mm);
    }

    let (pdf_doc, first_page, first_layer) = PdfDocument::new(
        title,
        Mm(width_mm),
        Mm(height_mm),
        "Content",
    );

    let font_regular = pdf_doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = pdf_doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_italic = pdf_doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;
    let font_code = pdf_doc.add_builtin_font(BuiltinFont::Courier)?;

    let mut current_page = first_page;
    let mut current_layer = pdf_doc.get_page(current_page).get_layer(first_layer);

    // Dynamic margin from document props
    let margin_mm = doc.props.paper.margin.val;
    let left_margin = margin_mm.max(10.0);
    let right_margin = margin_mm.max(10.0);
    let top_margin = (margin_mm * 1.1).max(12.0);
    let bottom_margin = margin_mm.max(10.0);
    let content_width = width_mm - left_margin - right_margin;

    let mut cursor_y = height_mm - top_margin;

    // Recursively render nodes
    render_doc_nodes(
        &doc.body.children,
        &pdf_doc,
        &mut current_page,
        &mut current_layer,
        &mut cursor_y,
        left_margin,
        content_width,
        height_mm,
        top_margin,
        bottom_margin,
        width_mm,
        &font_regular,
        &font_bold,
        &font_italic,
        &font_code,
    );

    let mut buf = std::io::BufWriter::new(Cursor::new(Vec::new()));
    pdf_doc.save(&mut buf)?;
    Ok(buf.into_inner()?.into_inner())
}

fn get_align(el: &ElementNode) -> Option<&'static str> {
    if let Some(a) = el.attributes.get("align") {
        if a.eq_ignore_ascii_case("center") {
            return Some("center");
        } else if a.eq_ignore_ascii_case("right") {
            return Some("right");
        }
    }
    if el.classes.iter().any(|c| c == "text-center") {
        return Some("center");
    }
    if el.classes.iter().any(|c| c == "text-right") {
        return Some("right");
    }
    if let Some(style) = el.inline_styles.get("text-align") {
        if style.eq_ignore_ascii_case("center") {
            return Some("center");
        } else if style.eq_ignore_ascii_case("right") {
            return Some("right");
        }
    }
    None
}

fn render_doc_nodes(
    nodes: &[Node],
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
    font_reg: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    font_italic: &IndirectFontRef,
    font_code: &IndirectFontRef,
) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                let text = t.content.trim();
                if !text.is_empty() {
                    render_paragraph(
                        text,
                        10.5,
                        None,
                        font_reg,
                        pdf_doc,
                        current_page,
                        current_layer,
                        cursor_y,
                        left_margin,
                        content_width,
                        height_mm,
                        top_margin,
                        bottom_margin,
                        width_mm,
                    );
                }
            }
            Node::Comment(_) => {}
            Node::Section(sec) => {
                render_doc_nodes(
                    &sec.children,
                    pdf_doc,
                    current_page,
                    current_layer,
                    cursor_y,
                    left_margin,
                    content_width,
                    height_mm,
                    top_margin,
                    bottom_margin,
                    width_mm,
                    font_reg,
                    font_bold,
                    font_italic,
                    font_code,
                );
            }
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                let align = get_align(el);
                match tag.as_str() {
                    "h1" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 4.0;
                        render_heading(
                            &text,
                            22.0,
                            align,
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                        );
                        *cursor_y -= 3.0;
                    }
                    "h2" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 3.0;
                        render_heading(
                            &text,
                            16.0,
                            align,
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                        );
                        *cursor_y -= 2.0;
                    }
                    "h3" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 2.0;
                        render_heading(
                            &text,
                            13.0,
                            align,
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                        );
                        *cursor_y -= 2.0;
                    }
                    "p" => {
                        let text = extract_plain_text(el);
                        render_paragraph(
                            &text,
                            10.5,
                            align,
                            font_reg,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                        );
                        *cursor_y -= 3.0;
                    }
                    "callout" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 2.0;
                        render_paragraph(
                            &format!("| {}", text),
                            10.5,
                            None,
                            font_italic,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin + 4.0,
                            content_width - 8.0,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                        );
                        *cursor_y -= 2.0;
                    }
                    "ul" | "ol" => {
                        for child in &el.children {
                            if let Node::Element(li) = child {
                                if li.tag.to_lowercase() == "li" {
                                    let text = extract_plain_text(li);
                                    let bullet = if tag == "ul" { "- " } else { "1. " };
                                    render_paragraph(
                                        &format!("{}{}", bullet, text),
                                        10.5,
                                        None,
                                        font_reg,
                                        pdf_doc,
                                        current_page,
                                        current_layer,
                                        cursor_y,
                                        left_margin + 6.0,
                                        content_width - 6.0,
                                        height_mm,
                                        top_margin,
                                        bottom_margin,
                                        width_mm,
                                    );
                                }
                            }
                        }
                        *cursor_y -= 2.0;
                    }
                    "table" => {
                        render_table(
                            el,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            font_bold,
                        );
                    }
                    "hr" => {
                        *cursor_y -= 3.0;
                        let points = vec![
                            (Point::new(Mm(left_margin), Mm(*cursor_y)), false),
                            (Point::new(Mm(left_margin + content_width), Mm(*cursor_y)), false),
                        ];
                        let line = Line {
                            points,
                            is_closed: false,
                        };
                        current_layer.add_line(line);
                        *cursor_y -= 4.0;
                    }
                    _ => {
                        // General container: div, grid, etc.
                        render_doc_nodes(
                            &el.children,
                            pdf_doc,
                            current_page,
                            current_layer,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            font_bold,
                            font_italic,
                            font_code,
                        );
                    }
                }
            }
        }
    }
}

fn render_heading(
    text: &str,
    font_size: f32,
    align: Option<&'static str>,
    font: &IndirectFontRef,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
) {
    let line_height_mm = font_size * 0.45;
    if *cursor_y - line_height_mm < bottom_margin {
        let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
        *current_page = new_page;
        *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
        *cursor_y = height_mm - top_margin;
    }

    let mut x = left_margin;
    if align == Some("center") {
        let text_width_approx = (text.len() as f32) * font_size * 0.20;
        x = (left_margin + (content_width - text_width_approx) / 2.0).max(left_margin);
    } else if align == Some("right") {
        let text_width_approx = (text.len() as f32) * font_size * 0.20;
        x = (left_margin + content_width - text_width_approx).max(left_margin);
    }

    current_layer.use_text(text, font_size, Mm(x), Mm(*cursor_y), font);
    *cursor_y -= line_height_mm;
}

fn render_paragraph(
    text: &str,
    font_size: f32,
    align: Option<&'static str>,
    font: &IndirectFontRef,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
) {
    let line_height_mm = font_size * 0.42;
    let chars_per_line = (content_width * 2.2).max(30.0) as usize;

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > chars_per_line && !current_line.is_empty() {
            if *cursor_y - line_height_mm < bottom_margin {
                let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
                *current_page = new_page;
                *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
                *cursor_y = height_mm - top_margin;
            }
            let mut x = left_margin;
            if align == Some("center") {
                let line_w = (current_line.len() as f32) * font_size * 0.20;
                x = (left_margin + (content_width - line_w) / 2.0).max(left_margin);
            }
            current_layer.use_text(&current_line, font_size, Mm(x), Mm(*cursor_y), font);
            *cursor_y -= line_height_mm;
            current_line.clear();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        if *cursor_y - line_height_mm < bottom_margin {
            let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
            *current_page = new_page;
            *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
            *cursor_y = height_mm - top_margin;
        }
        let mut x = left_margin;
        if align == Some("center") {
            let line_w = (current_line.len() as f32) * font_size * 0.20;
            x = (left_margin + (content_width - line_w) / 2.0).max(left_margin);
        }
        current_layer.use_text(&current_line, font_size, Mm(x), Mm(*cursor_y), font);
        *cursor_y -= line_height_mm;
    }
}

fn render_table(
    el: &ElementNode,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
    font_reg: &IndirectFontRef,
    font_bold: &IndirectFontRef,
) {
    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);
    if rows.is_empty() {
        return;
    }

    // Determine max columns
    let mut max_cols = 1;
    for r in &rows {
        let cell_count = r.children.iter().filter(|c| {
            if let Node::Element(cell) = c {
                let tag = cell.tag.to_lowercase();
                tag == "td" || tag == "th"
            } else {
                false
            }
        }).count();
        if cell_count > max_cols {
            max_cols = cell_count;
        }
    }

    let col_width = content_width / (max_cols as f32);
    *cursor_y -= 2.0;

    for row in rows {
        let cells: Vec<&ElementNode> = row.children.iter().filter_map(|c| {
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
        }).collect();

        if cells.is_empty() {
            continue;
        }

        let is_header_row = cells.iter().any(|c| c.tag.to_lowercase() == "th");
        let font = if is_header_row { font_bold } else { font_reg };
        let row_height_mm = 8.0f32;

        if *cursor_y - row_height_mm < bottom_margin {
            let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
            *current_page = new_page;
            *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
            *cursor_y = height_mm - top_margin;
        }

        // Render each cell
        for (col_idx, cell) in cells.iter().enumerate() {
            let cell_x = left_margin + (col_idx as f32) * col_width + 1.5;
            let text = extract_plain_text(cell);
            let max_chars = ((col_width - 3.0) * 2.2).max(10.0) as usize;
            let display_text = if text.len() > max_chars {
                format!("{}...", &text[..max_chars.saturating_sub(3)])
            } else {
                text
            };
            current_layer.use_text(&display_text, 9.5, Mm(cell_x), Mm(*cursor_y - 4.5), font);
        }

        *cursor_y -= row_height_mm;
    }

    *cursor_y -= 3.0;
}

fn collect_rows<'a>(el: &'a ElementNode, rows: &mut Vec<&'a ElementNode>) {
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

fn extract_plain_text(el: &ElementNode) -> String {
    let mut s = String::new();
    for child in &el.children {
        match child {
            Node::Text(t) => s.push_str(&t.content),
            Node::Element(sub) => s.push_str(&extract_plain_text(sub)),
            _ => {}
        }
    }
    s
}
