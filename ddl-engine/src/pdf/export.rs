use crate::ast::{Document, ElementNode, Node, PageStyle};
use base64::Engine;
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::*;
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    SansSerif,
    Serif,
    Monospace,
}

#[derive(Clone, Copy)]
pub struct FontSet<'a> {
    pub helvetica_reg: &'a IndirectFontRef,
    pub helvetica_bold: &'a IndirectFontRef,
    pub helvetica_italic: &'a IndirectFontRef,
    pub times_reg: &'a IndirectFontRef,
    pub times_bold: &'a IndirectFontRef,
    pub times_italic: &'a IndirectFontRef,
    pub courier: &'a IndirectFontRef,
}

impl<'a> FontSet<'a> {
    pub fn select(&self, family: FontFamily, bold: bool, italic: bool) -> &'a IndirectFontRef {
        match family {
            FontFamily::Monospace => self.courier,
            FontFamily::Serif => {
                if bold {
                    self.times_bold
                } else if italic {
                    self.times_italic
                } else {
                    self.times_reg
                }
            }
            FontFamily::SansSerif => {
                if bold {
                    self.helvetica_bold
                } else if italic {
                    self.helvetica_italic
                } else {
                    self.helvetica_reg
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct StyledRun {
    text: String,
    family: FontFamily,
    bold: bool,
    italic: bool,
}

#[derive(Debug, Clone)]
struct ParagraphWord {
    text: String,
    family: FontFamily,
    bold: bool,
    italic: bool,
    has_trailing_space: bool,
    width_mm: f32,
}

#[derive(Debug, Clone)]
struct LineSpan {
    text: String,
    family: FontFamily,
    bold: bool,
    italic: bool,
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
    let font_times_reg = pdf_doc.add_builtin_font(BuiltinFont::TimesRoman)?;
    let font_times_bold = pdf_doc.add_builtin_font(BuiltinFont::TimesBold)?;
    let font_times_italic = pdf_doc.add_builtin_font(BuiltinFont::TimesItalic)?;

    let fonts = FontSet {
        helvetica_reg: &font_regular,
        helvetica_bold: &font_bold,
        helvetica_italic: &font_italic,
        times_reg: &font_times_reg,
        times_bold: &font_times_bold,
        times_italic: &font_times_italic,
        courier: &font_code,
    };

    let mut current_page = first_page;
    let mut current_layer = pdf_doc.get_page(current_page).get_layer(first_layer);
    let mut page_count = 1;

    // Dynamic margins from document props
    let margin_mm = doc.props.paper.margin.val;
    let left_margin = margin_mm.max(12.0);
    let right_margin = margin_mm.max(12.0);
    let content_width = width_mm - left_margin - right_margin;
    let (header_clearance, footer_clearance) = get_header_and_footer_heights(doc, width_mm, content_width);
    let top_margin = (margin_mm * 1.1).max(22.0).max(header_clearance);
    let bottom_margin = (margin_mm * 1.0).max(18.0).max(footer_clearance);

    // Draw header and footer on page 1
    draw_running_header_footer(
        &current_layer,
        1,
        left_margin,
        content_width,
        width_mm,
        height_mm,
        fonts.helvetica_reg,
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
        &fonts,
        0,
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

fn find_img_in_nodes<'a>(nodes: &'a [Node]) -> Option<&'a ElementNode> {
    for n in nodes {
        match n {
            Node::Element(el) => {
                if el.tag.to_lowercase() == "img" {
                    return Some(el);
                }
                if let Some(sub) = find_img_in_nodes(&el.children) {
                    return Some(sub);
                }
            }
            _ => {}
        }
    }
    None
}

fn load_image_bytes(src: &str) -> Option<Vec<u8>> {
    if src.starts_with("data:image/") {
        if let Some(pos) = src.find(',') {
            let b64 = &src[pos + 1..];
            return base64::engine::general_purpose::STANDARD.decode(b64.trim()).ok();
        }
    }
    if let Ok(bytes) = std::fs::read(src) {
        return Some(bytes);
    }
    for prefix in &["assets/", "media/", "../", "../media/", "word/media/"] {
        let candidate = format!("{}{}", prefix, src);
        if let Ok(bytes) = std::fs::read(&candidate) {
            return Some(bytes);
        }
    }
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(src.trim()) {
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    None
}

fn draw_image_on_layer(
    layer: &PdfLayerReference,
    dynamic_img: &::image::DynamicImage,
    x_mm: f32,
    y_bottom_mm: f32,
    target_w_mm: f32,
    target_h_mm: f32,
) {
    let px_w = dynamic_img.width() as f32;
    let px_h = dynamic_img.height() as f32;
    if px_w <= 0.0 || px_h <= 0.0 {
        return;
    }

    let printpdf_img = Image::from_dynamic_image(dynamic_img);

    let native_w_mm = px_w * 25.4 / 300.0;
    let native_h_mm = px_h * 25.4 / 300.0;
    let scale_x = target_w_mm / native_w_mm;
    let scale_y = target_h_mm / native_h_mm;

    let transform = ImageTransform {
        translate_x: Some(Mm(x_mm)),
        translate_y: Some(Mm(y_bottom_mm)),
        rotate: None,
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        dpi: Some(300.0),
    };

    printpdf_img.add_to_layer(layer.clone(), transform);
}

fn get_header_and_footer_heights(doc: &Document, width_mm: f32, content_width: f32) -> (f32, f32) {
    let ps = get_active_page_style(doc);
    let mut header_h = 0.0f32;
    let mut footer_h = 0.0f32;

    if let Some(p) = ps {
        if let Some(h) = p.running_header.as_ref() {
            let img_opt = find_img_in_nodes(&h.center)
                .or_else(|| find_img_in_nodes(&h.left))
                .or_else(|| find_img_in_nodes(&h.right));
            if let Some(img_el) = img_opt {
                if let Some(src) = img_el.attributes.get("src") {
                    if let Some(bytes) = load_image_bytes(src) {
                        if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                            let px_w = dyn_img.width() as f32;
                            let px_h = dyn_img.height() as f32;
                            let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                            let max_target_h = 35.0_f32;
                            let target_w = content_width.min(max_target_h / aspect);
                            let target_h = target_w * aspect;
                            header_h = target_h + 8.0;
                        }
                    }
                }
            }
            if header_h == 0.0 && (!h.left.is_empty() || !h.center.is_empty() || !h.right.is_empty()) {
                header_h = 16.0;
            }
        }

        if let Some(f) = p.running_footer.as_ref() {
            let img_opt = find_img_in_nodes(&f.center)
                .or_else(|| find_img_in_nodes(&f.left))
                .or_else(|| find_img_in_nodes(&f.right));
            if let Some(img_el) = img_opt {
                if let Some(src) = img_el.attributes.get("src") {
                    if let Some(bytes) = load_image_bytes(src) {
                        if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                            let px_w = dyn_img.width() as f32;
                            let px_h = dyn_img.height() as f32;
                            let aspect = if px_w > 0.0 { px_h / px_w } else { 0.15 };
                            let is_full_bleed = img_el.classes.iter().any(|c| c == "full-bleed")
                                || aspect < 0.22
                                || img_el.inline_styles.get("width").map(|s| s.contains("100%")).unwrap_or(false);
                            if is_full_bleed {
                                let target_h = width_mm * aspect;
                                footer_h = target_h + 6.0;
                            } else {
                                let target_w = content_width.min(180.0);
                                let target_h = (target_w * aspect).min(25.0);
                                footer_h = target_h + 8.0;
                            }
                        }
                    }
                }
            }
            if footer_h == 0.0 && (!f.left.is_empty() || !f.center.is_empty() || !f.right.is_empty()) {
                footer_h = 14.0;
            }
        }
    }

    (header_h, footer_h)
}

fn draw_running_header_footer(
    layer: &PdfLayerReference,
    page_num: usize,
    left_margin: f32,
    content_width: f32,
    width_mm: f32,
    height_mm: f32,
    font_reg: &IndirectFontRef,
    doc: &Document,
) {
    let ps = get_active_page_style(doc);

    // 1. Check if running header contains an image
    let header_img_opt = ps
        .and_then(|p| p.running_header.as_ref())
        .and_then(|h| {
            find_img_in_nodes(&h.center)
                .or_else(|| find_img_in_nodes(&h.left))
                .or_else(|| find_img_in_nodes(&h.right))
        });

    let mut drew_header_image = false;
    if let Some(img_el) = header_img_opt {
        if let Some(src) = img_el.attributes.get("src") {
            if let Some(bytes) = load_image_bytes(src) {
                if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                    let px_w = dyn_img.width() as f32;
                    let px_h = dyn_img.height() as f32;
                    let aspect = if px_w > 0.0 { px_h / px_w } else { 0.2 };
                    let max_target_h = 35.0_f32;
                    let target_w = content_width.min(max_target_h / aspect);
                    let target_h = target_w * aspect;
                    let img_x = left_margin + (content_width - target_w) / 2.0;
                    let img_y = height_mm - target_h - 4.0;
                    draw_image_on_layer(layer, &dyn_img, img_x, img_y, target_w, target_h);
                    drew_header_image = true;
                }
            }
        }
    }

    if !drew_header_image {
        if let Some(h) = ps.and_then(|p| p.running_header.as_ref()) {
            let left_text = extract_nodes_text(&h.left);
            let center_text = extract_nodes_text(&h.center);
            let right_text = extract_nodes_text(&h.right);
            let display_right = if !right_text.is_empty() {
                right_text.replace("<page-number/>", &format!("{}", page_num))
            } else if !left_text.is_empty() || !center_text.is_empty() {
                format!("{} of 1", page_num)
            } else {
                String::new()
            };

            if !left_text.is_empty() || !center_text.is_empty() || !display_right.is_empty() {
                layer.set_fill_color(Color::Rgb(Rgb::new(
                    100.0 / 255.0,
                    116.0 / 255.0,
                    139.0 / 255.0,
                    None,
                )));

                let header_y = height_mm - 11.5;
                if !left_text.is_empty() {
                    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(header_y), font_reg);
                }
                if !center_text.is_empty() {
                    let center_w = (center_text.len() as f32) * 8.5 * 0.19;
                    let center_x = (left_margin + (content_width - center_w) / 2.0).max(left_margin);
                    layer.use_text(&center_text, 8.5, Mm(center_x), Mm(header_y), font_reg);
                }
                if !display_right.is_empty() {
                    let right_w = (display_right.len() as f32) * 8.5 * 0.19;
                    let right_x = (left_margin + content_width - right_w).max(left_margin);
                    layer.use_text(&display_right, 8.5, Mm(right_x), Mm(header_y), font_reg);
                }

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
            }
        }
    }

    // 2. Check if running footer contains an image
    let footer_img_opt = ps
        .and_then(|p| p.running_footer.as_ref())
        .and_then(|f| {
            find_img_in_nodes(&f.center)
                .or_else(|| find_img_in_nodes(&f.left))
                .or_else(|| find_img_in_nodes(&f.right))
        });

    let mut drew_footer_image = false;
    if let Some(img_el) = footer_img_opt {
        if let Some(src) = img_el.attributes.get("src") {
            if let Some(bytes) = load_image_bytes(src) {
                if let Ok(dyn_img) = ::image::load_from_memory(&bytes) {
                    let px_w = dyn_img.width() as f32;
                    let px_h = dyn_img.height() as f32;
                    let aspect = if px_w > 0.0 { px_h / px_w } else { 0.15 };
                    let is_full_bleed = img_el.classes.iter().any(|c| c == "full-bleed")
                        || aspect < 0.22
                        || img_el.inline_styles.get("width").map(|s| s.contains("100%")).unwrap_or(false);

                    if is_full_bleed {
                        let target_w = width_mm;
                        let target_h = target_w * aspect;
                        let img_x = 0.0;
                        let img_y = 0.0;
                        draw_image_on_layer(layer, &dyn_img, img_x, img_y, target_w, target_h);
                    } else {
                        let target_w = content_width.min(180.0);
                        let target_h = (target_w * aspect).min(24.0);
                        let img_x = left_margin + (content_width - target_w) / 2.0;
                        let img_y = 5.0;
                        draw_image_on_layer(layer, &dyn_img, img_x, img_y, target_w, target_h);
                    }
                    drew_footer_image = true;
                }
            }
        }
    }

    if !drew_footer_image {
        if let Some(f) = ps.and_then(|p| p.running_footer.as_ref()) {
            let left_text = extract_nodes_text(&f.left);
            let center_text = extract_nodes_text(&f.center);
            let right_text = extract_nodes_text(&f.right);

            if !left_text.is_empty() || !center_text.is_empty() || !right_text.is_empty() {
                layer.set_fill_color(Color::Rgb(Rgb::new(
                    100.0 / 255.0,
                    116.0 / 255.0,
                    139.0 / 255.0,
                    None,
                )));

                let footer_y = 10.5;
                if !left_text.is_empty() {
                    layer.use_text(&left_text, 8.5, Mm(left_margin), Mm(footer_y), font_reg);
                }
                if !center_text.is_empty() {
                    let footer_w = (center_text.len() as f32) * 8.5 * 0.19;
                    let footer_x = (left_margin + (content_width - footer_w) / 2.0).max(left_margin);
                    layer.use_text(&center_text, 8.5, Mm(footer_x), Mm(footer_y), font_reg);
                }
                if !right_text.is_empty() {
                    let right_w = (right_text.len() as f32) * 8.5 * 0.19;
                    let right_x = (left_margin + content_width - right_w).max(left_margin);
                    layer.use_text(&right_text, 8.5, Mm(right_x), Mm(footer_y), font_reg);
                }
            }
        }
    }

    // Reset fill color to Dark Slate #0f172a
    layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}

fn draw_rounded_rect(
    layer: &PdfLayerReference,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    mode: PaintMode,
) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    if r <= 0.05 {
        let mut rect = Rect::new(Mm(x), Mm(y), Mm(x + w), Mm(y + h));
        rect.mode = mode;
        layer.add_rect(rect);
        return;
    }

