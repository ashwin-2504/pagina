use crate::ast::{Document, ElementNode, Node, PageStyle};
use printpdf::path::PaintMode;
use printpdf::*;
use std::io::Cursor;

#[derive(Debug, Clone)]
struct StyledRun {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

#[derive(Debug, Clone)]
struct ParagraphWord {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
    has_trailing_space: bool,
    width_mm: f32,
}

#[derive(Debug, Clone)]
struct LineSpan {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
    font_size: f32,
    width_mm: f32,
}

pub fn export_pdf(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let title = doc
        .props
        .metadata
        .get("title")
        .map(|s| s.as_str())
        .unwrap_or("Pagina Document");

    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let (mut width_mm, mut height_mm) = if is_letter {
        (215.9, 279.4)
    } else {
        (210.0, 297.0)
    };
    if is_landscape {
        std::mem::swap(&mut width_mm, &mut height_mm);
    }

    let (pdf_doc, first_page, first_layer) =
        PdfDocument::new(title, Mm(width_mm), Mm(height_mm), "Content");

    let font_regular = pdf_doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = pdf_doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let font_italic = pdf_doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;
    let font_code = pdf_doc.add_builtin_font(BuiltinFont::Courier)?;

    let mut current_page = first_page;
    let mut current_layer = pdf_doc.get_page(current_page).get_layer(first_layer);
    let mut page_count = 1;

    // Dynamic margin from document props
    let margin_mm = doc.props.paper.margin.val;
    let left_margin = margin_mm.max(12.0);
    let right_margin = margin_mm.max(12.0);
    let top_margin = (margin_mm * 1.1).max(22.0);
    let bottom_margin = (margin_mm * 1.0).max(18.0);
    let content_width = width_mm - left_margin - right_margin;

    // Draw header and footer on page 1
    draw_running_header_footer(
        &current_layer,
        1,
        left_margin,
        content_width,
        width_mm,
        height_mm,
        &font_regular,
        doc,
    );

    let mut cursor_y = height_mm - top_margin;

    // Recursively render nodes
    render_doc_nodes(
        &doc.body.children,
        &pdf_doc,
        &mut current_page,
        &mut current_layer,
        &mut page_count,
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
        doc,
    );

    let mut buf = std::io::BufWriter::new(Cursor::new(Vec::new()));
    pdf_doc.save(&mut buf)?;
    Ok(buf.into_inner()?.into_inner())
}

fn get_active_page_style(doc: &Document) -> Option<&PageStyle> {
    if let Some(ps) = doc.props.page_styles.get("body") {
        return Some(ps);
    }
    doc.props.page_styles.values().find(|ps| ps.is_default).or_else(|| {
        doc.props.page_styles.values().next()
    })
}

fn extract_nodes_text(nodes: &[Node]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            Node::Text(t) => s.push_str(&t.content),
            Node::Element(el) => s.push_str(&extract_nodes_text(&el.children)),
            _ => {}
        }
    }
    s.trim().to_string()
}

fn draw_running_header_footer(
    layer: &PdfLayerReference,
    page_num: usize,
    left_margin: f32,
    content_width: f32,
    _width_mm: f32,
    height_mm: f32,
    font_reg: &IndirectFontRef,
    doc: &Document,
) {
    let ps = get_active_page_style(doc);
    let left_text = ps
        .and_then(|p| p.running_header.as_ref())
        .map(|h| extract_nodes_text(&h.left))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            doc.props
                .metadata
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Technical Report".to_string())
        });

    let right_text = format!("{} of 1", page_num);

    let footer_text = ps
        .and_then(|p| p.running_footer.as_ref())
        .map(|f| extract_nodes_text(&f.center))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Pagina Document Typesetting System".to_string());

    // Color: #64748B (Slate Gray)
    layer.set_fill_color(Color::Rgb(Rgb::new(
        100.0 / 255.0,
        116.0 / 255.0,
        139.0 / 255.0,
        None,
    )));

    // Running Header text
    let header_y = height_mm - 11.5;
    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(header_y), font_reg);

    let right_w = (right_text.len() as f32) * 8.5 * 0.19;
    let right_x = (left_margin + content_width - right_w).max(left_margin);
    layer.use_text(&right_text, 8.5, Mm(right_x), Mm(header_y), font_reg);

    // Running Header thin rule line (#CBD5E1)
    let rule_y = height_mm - 14.0;
    layer.set_outline_color(Color::Rgb(Rgb::new(
        203.0 / 255.0,
        213.0 / 255.0,
        225.0 / 255.0,
        None,
    )));
    layer.set_outline_thickness(0.5);
    let points = vec![
        (Point::new(Mm(left_margin), Mm(rule_y)), false),
        (Point::new(Mm(left_margin + content_width), Mm(rule_y)), false),
    ];
    layer.add_line(Line {
        points,
        is_closed: false,
    });

    // Running Footer centered at bottom
    let footer_y = 10.5;
    let footer_w = (footer_text.len() as f32) * 8.5 * 0.19;
    let footer_x = (left_margin + (content_width - footer_w) / 2.0).max(left_margin);
    layer.use_text(&footer_text, 8.5, Mm(footer_x), Mm(footer_y), font_reg);

    // Reset fill color to Dark Slate #0f172a
    layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
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

