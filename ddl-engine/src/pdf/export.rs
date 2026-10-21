use crate::ast::{Document, ElementNode, Node, PageStyle};
use printpdf::*;
use std::io::Cursor;

#[derive(Debug, Clone)]
struct StyledRun {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

struct WordToken<'a> {
    text: &'a str,
    font: &'a IndirectFontRef,
    font_size: f32,
    width_mm: f32,
    is_space: bool,
    is_newline: bool,
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
    let left_margin = margin_mm.max(10.0);
    let right_margin = margin_mm.max(10.0);
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
    let header_y = height_mm - 12.0;
    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(header_y), font_reg);

    let right_w = (right_text.len() as f32) * 8.5 * 0.19;
    let right_x = (left_margin + content_width - right_w).max(left_margin);
    layer.use_text(&right_text, 8.5, Mm(right_x), Mm(header_y), font_reg);

    // Running Header thin rule line (#CBD5E1)
    let rule_y = height_mm - 14.5;
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
    let footer_y = 11.0;
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

fn is_inline_node(node: &Node) -> bool {
    match node {
        Node::Text(_) => true,
        Node::Comment(_) => false,
        Node::Section(_) => false,
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            matches!(
                tag.as_str(),
                "b" | "strong"
                    | "i"
                    | "em"
                    | "u"
                    | "code"
                    | "span"
                    | "a"
                    | "mark"
                    | "sup"
                    | "sub"
                    | "br"
                    | "text"
            )
        }
    }
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
                        text: t.content.clone(),
                        bold: false,
                        italic: false,
                        code: false,
                    };
                    render_styled_paragraph(
                        &[run],
                        10.5,
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
                    render_styled_paragraph(
                        &runs,
                        10.5,
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
                    *cursor_y -= 3.0;
                    continue;
                }

                match tag.as_str() {
                    "h1" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 4.0;
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
                        *cursor_y -= 3.0;
                        render_heading(
                            &text,
                            16.0,
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
                            13.0,
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
                        let mut runs = Vec::new();
                        for child in &el.children {
                            collect_styled_runs(child, false, false, false, &mut runs);
                        }
                        render_styled_paragraph(
                            &runs,
                            10.5,
                            align,
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
                        let has_only_inlines = el.children.iter().all(|c| match c {
                            Node::Comment(_) => true,
                            Node::Text(_) => true,
                            Node::Element(sub) => is_inline_node(&Node::Element(sub.clone())),
                            _ => false,
                        });

                        if has_only_inlines && !el.children.is_empty() {
                            let mut runs = Vec::new();
                            for child in &el.children {
                                collect_styled_runs(child, false, false, false, &mut runs);
                            }
                            render_styled_paragraph(
                                &[StyledRun {
                                    text: String::new(),
                                    bold: false,
                                    italic: false,
                                    code: false,
                                }],
                                10.5,
                                align,
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
                        } else {
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
    let mut lowest_y = *cursor_y;

    for (i, col_el) in col_divs.iter().enumerate() {
        let col_left = left_margin + (i as f32) * (col_width + gap_mm);
        let mut col_cursor = start_y;

        render_doc_nodes(
            &col_el.children,
            pdf_doc,
            current_page,
            current_layer,
            page_count,
            &mut col_cursor,
            col_left,
            col_width,
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

        if col_cursor < lowest_y {
            lowest_y = col_cursor;
        }
    }

    *cursor_y = lowest_y - 4.0;
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
    let actual_left = if is_callout { left_margin + 5.0 } else { left_margin };
    let actual_width = if is_callout { content_width - 5.0 } else { content_width };

    let callout_top_y = *cursor_y + 1.0;

    // Tokenize runs into words and whitespace
    let mut tokens: Vec<WordToken> = Vec::new();

    for run in runs {
        let font = if run.code {
            font_code
        } else if run.bold {
            font_bold
        } else if run.italic {
            font_italic
        } else {
            font_reg
        };

        let width_factor = if run.code { 0.21 } else { 0.195 };

        let mut current_word = String::new();
        for ch in run.text.chars() {
            if ch == '\n' {
                if !current_word.is_empty() {
                    let w = (current_word.len() as f32) * font_size * width_factor;
                    let static_str = Box::leak(current_word.into_boxed_str());
                    tokens.push(WordToken {
                        text: static_str,
                        font,
                        font_size,
                        width_mm: w,
                        is_space: false,
                        is_newline: false,
                    });
                    current_word = String::new();
                }
                tokens.push(WordToken {
                    text: "",
                    font,
                    font_size,
                    width_mm: 0.0,
                    is_space: false,
                    is_newline: true,
                });
            } else if ch.is_whitespace() {
                if !current_word.is_empty() {
                    let w = (current_word.len() as f32) * font_size * width_factor;
                    let static_str = Box::leak(current_word.into_boxed_str());
                    tokens.push(WordToken {
                        text: static_str,
                        font,
                        font_size,
                        width_mm: w,
                        is_space: false,
                        is_newline: false,
                    });
                    current_word = String::new();
                }
                tokens.push(WordToken {
                    text: " ",
                    font,
                    font_size,
                    width_mm: font_size * 0.10,
                    is_space: true,
                    is_newline: false,
                });
            } else {
                current_word.push(ch);
            }
        }
        if !current_word.is_empty() {
            let w = (current_word.len() as f32) * font_size * width_factor;
            let static_str = Box::leak(current_word.into_boxed_str());
            tokens.push(WordToken {
                text: static_str,
                font,
                font_size,
                width_mm: w,
                is_space: false,
                is_newline: false,
            });
        }
    }

    if tokens.is_empty() {
        return;
    }

    // Line wrap
    let mut current_line: Vec<&WordToken> = Vec::new();
    let mut current_line_width = 0.0f32;

    for token in &tokens {
        if token.is_newline {
            // Flush current line
            flush_line(
                &current_line,
                current_line_width,
                actual_left,
                actual_width,
                align,
                current_layer,
                *cursor_y,
            );
            *cursor_y -= line_height_mm;
            current_line.clear();
            current_line_width = 0.0;
            continue;
        }

        if current_line_width + token.width_mm > actual_width && !current_line.is_empty() {
            if *cursor_y - line_height_mm < bottom_margin {
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

            flush_line(
                &current_line,
                current_line_width,
                actual_left,
                actual_width,
                align,
                current_layer,
                *cursor_y,
            );
            *cursor_y -= line_height_mm;
            current_line.clear();
            current_line_width = 0.0;

            if token.is_space {
                continue;
            }
        }

        current_line.push(token);
        current_line_width += token.width_mm;
    }

    if !current_line.is_empty() {
        if *cursor_y - line_height_mm < bottom_margin {
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

        flush_line(
            &current_line,
            current_line_width,
            actual_left,
            actual_width,
            align,
            current_layer,
            *cursor_y,
        );
        *cursor_y -= line_height_mm;
    }

    // Callout green left accent bar
    if is_callout {
        let callout_bottom_y = *cursor_y + 1.5;
        current_layer.set_outline_color(Color::Rgb(Rgb::new(
            22.0 / 255.0,
            163.0 / 255.0,
            74.0 / 255.0,
            None,
        ))); // #16A34A
        current_layer.set_outline_thickness(1.2);
        let bar_points = vec![
            (Point::new(Mm(left_margin), Mm(callout_top_y)), false),
            (Point::new(Mm(left_margin), Mm(callout_bottom_y)), false),
        ];
        current_layer.add_line(Line {
            points: bar_points,
            is_closed: false,
        });
        current_layer.set_outline_thickness(0.5);
    }
}

fn flush_line(
    line: &[&WordToken],
    line_width: f32,
    actual_left: f32,
    actual_width: f32,
    align: Option<&'static str>,
    layer: &PdfLayerReference,
    cursor_y: f32,
) {
    let mut x = actual_left;
    if align == Some("center") {
        x = (actual_left + (actual_width - line_width) / 2.0).max(actual_left);
    } else if align == Some("right") {
        x = (actual_left + actual_width - line_width).max(actual_left);
    }

    for tok in line {
        if !tok.is_space && !tok.text.is_empty() {
            layer.use_text(tok.text, tok.font_size, Mm(x), Mm(cursor_y), tok.font);
        }
        x += tok.width_mm;
    }
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