    let k = 0.55228475 * r;
    let ring: Vec<(Point, bool)> = vec![
        (Point::new(Mm(x + r), Mm(y)), false),
        (Point::new(Mm(x + w - r), Mm(y)), false),
        (Point::new(Mm(x + w - r + k), Mm(y)), true),
        (Point::new(Mm(x + w), Mm(y + r - k)), true),
        (Point::new(Mm(x + w), Mm(y + r)), false),
        (Point::new(Mm(x + w), Mm(y + h - r)), false),
        (Point::new(Mm(x + w), Mm(y + h - r + k)), true),
        (Point::new(Mm(x + w - r + k), Mm(y + h)), true),
        (Point::new(Mm(x + w - r), Mm(y + h)), false),
        (Point::new(Mm(x + r), Mm(y + h)), false),
        (Point::new(Mm(x + r - k), Mm(y + h)), true),
        (Point::new(Mm(x), Mm(y + h - r + k)), true),
        (Point::new(Mm(x), Mm(y + h - r)), false),
        (Point::new(Mm(x), Mm(y + r)), false),
        (Point::new(Mm(x), Mm(y + r - k)), true),
        (Point::new(Mm(x + r - k), Mm(y)), true),
        (Point::new(Mm(x + r), Mm(y)), false),
    ];