fn collect_styled_runs(node: &Node, bold: bool, italic: bool, code: bool, runs: &mut Vec<StyledRun>) {
    match node {
        Node::Text(t) => {
            runs.push(StyledRun {
                text: t.content.clone(),
                bold,
                italic,
                code,
            });
        }
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            let mut b = bold;
            let mut i = italic;
            let mut c = code;
            if tag == "b" || tag == "strong" {
                b = true;
            } else if tag == "i" || tag == "em" {
                i = true;
            } else if tag == "code" {
                c = true;
            } else if tag == "br" {
                runs.push(StyledRun {
                    text: "\n".to_string(),
                    bold: b,
                    italic: i,
                    code: c,
                });
                return;
            }
            for child in &el.children {
                collect_styled_runs(child, b, i, c, runs);
            }
        }
        _ => {}
    }
}

fn render_doc_nodes(
    nodes: &[Node],
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    page_count: &mut usize,
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
    doc: &Document,
) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                let trimmed = t.content.trim();
                if !trimmed.is_empty() {
                    let run = StyledRun {
                        text: trimmed.to_string(),
                        bold: false,
                        italic: false,
                        code: false,
                    };
                    render_styled_paragraph(
                        &[run],
                        10.5,
                        None,
                        None,
                        false,
                        pdf_doc,
                        current_page,
                        current_layer,
                        page_count,
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
                        doc,
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
                    page_count,
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
                    doc,
                );
            }
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                let align = get_align(el);
                let is_callout = tag == "callout" || el.classes.iter().any(|c| c == "callout");

                if is_callout {
                    let mut runs = Vec::new();
                    for child in &el.children {
                        collect_styled_runs(child, false, false, false, &mut runs);
                    }
                    *cursor_y -= 2.0;
                    render_styled_paragraph(
                        &runs,
                        10.0,
                        None,
                        None,
                        true,
                        pdf_doc,
                        current_page,
                        current_layer,
                        page_count,
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
                        doc,
                    );
                    *cursor_y -= 4.0;
                    continue;
                }

                match tag.as_str() {
                    "h1" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 3.0;
                        render_heading(
                            &text,
                            22.0,
                            align,
                            Some((30.0 / 255.0, 58.0 / 255.0, 138.0 / 255.0)), // #1e3a8a Dark Blue
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            doc,
                        );
                        *cursor_y -= 4.0;
                    }
                    "h2" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 4.0;
                        render_heading(
                            &text,
                            15.0,
                            align,
                            Some((15.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0)), // #0f172a
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            doc,
                        );
                        *cursor_y -= 3.0;
                    }
                    "h3" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 2.0;
                        render_heading(
                            &text,
                            11.5,
                            align,
                            Some((30.0 / 255.0, 41.0 / 255.0, 59.0 / 255.0)), // #1e293b
                            font_bold,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            doc,
                        );
                        *cursor_y -= 2.0;
                    }
                    "p" => {
                        let is_lead = el.classes.iter().any(|c| c == "lead");
                        let p_font_size = if is_lead { 11.5 } else { 10.5 };
                        let p_color = if is_lead {
                            Some((71.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0)) // #475569
                        } else {
                            Some((51.0 / 255.0, 65.0 / 255.0, 85.0 / 255.0)) // #334155
                        };

                        let mut runs = Vec::new();
                        for child in &el.children {
                            collect_styled_runs(child, false, false, false, &mut runs);
                        }
                        render_styled_paragraph(
                            &runs,
                            p_font_size,
                            align,
                            p_color,
                            false,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
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
                            doc,
                        );
                        *cursor_y -= 3.5;
                    }
                    "grid" => {
                        render_grid(
                            el,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
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
                            doc,
                        );
                    }
                    "ul" | "ol" => {
                        for (idx, child) in el.children.iter().enumerate() {
                            if let Node::Element(li) = child {
                                if li.tag.to_lowercase() == "li" {
                                    let mut runs = Vec::new();
                                    let bullet = if tag == "ul" {
                                        "- ".to_string()
                                    } else {
                                        format!("{}. ", idx + 1)
                                    };
                                    runs.push(StyledRun {
                                        text: bullet,
                                        bold: true,
                                        italic: false,
                                        code: false,
                                    });
                                    for li_child in &li.children {
                                        collect_styled_runs(li_child, false, false, false, &mut runs);
                                    }
                                    render_styled_paragraph(
                                        &runs,
                                        10.5,
                                        None,
                                        None,
                                        false,
                                        pdf_doc,
                                        current_page,
                                        current_layer,
                                        page_count,
                                        cursor_y,
                                        left_margin + 5.0,
                                        content_width - 5.0,
                                        height_mm,
                                        top_margin,
                                        bottom_margin,
                                        width_mm,
                                        font_reg,
                                        font_bold,
                                        font_italic,
                                        font_code,
                                        doc,
                                    );
                                    *cursor_y -= 1.5;
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
                            page_count,
                            cursor_y,
                            left_margin,
                            content_width,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            font_reg,
                            font_bold,
                            doc,
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
                        // General container: div, col-box, etc.
                        render_doc_nodes(
                            &el.children,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
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
                            doc,
                        );
                    }
                }
            }
        }
    }
}

