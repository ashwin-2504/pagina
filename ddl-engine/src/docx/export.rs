use crate::ast::{Document, ElementNode, Node, PageStyle};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn export_docx(doc: &Document) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(RELS_XML.as_bytes())?;

    // 3. word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(WORD_RELS_XML.as_bytes())?;

    // 4. word/styles.xml
    zip.start_file("word/styles.xml", options)?;
    let styles_xml = generate_styles_xml(doc);
    zip.write_all(styles_xml.as_bytes())?;

    // 5. word/header1.xml
    zip.start_file("word/header1.xml", options)?;
    let header_xml = generate_header_xml(doc);
    zip.write_all(header_xml.as_bytes())?;

    // 6. word/footer1.xml
    zip.start_file("word/footer1.xml", options)?;
    let footer_xml = generate_footer_xml(doc);
    zip.write_all(footer_xml.as_bytes())?;

    // 7. word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = generate_document_xml(doc);
    zip.write_all(doc_xml.as_bytes())?;

    let finished_cursor = zip.finish()?;
    Ok(finished_cursor.into_inner())
}

fn get_active_page_style(doc: &Document) -> Option<&PageStyle> {
    if let Some(ps) = doc.props.page_styles.get("body") {
        return Some(ps);
    }
    doc.props.page_styles.values().find(|ps| ps.is_default).or_else(|| {
        doc.props.page_styles.values().next()
    })
}

fn extract_nodes_plain_text(nodes: &[Node]) -> String {
    let mut s = String::new();
    for node in nodes {
        match node {
            Node::Text(t) => s.push_str(&t.content),
            Node::Element(el) => s.push_str(&extract_nodes_plain_text(&el.children)),
            _ => {}
        }
    }
    s.trim().to_string()
}

fn generate_header_xml(doc: &Document) -> String {
    let ps = get_active_page_style(doc);
    let left_text = ps
        .and_then(|p| p.running_header.as_ref())
        .map(|h| extract_nodes_plain_text(&h.left))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            doc.props
                .metadata
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Document".to_string())
        });

    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let (mut w, mut _h): (u32, u32) = if is_letter { (12240, 15840) } else { (11906, 16838) };
    if is_landscape {
        std::mem::swap(&mut w, &mut _h);
    }
    let margin_mm = doc.props.paper.margin.val;
    let margin_dxa = (margin_mm * 56.6929).round() as u32;
    let tab_pos = w.saturating_sub(margin_dxa * 2);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:p>
    <w:pPr>
      <w:pBdr>
        <w:bottom w:val="single" w:sz="6" w:space="10" w:color="CBD5E1"/>
      </w:pBdr>
      <w:tabs>
        <w:tab w:val="right" w:pos="{}"/>
      </w:tabs>
      <w:spacing w:after="160"/>
    </w:pPr>
    <w:r>
      <w:rPr>
        <w:rFonts w:ascii="Segoe UI" w:hAnsi="Segoe UI"/>
        <w:sz w:val="18"/>
        <w:color w:val="64748B"/>
      </w:rPr>
      <w:t>{}</w:t>
    </w:r>
    <w:r><w:tab/></w:r>
    <w:fldSimple w:instr="PAGE">
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="Segoe UI" w:hAnsi="Segoe UI"/>
          <w:sz w:val="18"/>
          <w:color w:val="64748B"/>
        </w:rPr>
        <w:t>1</w:t>
      </w:r>
    </w:fldSimple>
    <w:r>
      <w:rPr>
        <w:rFonts w:ascii="Segoe UI" w:hAnsi="Segoe UI"/>
        <w:sz w:val="18"/>
        <w:color w:val="64748B"/>
      </w:rPr>
      <w:t xml:space="preserve"> of </w:t>
    </w:r>
    <w:fldSimple w:instr="NUMPAGES">
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="Segoe UI" w:hAnsi="Segoe UI"/>
          <w:sz w:val="18"/>
          <w:color w:val="64748B"/>
        </w:rPr>
        <w:t>1</w:t>
      </w:r>
    </w:fldSimple>
  </w:p>
</w:hdr>"#,
        tab_pos,
        escape_xml(&left_text)
    )
}