    let poly = Polygon {
        rings: vec![ring],
        mode,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(poly);
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

fn detect_font_family(el: &ElementNode, doc: &Document) -> FontFamily {
    if let Some(ff) = el.inline_styles.get("font-family") {
        let ffl = ff.to_lowercase();
        if ffl.contains("serif") || ffl.contains("times") || ffl.contains("georgia") {
            return FontFamily::Serif;
        }
        if ffl.contains("mono") || ffl.contains("consolas") || ffl.contains("courier") {
            return FontFamily::Monospace;
        }
    }
    for rule in &doc.props.stylesheet.rules {
        if rule.selectors.iter().any(|s| s == "body" || s == &el.tag) {
            if let Some(ff) = rule.declarations.get("font-family") {
                let ffl = ff.to_lowercase();
                if ffl.contains("serif") || ffl.contains("times") || ffl.contains("georgia") {
                    return FontFamily::Serif;
                }
                if ffl.contains("mono") || ffl.contains("consolas") || ffl.contains("courier") {
                    return FontFamily::Monospace;
                }
            }
        }
    }
    FontFamily::SansSerif
}

fn collect_styled_runs(
    node: &Node,
    family: FontFamily,
    bold: bool,
    italic: bool,
    runs: &mut Vec<StyledRun>,
) {
    match node {
        Node::Text(t) => {
            runs.push(StyledRun {
                text: t.content.clone(),
                family,
                bold,
                italic,
            });
        }
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            if tag == "ul" || tag == "ol" {
                return;
            }
            let mut b = bold;
            let mut i = italic;
            let mut f = family;
            if tag == "b" || tag == "strong" {
                b = true;
            } else if tag == "i" || tag == "em" {
                i = true;
            } else if tag == "code" {
                f = FontFamily::Monospace;
            } else if tag == "br" {
                runs.push(StyledRun {
                    text: "\n".to_string(),
                    family: f,
                    bold: b,
                    italic: i,
                });
                return;
            }
            for child in &el.children {
                collect_styled_runs(child, f, b, i, runs);
            }
        }
        _ => {}
    }
}

fn to_lower_alpha(mut n: usize) -> String {
    if n == 0 {
        return "a".to_string();
    }
    let mut s = String::new();
    while n > 0 {
        n -= 1;
        let rem = (n % 26) as u8;
        s.insert(0, (b'a' + rem) as char);
        n /= 26;
    }
    s
}

fn to_upper_alpha(n: usize) -> String {
    to_lower_alpha(n).to_uppercase()
}

fn to_roman(mut n: usize) -> String {
    if n == 0 {
        return "i".to_string();
    }
    let table = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut res = String::new();
    for &(val, sym) in &table {
        while n >= val {
            res.push_str(sym);
            n -= val;
        }
    }
    res
}

fn format_list_bullet(tag: &str, list_type: Option<&str>, idx: usize, indent_level: usize) -> String {
    if tag == "ul" {
        let default_bullet = match indent_level % 3 {
            0 => "disc",
            1 => "circle",
            _ => "square",
        };
        match list_type.unwrap_or(default_bullet) {
            "circle" => "◦ ".to_string(),
            "square" => "▪ ".to_string(),
            "dash" => "- ".to_string(),
            _ => "• ".to_string(),
        }
    } else {
        let default_num = match indent_level % 3 {
            0 => "1",
            1 => "a",
            _ => "i",
        };
        match list_type.unwrap_or(default_num) {
            "a" | "lower-alpha" => format!("{}. ", to_lower_alpha(idx)),
            "A" | "upper-alpha" => format!("{}. ", to_upper_alpha(idx)),
            "i" | "lower-roman" => format!("{}. ", to_roman(idx)),
            "I" | "upper-roman" => format!("{}. ", to_roman(idx).to_uppercase()),
            _ => format!("{}. ", idx),
        }
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
    fonts: &FontSet,
    indent_level: usize,
    doc: &Document,
) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                let trimmed = t.content.trim();
                if !trimmed.is_empty() {
                    let run = StyledRun {
                        text: trimmed.to_string(),
                        family: FontFamily::SansSerif,
                        bold: false,
                        italic: false,
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
                        fonts,
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
                    fonts,
                    indent_level,
                    doc,
                );
            }
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                let align = get_align(el);
                let is_callout = tag == "callout" || el.classes.iter().any(|c| c == "callout");
                let el_font_family = detect_font_family(el, doc);

                if is_callout {
                    let mut runs = Vec::new();
                    for child in &el.children {
                        collect_styled_runs(child, el_font_family, false, false, &mut runs);
                    }
                    *cursor_y -= 4.0;
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
                        fonts,
                        doc,
                    );
                    *cursor_y -= 5.0;
                    continue;
                }