fn char_width_helvetica(ch: char) -> f32 {
    let w = match ch {
        ' ' => 278.0,
        '!' | ',' | '.' | ':' | ';' | 'i' | 'l' | '\'' | '`' => 278.0,
        'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
        'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' | 'z' | '0'..='9' => 556.0,
        's' => 500.0,
        'w' => 722.0,
        'm' => 833.0,
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
        'I' => 278.0,
        'J' => 500.0,
        'L' => 556.0,
        'M' | 'W' => 833.0,
        _ => 556.0,
    };
    w / 1000.0
}

fn measure_text(text: &str, font_size: f32, is_code: bool) -> f32 {
    let pt_to_mm = 0.352778;
    if is_code {
        (text.chars().count() as f32) * 0.60 * font_size * pt_to_mm
    } else {
        let em_sum: f32 = text.chars().map(char_width_helvetica).sum();
        em_sum * font_size * pt_to_mm
    }
}

fn estimate_nodes_height(nodes: &[Node], width: f32) -> f32 {
    let mut h = 0.0f32;
    for node in nodes {
        match node {
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                match tag.as_str() {
                    "h1" => h += 14.0,
                    "h2" => h += 10.0,
                    "h3" => h += 8.0,
                    "p" => {
                        let text = extract_plain_text(el);
                        let chars = text.chars().count();
                        let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                        h += est_lines * 5.0 + 4.0;
                    }
                    _ => h += estimate_nodes_height(&el.children, width),
                }
            }
            Node::Text(t) => {
                let chars = t.content.trim().chars().count();
                if chars > 0 {
                    let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                    h += est_lines * 5.0;
                }
            }
            _ => {}
        }
    }
    h
}