fn generate_footer_xml(doc: &Document) -> String {
    let ps = get_active_page_style(doc);
    let center_text = ps
        .and_then(|p| p.running_footer.as_ref())
        .map(|f| extract_nodes_plain_text(&f.center))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Pagina Typesetting System".to_string());

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:p>
    <w:pPr>
      <w:jc w:val="center"/>
      <w:pBdr>
        <w:top w:val="single" w:sz="6" w:space="8" w:color="E2E8F0"/>
      </w:pBdr>
      <w:spacing w:before="140" w:after="0"/>
    </w:pPr>
    <w:r>
      <w:rPr>
        <w:rFonts w:ascii="Segoe UI" w:hAnsi="Segoe UI"/>
        <w:sz w:val="18"/>
        <w:color w:val="94A3B8"/>
      </w:rPr>
      <w:t>{}</w:t>
    </w:r>
  </w:p>
</w:ftr>"#,
        escape_xml(&center_text)
    )
}

fn generate_document_xml(doc: &Document) -> String {
    let mut body_xml = String::new();

    // Convert all body children
    for child in &doc.body.children {
        convert_node(child, &mut body_xml);
    }

    // Section properties (A4 default: 11906 x 16838 dxa, Letter: 12240 x 15840)
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let (mut w, mut h) = if is_letter { (12240, 15840) } else { (11906, 16838) };
    if is_landscape {
        std::mem::swap(&mut w, &mut h);
    }
    let orient_attr = if is_landscape { " w:orient=\"landscape\"" } else { "" };

    // Dynamic margin in dxa (1mm ≈ 56.7 dxa)
    let margin_mm = doc.props.paper.margin.val;
    let margin_dxa = (margin_mm * 56.6929).round() as u32;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    {}
    <w:sectPr>
      <w:headerReference w:type="default" r:id="rIdH1"/>
      <w:footerReference w:type="default" r:id="rIdF1"/>
      <w:pgSz w:w="{}" w:h="{}"{}/>
      <w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720"/>
    </w:sectPr>
  </w:body>
</w:document>"#,
        body_xml, w, h, orient_attr, margin_dxa, margin_dxa, margin_dxa, margin_dxa
    )
}

fn get_p_align(el: &ElementNode) -> Option<&'static str> {
    if let Some(align) = el.attributes.get("align") {
        match align.to_lowercase().as_str() {
            "center" => return Some("center"),
            "right" => return Some("right"),
            "justify" => return Some("both"),
            _ => {}
        }
    }
    if el.classes.iter().any(|c| c == "text-center") {
        return Some("center");
    }
    if el.classes.iter().any(|c| c == "text-right") {
        return Some("right");
    }
    if el.classes.iter().any(|c| c == "text-justify") {
        return Some("both");
    }
    if let Some(style) = el.inline_styles.get("text-align") {
        match style.to_lowercase().as_str() {
            "center" => return Some("center"),
            "right" => return Some("right"),
            "justify" => return Some("both"),
            _ => {}
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
                    | "page-number"
                    | "total-pages"
            )
        }
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

fn convert_node(node: &Node, out: &mut String) {
    convert_node_with_indent(node, 0, out);
}

