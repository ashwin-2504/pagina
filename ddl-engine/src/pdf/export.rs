use crate::ast::{Document, Node, ElementNode};
use printpdf::*;
use std::io::Cursor;

pub fn export_pdf(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let title = doc.props.metadata.get("title").map(|s| s.as_str()).unwrap_or("Pagina Document");
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let (width_mm, height_mm) = if is_landscape { (297.0, 210.0) } else { (210.0, 297.0) };

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

    let left_margin = 20.0;
    let right_margin = 20.0;
    let top_margin = 25.0;
    let bottom_margin = 20.0;
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
                match tag.as_str() {
                    "h1" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 4.0;
                        render_heading(
                            &text,
                            22.0,
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
    font: &IndirectFontRef,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    cursor_y: &mut f32,
    left_margin: f32,
    _content_width: f32,
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

    current_layer.use_text(text, font_size, Mm(left_margin), Mm(*cursor_y), font);
    *cursor_y -= line_height_mm;
}

fn render_paragraph(
    text: &str,
    font_size: f32,
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
    // Word-wrap approximately: ~80 chars per line for standard width
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
            current_layer.use_text(&current_line, font_size, Mm(left_margin), Mm(*cursor_y), font);
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
        current_layer.use_text(&current_line, font_size, Mm(left_margin), Mm(*cursor_y), font);
        *cursor_y -= line_height_mm;
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