                match tag.as_str() {
                    "h1" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 6.0;
                        render_heading(
                            &text,
                            24.0,
                            align,
                            Some((30.0 / 255.0, 58.0 / 255.0, 138.0 / 255.0)), // #1e3a8a
                            fonts.select(el_font_family, true, false),
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
                            fonts.helvetica_reg,
                            doc,
                        );
                        *cursor_y -= 4.0;
                    }
                    "h2" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 6.0;
                        render_heading(
                            &text,
                            15.0,
                            align,
                            Some((15.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0)), // #0f172a
                            fonts.select(el_font_family, true, false),
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
                            fonts.helvetica_reg,
                            doc,
                        );
                        *cursor_y -= 3.0;
                    }
                    "h3" => {
                        let text = extract_plain_text(el);
                        *cursor_y -= 4.0;
                        render_heading(
                            &text,
                            11.5,
                            align,
                            Some((30.0 / 255.0, 41.0 / 255.0, 59.0 / 255.0)), // #1e293b
                            fonts.select(el_font_family, true, false),
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
                            fonts.helvetica_reg,
                            doc,
                        );
                        *cursor_y -= 2.0;
                    }
                    "p" => {
                        let is_lead = el.classes.iter().any(|c| c == "lead");
                        let p_font_size = if is_lead { 12.0 } else { 10.5 };
                        let p_color = if is_lead {
                            Some((71.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0)) // #475569
                        } else {
                            Some((51.0 / 255.0, 65.0 / 255.0, 85.0 / 255.0)) // #334155
                        };

                        let mut runs = Vec::new();
                        for child in &el.children {
                            collect_styled_runs(child, el_font_family, false, false, &mut runs);
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
                            fonts,
                            doc,
                        );
                        *cursor_y -= 3.5;
                    }
                    "blockquote" => {
                        *cursor_y -= 3.0;
                        let bq_start_y = *cursor_y;
                        let mut runs = Vec::new();
                        for child in &el.children {
                            collect_styled_runs(child, el_font_family, false, true, &mut runs);
                        }
                        let bq_indent = 7.0;
                        let bq_color = Some((71.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0));
                        render_styled_paragraph(
                            &runs,
                            10.0,
                            None,
                            bq_color,
                            false,
                            pdf_doc,
                            current_page,
                            current_layer,
                            page_count,
                            cursor_y,
                            left_margin + bq_indent,
                            content_width - bq_indent,
                            height_mm,
                            top_margin,
                            bottom_margin,
                            width_mm,
                            fonts,
                            doc,
                        );
                        let bq_end_y = *cursor_y + 1.0;
                        current_layer.set_outline_color(Color::Rgb(Rgb::new(
                            203.0 / 255.0,
                            213.0 / 255.0,
                            225.0 / 255.0,
                            None,
                        )));
                        current_layer.set_outline_thickness(1.2);
                        let bar_line = Line {
                            points: vec![
                                (Point::new(Mm(left_margin + 2.0), Mm(bq_end_y)), false),
                                (Point::new(Mm(left_margin + 2.0), Mm(bq_start_y)), false),
                            ],
                            is_closed: false,
                        };
                        current_layer.add_line(bar_line);
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
                            fonts,
                            doc,
                        );
                    }
                    "ul" | "ol" => {
                        let list_type_attr = el.attributes.get("type").map(|s| s.as_str());
                        let list_style = el
                            .inline_styles
                            .get("list-style-type")
                            .map(|s| s.as_str())
                            .or(list_type_attr);
                        let start_idx = el
                            .attributes
                            .get("start")
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(1);

                        *cursor_y -= 1.5;
                        let mut item_idx = 0;
                        for child in &el.children {
                            if let Node::Element(li) = child {
                                if li.tag.to_lowercase() == "li" {
                                    let bullet = format_list_bullet(
                                        &tag,
                                        list_style,
                                        start_idx + item_idx,
                                        indent_level,
                                    );
                                    item_idx += 1;
                                    let mut runs = Vec::new();
                                    runs.push(StyledRun {
                                        text: bullet,
                                        family: FontFamily::SansSerif,
                                        bold: true,
                                        italic: false,
                                    });
                                    let mut nested_lists: Vec<Node> = Vec::new();
                                    for li_child in &li.children {
                                        if let Node::Element(child_el) = li_child {
                                            let child_tag = child_el.tag.to_lowercase();
                                            if child_tag == "ul" || child_tag == "ol" {
                                                nested_lists.push(li_child.clone());
                                                continue;
                                            }
                                        }
                                        collect_styled_runs(
                                            li_child,
                                            el_font_family,
                                            false,
                                            false,
                                            &mut runs,
                                        );
                                    }
                                    let indent_offset = (indent_level as f32) * 6.0 + 5.0;
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
                                        left_margin + indent_offset,
                                        content_width - indent_offset,
                                        height_mm,
                                        top_margin,
                                        bottom_margin,
                                        width_mm,
                                        fonts,
                                        doc,
                                    );
                                    *cursor_y -= 1.5;

                                    if !nested_lists.is_empty() {
                                        render_doc_nodes(
                                            &nested_lists,
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
                                            fonts,
                                            indent_level + 1,
                                            doc,
                                        );
                                    }
                                }
                            }
                        }
                        *cursor_y -= 2.0;
                    }
                    "img" => {
                        render_image(
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
                            fonts,
                            doc,
                        );
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
                            fonts,
                            doc,
                        );
                    }
                    "hr" => {
                        *cursor_y -= 3.0;
                        current_layer.set_outline_color(Color::Rgb(Rgb::new(
                            203.0 / 255.0,
                            213.0 / 255.0,
                            225.0 / 255.0,
                            None,
                        )));
                        current_layer.set_outline_thickness(0.6);
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
                        // General container: div, etc.
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
                            fonts,
                            indent_level,
                            doc,
                        );
                    }
                }
            }
        }
    }
}