fn render_grid(
    el: &ElementNode,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    page_count: &mut usize,
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
    doc: &Document,
) {
    let col_divs: Vec<&ElementNode> = el
        .children
        .iter()
        .filter_map(|c| {
            if let Node::Element(child_el) = c {
                Some(child_el)
            } else {
                None
            }
        })
        .collect();

    if col_divs.is_empty() {
        return;
    }

    let n = col_divs.len();
    let gap_mm = 6.0f32;
    let total_gap = (n - 1) as f32 * gap_mm;
    let col_width = (content_width - total_gap) / (n as f32);

    let start_y = *cursor_y;
    let inner_padding = 4.0f32;

    // Estimate maximum height across columns
    let max_col_height = col_divs
        .iter()
        .map(|c| estimate_nodes_height(&c.children, col_width - (inner_padding * 2.0)))
        .fold(0.0f32, f32::max)
        .max(22.0)
        + (inner_padding * 2.0);

    let uniform_bottom = start_y - max_col_height;

    // Check page overflow
    if uniform_bottom < bottom_margin {
        *page_count += 1;
        let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
        *current_page = new_page;
        *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
        draw_running_header_footer(
            current_layer,
            *page_count,
            left_margin,
            content_width,
            width_mm,
            height_mm,
            font_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    let actual_start_y = *cursor_y;
    let actual_uniform_bottom = actual_start_y - max_col_height;

    // Draw card background & border ONCE
    for (i, col_el) in col_divs.iter().enumerate() {
        let is_boxed = col_el.classes.iter().any(|c| c == "col-box");
        if is_boxed {
            let col_left = left_margin + (i as f32) * (col_width + gap_mm);
            current_layer.set_fill_color(Color::Rgb(Rgb::new(
                248.0 / 255.0,
                250.0 / 255.0,
                252.0 / 255.0,
                None,
            )));
            current_layer.set_outline_color(Color::Rgb(Rgb::new(
                226.0 / 255.0,
                232.0 / 255.0,
                240.0 / 255.0,
                None,
            )));
            current_layer.set_outline_thickness(0.6);

            let mut card_rect = Rect::new(
                Mm(col_left),
                Mm(actual_uniform_bottom),
                Mm(col_left + col_width),
                Mm(actual_start_y),
            );
            card_rect.mode = PaintMode::FillStroke;
            current_layer.add_rect(card_rect);
        }
    }

    // Render text inside cards ONCE
    for (i, col_el) in col_divs.iter().enumerate() {
        let is_boxed = col_el.classes.iter().any(|c| c == "col-box");
        let col_left = left_margin + (i as f32) * (col_width + gap_mm);
        let pad = if is_boxed { inner_padding } else { 0.0 };
        let mut col_cursor = actual_start_y - pad - 1.5;

        render_doc_nodes(
            &col_el.children,
            pdf_doc,
            current_page,
            current_layer,
            page_count,
            &mut col_cursor,
            col_left + pad,
            col_width - (pad * 2.0),
            height_mm,
            top_margin,
            bottom_margin,
            width_mm,
            font_reg,
            font_bold,
            font_italic,
            font_code,
            doc,
        );
    }

    *cursor_y = actual_uniform_bottom - 6.0;
}

fn render_heading(
    text: &str,
    font_size: f32,
    align: Option<&'static str>,
    color: Option<(f32, f32, f32)>,
    font: &IndirectFontRef,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    page_count: &mut usize,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
    font_reg: &IndirectFontRef,
    doc: &Document,
) {
    let line_height_mm = font_size * 0.45;
    if *cursor_y - line_height_mm < bottom_margin {
        *page_count += 1;
        let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
        *current_page = new_page;
        *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
        draw_running_header_footer(
            current_layer,
            *page_count,
            left_margin,
            content_width,
            width_mm,
            height_mm,
            font_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    if let Some((r, g, b)) = color {
        current_layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));
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

    // Reset color to dark slate
    current_layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}

fn render_styled_paragraph(
    runs: &[StyledRun],
    font_size: f32,
    align: Option<&'static str>,
    color: Option<(f32, f32, f32)>,
    is_callout: bool,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    page_count: &mut usize,
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
    doc: &Document,
) {
    let line_height_mm = font_size * 0.44;
    let actual_left = if is_callout {
        left_margin + 6.0
    } else {
        left_margin
    };
    let actual_width = if is_callout {
        content_width - 8.0
    } else {
        content_width
    };

    // Clean whitespace from runs
    let mut cleaned_runs: Vec<StyledRun> = Vec::new();
    let num_runs = runs.len();

    for (idx, run) in runs.iter().enumerate() {
        let mut text = run.text.clone();
        if idx == 0 {
            text = text.trim_start().to_string();
        }
        if idx == num_runs - 1 {
            text = text.trim_end().to_string();
        }
        if !text.is_empty() {
            cleaned_runs.push(StyledRun {
                text,
                bold: run.bold,
                italic: run.italic,
                code: run.code,
            });
        }
    }

    if cleaned_runs.is_empty() {
        return;
    }

    // Tokenize into words with exact font metrics
    let mut words: Vec<ParagraphWord> = Vec::new();
    let space_w = measure_text(" ", font_size, false);

    for run in &cleaned_runs {
        let is_code = run.code;
        let mut cur_word = String::new();
        let chars: Vec<char> = run.text.chars().collect();
        let n = chars.len();
        let mut i = 0;

        while i < n {
            let ch = chars[i];
            if ch == '\n' {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, is_code);
                    words.push(ParagraphWord {
                        text: cur_word,
                        bold: run.bold,
                        italic: run.italic,
                        code: run.code,
                        has_trailing_space: false,
                        width_mm: w,
                    });
                    cur_word = String::new();
                }
                words.push(ParagraphWord {
                    text: "\n".to_string(),
                    bold: run.bold,
                    italic: run.italic,
                    code: run.code,
                    has_trailing_space: false,
                    width_mm: 0.0,
                });
                i += 1;
            } else if ch.is_whitespace() {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, is_code);
                    words.push(ParagraphWord {
                        text: cur_word,
                        bold: run.bold,
                        italic: run.italic,
                        code: run.code,
                        has_trailing_space: true,
                        width_mm: w,
                    });
                    cur_word = String::new();
                } else if let Some(last) = words.last_mut() {
                    last.has_trailing_space = true;
                }
                while i < n && chars[i].is_whitespace() && chars[i] != '\n' {
                    i += 1;
                }
            } else {
                cur_word.push(ch);
                i += 1;
            }
        }

        if !cur_word.is_empty() {
            let w = measure_text(&cur_word, font_size, is_code);
            words.push(ParagraphWord {
                text: cur_word,
                bold: run.bold,
                italic: run.italic,
                code: run.code,
                has_trailing_space: false,
                width_mm: w,
            });
        }
    }

    if words.is_empty() {
        return;
    }

    // Wrap words into lines
    let mut lines: Vec<Vec<ParagraphWord>> = Vec::new();
    let mut cur_line: Vec<ParagraphWord> = Vec::new();
    let mut cur_line_width = 0.0f32;

    for word in words {
        if word.text == "\n" {
            lines.push(cur_line);
            cur_line = Vec::new();
            cur_line_width = 0.0;
            continue;
        }

        let needed = word.width_mm + if word.has_trailing_space { space_w } else { 0.0 };
        if !cur_line.is_empty() && cur_line_width + word.width_mm > actual_width {
            lines.push(cur_line);
            cur_line = Vec::new();
            cur_line_width = 0.0;
        }

        cur_line_width += needed;
        cur_line.push(word);
    }

    if !cur_line.is_empty() {
        lines.push(cur_line);
    }

    // Combine contiguous words with the same style into spans for natural text kerning
    let mut rendered_lines: Vec<(Vec<LineSpan>, f32)> = Vec::new();

    for line_words in lines {
        if line_words.is_empty() {
            continue;
        }

        let mut spans: Vec<LineSpan> = Vec::new();
        let mut cur_span_text = String::new();
        let mut cur_bold = false;
        let mut cur_italic = false;
        let mut cur_code = false;
        let num_words = line_words.len();

        for (idx, w) in line_words.iter().enumerate() {
            let is_last_word = idx == num_words - 1;
            let style_matches = !cur_span_text.is_empty()
                && cur_bold == w.bold
                && cur_italic == w.italic
                && cur_code == w.code;

            if style_matches {
                cur_span_text.push_str(&w.text);
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
            } else {
                if !cur_span_text.is_empty() {
                    let span_w = measure_text(&cur_span_text, font_size, cur_code);
                    spans.push(LineSpan {
                        text: cur_span_text,
                        bold: cur_bold,
                        italic: cur_italic,
                        code: cur_code,
                        font_size,
                        width_mm: span_w,
                    });
                }
                cur_span_text = w.text.clone();
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
                cur_bold = w.bold;
                cur_italic = w.italic;
                cur_code = w.code;
            }
        }

        if !cur_span_text.is_empty() {
            let span_w = measure_text(&cur_span_text, font_size, cur_code);
            spans.push(LineSpan {
                text: cur_span_text,
                bold: cur_bold,
                italic: cur_italic,
                code: cur_code,
                font_size,
                width_mm: span_w,
            });
        }

        let total_line_w: f32 = spans.iter().map(|s| s.width_mm).sum();
        rendered_lines.push((spans, total_line_w));
    }

    if rendered_lines.is_empty() {
        return;
    }

    let total_paragraph_height = (rendered_lines.len() as f32) * line_height_mm;

    // Check page overflow
    if *cursor_y - total_paragraph_height < bottom_margin {
        *page_count += 1;
        let (new_page, new_layer) = pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
        *current_page = new_page;
        *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
        draw_running_header_footer(
            current_layer,
            *page_count,
            left_margin,
            content_width,
            width_mm,
            height_mm,
            font_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    // Draw Callout filled background (#F0FDF4) and left green accent bar (#16A34A)
    if is_callout {
        let callout_top_y = *cursor_y + 2.5;
        let callout_bottom_y = *cursor_y - total_paragraph_height - 1.5;

        current_layer.set_fill_color(Color::Rgb(Rgb::new(
            240.0 / 255.0,
            253.0 / 255.0,
            244.0 / 255.0,
            None,
        )));
        let mut bg_rect = Rect::new(
            Mm(left_margin),
            Mm(callout_bottom_y),
            Mm(left_margin + content_width),
            Mm(callout_top_y),
        );
        bg_rect.mode = PaintMode::Fill;
        current_layer.add_rect(bg_rect);

        current_layer.set_fill_color(Color::Rgb(Rgb::new(
            22.0 / 255.0,
            163.0 / 255.0,
            74.0 / 255.0,
            None,
        )));
        let mut bar_rect = Rect::new(
            Mm(left_margin),
            Mm(callout_bottom_y),
            Mm(left_margin + 1.8),
            Mm(callout_top_y),
        );
        bar_rect.mode = PaintMode::Fill;
        current_layer.add_rect(bar_rect);
    }

    // Render lines of text
    for (spans, line_width) in rendered_lines {
        let mut x = actual_left;
        if align == Some("center") {
            x = (actual_left + (actual_width - line_width) / 2.0).max(actual_left);
        } else if align == Some("right") {
            x = (actual_left + actual_width - line_width).max(actual_left);
        }

        for span in &spans {
            let font = if span.code {
                font_code
            } else if span.bold {
                font_bold
            } else if span.italic {
                font_italic
            } else {
                font_reg
            };

            if span.code {
                // Subtle code pill background
                let pill_pad_x = 0.8;
                let pill_pad_y = 0.4;
                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    241.0 / 255.0,
                    245.0 / 255.0,
                    249.0 / 255.0,
                    None,
                )));
                let mut pill = Rect::new(
                    Mm(x - pill_pad_x),
                    Mm(*cursor_y - pill_pad_y),
                    Mm(x + span.width_mm + pill_pad_x),
                    Mm(*cursor_y + span.font_size * 0.35 + pill_pad_y),
                );
                pill.mode = PaintMode::Fill;
                current_layer.add_rect(pill);
            }

            if span.code {
                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
            } else if let Some((r, g, b)) = color {
                current_layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));
            } else {
                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
            }

            current_layer.use_text(&span.text, span.font_size, Mm(x), Mm(*cursor_y), font);
            x += span.width_mm;
        }

        *cursor_y -= line_height_mm;
    }

    // Reset color to dark slate
    current_layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}