fn convert_node_with_indent(node: &Node, indent_level: usize, out: &mut String) {
    match node {
        Node::Text(t) => {
            let trimmed = t.content.trim();
            if !trimmed.is_empty() {
                out.push_str(&format!(
                    "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>\n",
                    escape_xml(&t.content)
                ));
            }
        }
        Node::Comment(_) => {}
        Node::Section(sec) => {
            for child in &sec.children {
                convert_node_with_indent(child, indent_level, out);
            }
        }
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            let is_callout = tag == "callout" || el.classes.iter().any(|c| c == "callout");

            if is_callout {
                out.push_str(
                    "<w:p><w:pPr><w:pBdr><w:left w:val=\"single\" w:sz=\"24\" w:space=\"12\" w:color=\"16A34A\"/></w:pBdr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F0FDF4\"/><w:ind w:left=\"240\" w:right=\"180\"/><w:spacing w:before=\"180\" w:after=\"200\" w:line=\"280\" w:lineRule=\"auto\"/></w:pPr>",
                );
                emit_inline_runs(el, false, false, false, false, false, false, false, out);
                out.push_str("</w:p>\n");
                return;
            }

            match tag.as_str() {
                "h1" => emit_heading(el, "Heading1", out),
                "h2" => emit_heading(el, "Heading2", out),
                "h3" => emit_heading(el, "Heading3", out),
                "h4" => emit_heading(el, "Heading4", out),
                "h5" => emit_heading(el, "Heading5", out),
                "h6" => emit_heading(el, "Heading6", out),
                "p" => {
                    let mut ppr = String::new();
                    let is_lead = el.classes.iter().any(|c| c == "lead");
                    if is_lead {
                        ppr.push_str("<w:rPr><w:sz w:val=\"24\"/><w:color w:val=\"475569\"/></w:rPr><w:spacing w:before=\"60\" w:after=\"180\" w:line=\"280\" w:lineRule=\"auto\"/>");
                    } else {
                        ppr.push_str("<w:spacing w:before=\"40\" w:after=\"140\" w:line=\"260\" w:lineRule=\"auto\"/>");
                    }
                    if let Some(align) = get_p_align(el) {
                        ppr.push_str(&format!("<w:jc w:val=\"{}\"/>", align));
                    }
                    if indent_level > 0 {
                        let ind = (indent_level as u32) * 480;
                        ppr.push_str(&format!("<w:ind w:left=\"{}\"/>", ind));
                    }
                    out.push_str(&format!("<w:p><w:pPr>{}</w:pPr>", ppr));
                    emit_inline_runs(el, false, false, false, false, false, false, false, out);
                    out.push_str("</w:p>\n");
                }
                "blockquote" => {
                    out.push_str(
                        "<w:p><w:pPr><w:pBdr><w:left w:val=\"single\" w:sz=\"18\" w:space=\"12\" w:color=\"CBD5E1\"/></w:pBdr><w:ind w:left=\"720\"/><w:spacing w:before=\"120\" w:after=\"160\" w:line=\"260\" w:lineRule=\"auto\"/></w:pPr>",
                    );
                    emit_inline_runs(el, false, true, false, false, false, false, false, out);
                    out.push_str("</w:p>\n");
                }
                "ul" => {
                    let list_type = el
                        .attributes
                        .get("type")
                        .map(|s| s.as_str())
                        .or_else(|| el.inline_styles.get("list-style-type").map(|s| s.as_str()));
                    let default_bullet = match indent_level % 3 {
                        0 => "•",
                        1 => "◦",
                        _ => "▪",
                    };
                    let bullet_sym = match list_type {
                        Some("circle") => "◦",
                        Some("square") => "▪",
                        Some("dash") => "–",
                        Some("disc") => "•",
                        _ => default_bullet,
                    };
                    let indent_dxa = (indent_level as u32) * 480 + 720;
                    for li in &el.children {
                        if let Node::Element(li_el) = li {
                            if li_el.tag.to_lowercase() == "li" {
                                out.push_str(&format!(
                                    "<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/><w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"1\"/></w:numPr><w:ind w:left=\"{}\" w:hanging=\"360\"/><w:spacing w:after=\"60\" w:line=\"260\" w:lineRule=\"auto\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">{} </w:t></w:r>",
                                    indent_level, indent_dxa, bullet_sym
                                ));
                                emit_inline_runs(li_el, false, false, false, false, false, false, false, out);
                                out.push_str("</w:p>\n");

                                for li_child in &li_el.children {
                                    if let Node::Element(nested_list) = li_child {
                                        let tag_lower = nested_list.tag.to_lowercase();
                                        if tag_lower == "ul" || tag_lower == "ol" {
                                            convert_node_with_indent(
                                                &Node::Element(nested_list.clone()),
                                                indent_level + 1,
                                                out,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "ol" => {
                    let num_type = el
                        .attributes
                        .get("type")
                        .map(|s| s.as_str())
                        .or_else(|| el.inline_styles.get("list-style-type").map(|s| s.as_str()));
                    let default_type = match indent_level % 3 {
                        0 => "1",
                        1 => "a",
                        _ => "i",
                    };
                    let effective_type = num_type.unwrap_or(default_type);
                    let start_idx = el
                        .attributes
                        .get("start")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1);
                    let indent_dxa = (indent_level as u32) * 480 + 720;
                    let mut item_idx = 0;
                    for li in &el.children {
                        if let Node::Element(li_el) = li {
                            if li_el.tag.to_lowercase() == "li" {
                                let num_str = match effective_type {
                                    "a" | "lower-alpha" => format!("{}.", to_lower_alpha(start_idx + item_idx)),
                                    "A" | "upper-alpha" => format!("{}.", to_upper_alpha(start_idx + item_idx)),
                                    "i" | "lower-roman" => format!("{}.", to_roman(start_idx + item_idx)),
                                    "I" | "upper-roman" => format!("{}.", to_roman(start_idx + item_idx).to_uppercase()),
                                    _ => format!("{}.", start_idx + item_idx),
                                };
                                item_idx += 1;
                                out.push_str(&format!(
                                    "<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/><w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"2\"/></w:numPr><w:ind w:left=\"{}\" w:hanging=\"360\"/><w:spacing w:after=\"60\" w:line=\"260\" w:lineRule=\"auto\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">{} </w:t></w:r>",
                                    indent_level, indent_dxa, num_str
                                ));
                                emit_inline_runs(li_el, false, false, false, false, false, false, false, out);
                                out.push_str("</w:p>\n");

                                for li_child in &li_el.children {
                                    if let Node::Element(nested_list) = li_child {
                                        let tag_lower = nested_list.tag.to_lowercase();
                                        if tag_lower == "ul" || tag_lower == "ol" {
                                            convert_node_with_indent(
                                                &Node::Element(nested_list.clone()),
                                                indent_level + 1,
                                                out,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "table" => {
                    emit_table(el, out);
                }
                "grid" => {
                    emit_grid(el, out);
                }
                "hr" => {
                    out.push_str(
                        "<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"CBD5E1\"/></w:pBdr></w:pPr></w:p>\n",
                    );
                }
                _ => {
                    let has_only_inlines = el.children.iter().all(|c| match c {
                        Node::Comment(_) => true,
                        Node::Text(_) => true,
                        Node::Element(sub) => is_inline_node(&Node::Element(sub.clone())),
                        _ => false,
                    });

                    if has_only_inlines && !el.children.is_empty() {
                        out.push_str("<w:p>");
                        emit_inline_runs(el, false, false, false, false, false, false, false, out);
                        out.push_str("</w:p>\n");
                    } else {
                        for child in &el.children {
                            convert_node_with_indent(child, indent_level, out);
                        }
                    }
                }
            }
        }
    }
}

fn emit_heading(el: &ElementNode, style: &str, out: &mut String) {
    let mut ppr = format!("<w:pStyle w:val=\"{}\"/>", style);
    let (before, after) = match style {
        "Heading1" => (160, 160),
        "Heading2" => (240, 120),
        "Heading3" => (100, 80),
        _ => (120, 80),
    };
    ppr.push_str(&format!("<w:spacing w:before=\"{}\" w:after=\"{}\"/>", before, after));
    if let Some(align) = get_p_align(el) {
        ppr.push_str(&format!("<w:jc w:val=\"{}\"/>", align));
    }
    out.push_str(&format!("<w:p><w:pPr>{}</w:pPr>", ppr));
    emit_inline_runs(el, false, false, false, false, false, false, false, out);
    out.push_str("</w:p>\n");
}

fn normalize_inline_whitespace(text: &str, is_first: bool, is_last: bool) -> String {
    let mut result = String::new();
    let mut in_whitespace = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(ch);
            in_whitespace = false;
        }
    }

    if is_first {
        result = result.trim_start().to_string();
    }
    if is_last {
        result = result.trim_end().to_string();
    }

    result
}

fn emit_inline_runs(
    el: &ElementNode,
    mut bold: bool,
    mut italic: bool,
    mut underline: bool,
    mut code: bool,
    mut sup: bool,
    mut sub: bool,
    mut mark: bool,
    out: &mut String,
) {
    let tag = el.tag.to_lowercase();
    if tag == "b" || tag == "strong" {
        bold = true;
    } else if tag == "i" || tag == "em" {
        italic = true;
    } else if tag == "u" {
        underline = true;
    } else if tag == "code" {
        code = true;
    } else if tag == "sup" {
        sup = true;
    } else if tag == "sub" {
        sub = true;
    } else if tag == "mark" {
        mark = true;
    } else if tag == "br" {
        out.push_str("<w:r><w:br/></w:r>");
        return;
    }

    let num_children = el.children.len();
    for (idx, child) in el.children.iter().enumerate() {
        let is_first = idx == 0;
        let is_last = idx == num_children - 1;

        match child {
            Node::Text(t) => {
                let norm = normalize_inline_whitespace(&t.content, is_first, is_last);
                if norm.is_empty() {
                    continue;
                }
                out.push_str("<w:r>");
                let mut rpr = String::new();
                if bold {
                    rpr.push_str("<w:b/>");
                }
                if italic {
                    rpr.push_str("<w:i/>");
                }
                if underline {
                    rpr.push_str("<w:u w:val=\"single\"/>");
                }
                if code {
                    rpr.push_str("<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\"/><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F1F5F9\"/>");
                }
                if sup {
                    rpr.push_str("<w:vertAlign w:val=\"superscript\"/>");
                } else if sub {
                    rpr.push_str("<w:vertAlign w:val=\"subscript\"/>");
                }
                if mark {
                    rpr.push_str("<w:highlight w:val=\"yellow\"/>");
                }
                if !rpr.is_empty() {
                    out.push_str(&format!("<w:rPr>{}</w:rPr>", rpr));
                }
                out.push_str(&format!("<w:t xml:space=\"preserve\">{}</w:t></w:r>", escape_xml(&norm)));
            }
            Node::Element(sub_el) => {
                let sub_tag = sub_el.tag.to_lowercase();
                if sub_tag == "ul" || sub_tag == "ol" {
                    continue;
                }
                emit_inline_runs(sub_el, bold, italic, underline, code, sup, sub, mark, out);
            }
            _ => {}
        }
    }
}

fn emit_grid(el: &ElementNode, out: &mut String) {
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
    let gap_pct = if n > 1 { 200 } else { 0 };
    let total_gap = (n - 1) * gap_pct;
    let col_pct = ((5000 - total_gap) / n).max(500);

    out.push_str("<w:tbl>\n");
    out.push_str(
        r#"  <w:tblPr>
    <w:tblW w:w="5000" w:type="pct"/>
    <w:tblBorders>
      <w:top w:val="nil"/>
      <w:left w:val="nil"/>
      <w:bottom w:val="nil"/>
      <w:right w:val="nil"/>
      <w:insideH w:val="nil"/>
      <w:insideV w:val="nil"/>
    </w:tblBorders>
  </w:tblPr>
  <w:tr>
"#,
    );

    for (i, col) in col_divs.iter().enumerate() {
        if i > 0 && gap_pct > 0 {
            // Spacer column
            out.push_str(&format!(
                r#"    <w:tc>
      <w:tcPr>
        <w:tcW w:w="{}" w:type="pct"/>
        <w:tcBorders>
          <w:top w:val="nil"/>
          <w:left w:val="nil"/>
          <w:bottom w:val="nil"/>
          <w:right w:val="nil"/>
        </w:tcBorders>
      </w:tcPr>
      <w:p/>
    </w:tc>
"#,
                gap_pct
            ));
        }

        let is_boxed = col.classes.iter().any(|c| c == "col-box");
        let box_styling = if is_boxed {
            r#"        <w:tcBorders>
          <w:top w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:left w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:bottom w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
          <w:right w:val="single" w:sz="6" w:space="0" w:color="E2E8F0"/>
        </w:tcBorders>
        <w:shd w:val="clear" w:color="auto" w:fill="F8FAFC"/>
        <w:tcMar>
          <w:top w:w="180" w:type="dxa"/>
          <w:left w:w="220" w:type="dxa"/>
          <w:bottom w:w="180" w:type="dxa"/>
          <w:right w:w="220" w:type="dxa"/>
        </w:tcMar>"#
        } else {
            r#"        <w:tcBorders>
          <w:top w:val="nil"/>
          <w:left w:val="nil"/>
          <w:bottom w:val="nil"/>
          <w:right w:val="nil"/>
        </w:tcBorders>
        <w:tcMar>
          <w:top w:w="120" w:type="dxa"/>
          <w:left w:w="120" w:type="dxa"/>
          <w:bottom w:w="120" w:type="dxa"/>
          <w:right w:w="120" w:type="dxa"/>
        </w:tcMar>"#
        };

        out.push_str(&format!(
            r#"    <w:tc>
      <w:tcPr>
        <w:tcW w:w="{}" w:type="pct"/>
{}
      </w:tcPr>
"#,
            col_pct, box_styling
        ));

        for child in &col.children {
            convert_node(child, out);
        }

        out.push_str("    </w:tc>\n");
    }

    out.push_str("  </w:tr>\n</w:tbl>\n");
}

fn emit_table(el: &ElementNode, out: &mut String) {
    out.push_str("<w:tbl>\n");
    out.push_str(r#"  <w:tblPr>
    <w:tblW w:w="5000" w:type="pct"/>
    <w:tblBorders>
      <w:top w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:left w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:bottom w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:right w:val="single" w:sz="4" w:space="0" w:color="CBD5E1"/>
      <w:insideH w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/>
      <w:insideV w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/>
    </w:tblBorders>
  </w:tblPr>
"#);

    let mut rows: Vec<&ElementNode> = Vec::new();
    collect_rows(el, &mut rows);

    for tr in rows {
        out.push_str("  <w:tr>\n");
        for child in &tr.children {
            if let Node::Element(cell) = child {
                let cell_tag = cell.tag.to_lowercase();
                if cell_tag == "td" || cell_tag == "th" {
                    let is_header = cell_tag == "th";
                    out.push_str("    <w:tc>\n");
                    out.push_str("      <w:tcPr>\n");
                    out.push_str(
                        r#"        <w:tcMar>
          <w:top w:w="140" w:type="dxa"/>
          <w:left w:w="180" w:type="dxa"/>
          <w:bottom w:w="140" w:type="dxa"/>
          <w:right w:w="180" w:type="dxa"/>
        </w:tcMar>
"#,
                    );
                    if is_header {
                        out.push_str("        <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F8FAFC\"/>\n");
                    }
                    out.push_str("      </w:tcPr>\n");
                    out.push_str("      <w:p>");
                    emit_inline_runs(cell, is_header, false, false, false, false, false, false, out);
                    out.push_str("</w:p>\n");
                    out.push_str("    </w:tc>\n");
                }
            }
        }
        out.push_str("  </w:tr>\n");
    }

    out.push_str("</w:tbl>\n");
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

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn detect_doc_font_family(doc: &Document) -> &'static str {
    for rule in &doc.props.stylesheet.rules {
        if rule.selectors.iter().any(|s| s == "body") {
            if let Some(ff) = rule.declarations.get("font-family") {
                let ffl = ff.to_lowercase();
                if ffl.contains("times") || ffl.contains("serif") || ffl.contains("georgia") {
                    return "Times New Roman";
                }
                if ffl.contains("arial") || ffl.contains("helvetica") {
                    return "Arial";
                }
                if ffl.contains("calibri") {
                    return "Calibri";
                }
            }
        }
    }
    "Segoe UI"
}

fn generate_styles_xml(doc: &Document) -> String {
    let font_name = detect_doc_font_family(doc);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:pPr>
      <w:spacing w:after="140" w:line="260" w:lineRule="auto"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:sz w:val="21"/>
      <w:color w:val="334155"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:pPr>
      <w:spacing w:before="160" w:after="160"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:b/>
      <w:sz w:val="48"/>
      <w:color w:val="1E3A8A"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:pPr>
      <w:spacing w:before="240" w:after="120"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:b/>
      <w:sz w:val="32"/>
      <w:color w:val="0F172A"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:pPr>
      <w:spacing w:before="100" w:after="80"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:b/>
      <w:sz w:val="26"/>
      <w:color w:val="1E293B"/>
    </w:rPr>
  </w:style>
</w:styles>"#,
        font = font_name
    )
}

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>
  <Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>
</Types>"#;

const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const WORD_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rIdH1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
  <Relationship Id="rIdF1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
</Relationships>"#;