fn char_width(ch: char, family: FontFamily, bold: bool) -> f32 {
    match family {
        FontFamily::Monospace => 600.0 / 1000.0,
        FontFamily::Serif => {
            let w = if bold {
                match ch {
                    ' ' => 250.0,
                    'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 278.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 444.0,
                    'a' | 'c' | 'e' | 'o' | 'u' | 'v' | 'x' | 'z' => 500.0,
                    'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'y' | '0'..='9' => 556.0,
                    'w' => 722.0,
                    'm' => 833.0,
                    'I' => 333.0,
                    'J' => 444.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 889.0,
                    _ => 500.0,
                }
            } else {
                match ch {
                    ' ' => 250.0,
                    'i' | 'l' | '!' | '.' | ',' | ':' | ';' | '\'' | '`' => 250.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 389.0,
                    'a' | 'c' | 'e' | 'o' | 'u' | 'v' | 'x' | 'z' => 444.0,
                    'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'y' | '0'..='9' => 500.0,
                    'w' => 722.0,
                    'm' => 778.0,
                    'I' => 333.0,
                    'J' => 389.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 889.0,
                    _ => 500.0,
                }
            };
            w / 1000.0
        }
        FontFamily::SansSerif => {
            let w = if bold {
                match ch {
                    ' ' => 278.0,
                    '!' | ',' | '.' | ':' | ';' | 'i' | 'l' | '\'' | '`' => 300.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 389.0,
                    's' => 556.0,
                    'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' | 'z' | '0'..='9' => 611.0,
                    'w' => 778.0,
                    'm' => 889.0,
                    'I' => 333.0,
                    'J' => 556.0,
                    'L' => 611.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 722.0,
                    'M' | 'W' => 944.0,
                    _ => 611.0,
                }
            } else {
                match ch {
                    ' ' => 278.0,
                    '!' | ',' | '.' | ':' | ';' | 'i' | 'l' | '\'' | '`' => 278.0,
                    'j' | 'r' | 't' | 'f' | '-' | '(' | ')' | '[' | ']' => 333.0,
                    's' => 500.0,
                    'a' | 'b' | 'c' | 'd' | 'e' | 'g' | 'h' | 'k' | 'n' | 'o' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' | 'z' | '0'..='9' => 556.0,
                    'w' => 722.0,
                    'm' => 833.0,
                    'I' => 278.0,
                    'J' => 500.0,
                    'L' => 556.0,
                    'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'T' | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
                    'M' | 'W' => 833.0,
                    _ => 556.0,
                }
            };
            w / 1000.0
        }
    }
}