fn render_table(
    el: &ElementNode,
    pdf_doc: &PdfDocumentReference,
    current_page: &mut PdfPageIndex,
    current_layer: &mut PdfLayerReference,
    page_count: &mut usize,
    cursor_y: &mut f32,
    left_margin: f32,
    content_width: f32,
    height_mm: f32,
    top_margin: f32,
    bottom_margin: f32,
    width_mm: f32,
    font_reg: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    doc: &Document,
) {
    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);
    if rows.is_empty() {
        return;
    }

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

    let col_width = content_width / (max_cols as f32);
    *cursor_y -= 2.0;

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

        let is_header_row = cells.iter().any(|c| c.tag.to_lowercase() == "th");
        let font = if is_header_row { font_bold } else { font_reg };
        let row_height_mm = 8.0f32;

        if *cursor_y - row_height_mm < bottom_margin {
            *page_count += 1;
            let (new_page, new_layer) =
                pdf_doc.add_page(Mm(width_mm), Mm(height_mm), "Content");
            *current_page = new_page;
            *current_layer = pdf_doc.get_page(new_page).get_layer(new_layer);
            draw_running_header_footer(
                current_layer,
                *page_count,
                left_margin,
                content_width,
                width_mm,
                height_mm,
                font_reg,
                doc,
            );
            *cursor_y = height_mm - top_margin;
        }

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