fn measure_text(text: &str, font_size: f32, family: FontFamily, bold: bool) -> f32 {
    let pt_to_mm = 0.352778;
    if family == FontFamily::Monospace {
        (text.chars().count() as f32) * 0.60 * font_size * pt_to_mm
    } else {
        let em_sum: f32 = text.chars().map(|c| char_width(c, family, bold)).sum();
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
                    "h1" => h += 16.0,
                    "h2" => h += 11.0,
                    "h3" => h += 8.5,
                    "p" => {
                        let text = extract_plain_text(el);
                        let chars = text.chars().count();
                        let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                        h += est_lines * 5.2 + 4.0;
                    }
                    _ => h += estimate_nodes_height(&el.children, width),
                }
            }
            Node::Text(t) => {
                let chars = t.content.trim().chars().count();
                if chars > 0 {
                    let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                    h += est_lines * 5.2;
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
    fonts: &FontSet,
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
        .max(24.0)
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
            fonts.helvetica_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    let actual_start_y = *cursor_y;
    let actual_uniform_bottom = actual_start_y - max_col_height;
    let card_height = actual_start_y - actual_uniform_bottom;

    // Draw card background & border with rounded corners
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

            draw_rounded_rect(
                current_layer,
                col_left,
                actual_uniform_bottom,
                col_width,
                card_height,
                1.8,
                PaintMode::FillStroke,
            );
        }
    }

    // Render text inside cards
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
            fonts,
            0,
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
        let text_width_approx = (text.len() as f32) * font_size * 0.22;
        x = (left_margin + (content_width - text_width_approx) / 2.0).max(left_margin);
    } else if align == Some("right") {
        let text_width_approx = (text.len() as f32) * font_size * 0.22;
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
    fonts: &FontSet,
    doc: &Document,
) {
    let line_height_mm = font_size * 0.44;
    let actual_left = if is_callout {
        left_margin + 6.0
    } else {
        left_margin
    };
    let actual_width = if is_callout {
        content_width - 12.0
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
                family: run.family,
                bold: run.bold,
                italic: run.italic,
            });
        }
    }

    if cleaned_runs.is_empty() {
        return;
    }

    // Tokenize into words with exact font metrics
    let mut words: Vec<ParagraphWord> = Vec::new();
    let space_w = measure_text(" ", font_size, FontFamily::SansSerif, false);

    for run in &cleaned_runs {
        let family = run.family;
        let bold = run.bold;
        let italic = run.italic;
        let mut cur_word = String::new();
        let chars: Vec<char> = run.text.chars().collect();
        let n = chars.len();
        let mut i = 0;

        while i < n {
            let ch = chars[i];
            if ch == '\n' {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, family, bold);
                    words.push(ParagraphWord {
                        text: cur_word,
                        family,
                        bold,
                        italic,
                        has_trailing_space: false,
                        width_mm: w,
                    });
                    cur_word = String::new();
                }
                words.push(ParagraphWord {
                    text: "\n".to_string(),
                    family,
                    bold,
                    italic,
                    has_trailing_space: false,
                    width_mm: 0.0,
                });
                i += 1;
            } else if ch.is_whitespace() {
                if !cur_word.is_empty() {
                    let w = measure_text(&cur_word, font_size, family, bold);
                    words.push(ParagraphWord {
                        text: cur_word,
                        family,
                        bold,
                        italic,
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
            } else if (ch == ',' || ch == '.' || ch == ':' || ch == ';' || ch == '!' || ch == '?')
                && cur_word.is_empty()
            {
                // Punctuation following an inline tag clings to the preceding token
                if let Some(last) = words.last_mut() {
                    if !last.has_trailing_space {
                        last.text.push(ch);
                        last.width_mm = measure_text(&last.text, font_size, last.family, last.bold);
                        i += 1;
                        continue;
                    }
                }
                cur_word.push(ch);
                i += 1;
            } else {
                cur_word.push(ch);
                i += 1;
            }
        }

        if !cur_word.is_empty() {
            let w = measure_text(&cur_word, font_size, family, bold);
            words.push(ParagraphWord {
                text: cur_word,
                family,
                bold,
                italic,
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
        let mut cur_family = FontFamily::SansSerif;
        let mut cur_bold = false;
        let mut cur_italic = false;
        let num_words = line_words.len();

        for (idx, w) in line_words.iter().enumerate() {
            let is_last_word = idx == num_words - 1;
            let style_matches = !cur_span_text.is_empty()
                && cur_family == w.family
                && cur_bold == w.bold
                && cur_italic == w.italic;

            if style_matches {
                cur_span_text.push_str(&w.text);
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
            } else {
                if !cur_span_text.is_empty() {
                    let span_w = measure_text(&cur_span_text, font_size, cur_family, cur_bold);
                    spans.push(LineSpan {
                        text: cur_span_text,
                        family: cur_family,
                        bold: cur_bold,
                        italic: cur_italic,
                        font_size,
                        width_mm: span_w,
                    });
                }
                cur_span_text = w.text.clone();
                if w.has_trailing_space && !is_last_word {
                    cur_span_text.push(' ');
                }
                cur_family = w.family;
                cur_bold = w.bold;
                cur_italic = w.italic;
            }
        }

        if !cur_span_text.is_empty() {
            let span_w = measure_text(&cur_span_text, font_size, cur_family, cur_bold);
            spans.push(LineSpan {
                text: cur_span_text,
                family: cur_family,
                bold: cur_bold,
                italic: cur_italic,
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
            fonts.helvetica_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    // Draw Callout filled rounded background (#F0FDF4) and left green accent bar (#16A34A)
    if is_callout {
        let callout_top_y = *cursor_y + 3.0;
        let callout_bottom_y = *cursor_y - total_paragraph_height - 2.5;
        let callout_h = callout_top_y - callout_bottom_y;

        current_layer.set_fill_color(Color::Rgb(Rgb::new(
            240.0 / 255.0,
            253.0 / 255.0,
            244.0 / 255.0,
            None,
        )));
        draw_rounded_rect(
            current_layer,
            left_margin,
            callout_bottom_y,
            content_width,
            callout_h,
            1.8,
            PaintMode::Fill,
        );

        current_layer.set_fill_color(Color::Rgb(Rgb::new(
            22.0 / 255.0,
            163.0 / 255.0,
            74.0 / 255.0,
            None,
        )));
        draw_rounded_rect(
            current_layer,
            left_margin,
            callout_bottom_y,
            2.2,
            callout_h,
            1.0,
            PaintMode::Fill,
        );
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
            let font = fonts.select(span.family, span.bold, span.italic);

            if span.family == FontFamily::Monospace {
                // Rounded subtle code pill background
                let pill_pad_x = 0.8;
                let pill_pad_y = 0.4;
                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    241.0 / 255.0,
                    245.0 / 255.0,
                    249.0 / 255.0,
                    None,
                )));
                let pill_x = x - pill_pad_x;
                let pill_y = *cursor_y - pill_pad_y;
                let pill_w = span.width_mm + (pill_pad_x * 2.0);
                let pill_h = span.font_size * 0.35 + (pill_pad_y * 2.0);
                draw_rounded_rect(
                    current_layer,
                    pill_x,
                    pill_y,
                    pill_w,
                    pill_h,
                    0.8,
                    PaintMode::Fill,
                );
            }

            if span.family == FontFamily::Monospace {
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
    fonts: &FontSet,
    doc: &Document,
) {
    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);
    if rows.is_empty() {
        return;
    }

    let is_borderless = el.classes.iter().any(|c| c == "borderless")
        || el.attributes.get("class").map(|c| c.contains("borderless")).unwrap_or(false)
        || el.inline_styles.get("border").map(|b| b.contains("none")).unwrap_or(false);

    let is_align_right = el.classes.iter().any(|c| c == "align-right")
        || el.attributes.get("class").map(|c| c.contains("align-right")).unwrap_or(false)
        || el.attributes.get("align").map(|a| a == "right").unwrap_or(false)
        || el.inline_styles.get("margin-left").map(|m| m.contains("auto")).unwrap_or(false);

    let (table_left, table_width) = if is_align_right {
        (left_margin + content_width * 0.52, content_width * 0.48)
    } else {
        (left_margin, content_width)
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
    *cursor_y -= 3.0;

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
            fonts.helvetica_bold
        } else {
            fonts.helvetica_reg
        };

        // Extract lines for each cell to properly calculate row height
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
                fonts.helvetica_reg,
                doc,
            );
            *cursor_y = height_mm - top_margin;
        }

        // Draw header background & bottom border line only for standard bordered tables
        if !is_borderless {
            if is_header_row {
                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    248.0 / 255.0,
                    250.0 / 255.0,
                    252.0 / 255.0,
                    None,
                )));
                let mut header_rect = Rect::new(
                    Mm(table_left),
                    Mm(*cursor_y - row_height_mm),
                    Mm(table_left + table_width),
                    Mm(*cursor_y),
                );
                header_rect.mode = PaintMode::Fill;
                current_layer.add_rect(header_rect);
            }

            current_layer.set_outline_color(Color::Rgb(Rgb::new(
                226.0 / 255.0,
                232.0 / 255.0,
                240.0 / 255.0,
                None,
            )));
            current_layer.set_outline_thickness(0.5);
            let border_line = Line {
                points: vec![
                    (Point::new(Mm(table_left), Mm(*cursor_y - row_height_mm)), false),
                    (Point::new(Mm(table_left + table_width), Mm(*cursor_y - row_height_mm)), false),
                ],
                is_closed: false,
            };
            current_layer.add_line(border_line);
        }

        for (col_idx, cell) in cells.iter().enumerate() {
            let lines = &row_cell_lines[col_idx];
            let cell_align = cell.attributes.get("align")
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
            let start_y = *cursor_y - 4.5;
            let base_cell_x = table_left + (col_idx as f32) * col_width;

            for (line_idx, line_txt) in lines.iter().enumerate() {
                let y_pos = start_y - (line_idx as f32) * 4.2;
                let text_w_mm = (line_txt.len() as f32) * font_size * 0.18;
                let cell_x = match cell_align {
                    Some("center") => base_cell_x + ((col_width - text_w_mm) / 2.0).max(1.5),
                    Some("right") => base_cell_x + (col_width - text_w_mm - 2.0).max(1.5),
                    _ => base_cell_x + 2.0,
                };

                current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
                current_layer.use_text(line_txt, font_size, Mm(cell_x), Mm(y_pos), font);
            }
        }

        *cursor_y -= row_height_mm;
    }

    *cursor_y -= 4.0;
}

fn render_image(
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
    fonts: &FontSet,
    doc: &Document,
) {
    let src = match el.attributes.get("src") {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };

    let bytes = match load_image_bytes(src) {
        Some(b) => b,
        None => return,
    };

    let dyn_img = match ::image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => return,
    };

    let px_w = dyn_img.width() as f32;
    let px_h = dyn_img.height() as f32;
    if px_w <= 0.0 || px_h <= 0.0 {
        return;
    }

    let aspect = px_h / px_w;
    let mut target_w = content_width;
    if let Some(w_str) = el.attributes.get("width").or_else(|| el.inline_styles.get("width")) {
        if let Ok(w) = w_str.trim_end_matches("mm").trim_end_matches("px").parse::<f32>() {
            if w > 0.0 {
                target_w = w.min(content_width);
            }
        }
    }
    let mut target_h = target_w * aspect;
    if let Some(h_str) = el.attributes.get("height").or_else(|| el.inline_styles.get("max-height")) {
        if let Ok(h) = h_str.trim_end_matches("mm").trim_end_matches("px").parse::<f32>() {
            if h > 0.0 && target_h > h {
                target_h = h;
                target_w = target_h / aspect;
            }
        }
    }

    let max_avail_h = height_mm - top_margin - bottom_margin;
    if target_h > max_avail_h {
        target_h = max_avail_h;
        target_w = target_h / aspect;
    }

    if *cursor_y - target_h < bottom_margin {
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
            fonts.helvetica_reg,
            doc,
        );
        *cursor_y = height_mm - top_margin;
    }

    let align = el.attributes.get("align").map(|s| s.as_str()).unwrap_or("center");
    let img_x = match align {
        "left" => left_margin,
        "right" => left_margin + content_width - target_w,
        _ => left_margin + (content_width - target_w) / 2.0,
    };

    let img_bottom_y = *cursor_y - target_h;
    draw_image_on_layer(current_layer, &dyn_img, img_x, img_bottom_y, target_w, target_h);

    *cursor_y = img_bottom_y - 4.0;
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
