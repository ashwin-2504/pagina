use crate::ast::{Document, Node, ElementNode};
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
    zip.write_all(STYLES_XML.as_bytes())?;

    // 5. word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = generate_document_xml(doc);
    zip.write_all(doc_xml.as_bytes())?;

    let finished_cursor = zip.finish()?;
    Ok(finished_cursor.into_inner())
}

fn generate_document_xml(doc: &Document) -> String {
    let mut body_xml = String::new();

    // Convert all body children
    for child in &doc.body.children {
        convert_node(child, &mut body_xml);
    }

    // Section properties (A4 default: 11906 x 16838 dxa, margins: 1134 dxa = 20mm)
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let (w, h) = if is_landscape { (16838, 11906) } else { (11906, 16838) };
    let orient_attr = if is_landscape { " w:orient=\"landscape\"" } else { "" };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    {}
    <w:sectPr>
      <w:pgSz w:w="{}" w:h="{}"{}/>
      <w:pgMar w:top="1134" w:right="1134" w:bottom="1134" w:left="1134" w:header="720" w:footer="720"/>
    </w:sectPr>
  </w:body>
</w:document>"#,
        body_xml, w, h, orient_attr
    )
}

fn convert_node(node: &Node, out: &mut String) {
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
                convert_node(child, out);
            }
        }
        Node::Element(el) => {
            let tag = el.tag.to_lowercase();
            match tag.as_str() {
                "h1" => emit_heading(el, "Heading1", out),
                "h2" => emit_heading(el, "Heading2", out),
                "h3" => emit_heading(el, "Heading3", out),
                "h4" => emit_heading(el, "Heading4", out),
                "h5" => emit_heading(el, "Heading5", out),
                "h6" => emit_heading(el, "Heading6", out),
                "p" => {
                    out.push_str("<w:p>");
                    emit_inline_runs(el, false, false, false, false, out);
                    out.push_str("</w:p>\n");
                }
                "ul" => {
                    for li in &el.children {
                        if let Node::Element(li_el) = li {
                            if li_el.tag.to_lowercase() == "li" {
                                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/></w:pPr>");
                                emit_inline_runs(li_el, false, false, false, false, out);
                                out.push_str("</w:p>\n");
                            }
                        }
                    }
                }
                "ol" => {
                    for li in &el.children {
                        if let Node::Element(li_el) = li {
                            if li_el.tag.to_lowercase() == "li" {
                                out.push_str("<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/></w:pPr>");
                                emit_inline_runs(li_el, false, false, false, false, out);
                                out.push_str("</w:p>\n");
                            }
                        }
                    }
                }
                "table" => {
                    emit_table(el, out);
                }
                "callout" => {
                    out.push_str(
                        "<w:p><w:pPr><w:pBdr><w:left w:val=\"single\" w:sz=\"24\" w:space=\"12\" w:color=\"16A34A\"/></w:pBdr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F0FDF4\"/></w:pPr>",
                    );
                    emit_inline_runs(el, false, false, false, false, out);
                    out.push_str("</w:p>\n");
                }
                "hr" => {
                    out.push_str(
                        "<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"CBD5E1\"/></w:pBdr></w:pPr></w:p>\n",
                    );
                }
                _ => {
                    // Container element like div, grid, etc.
                    for child in &el.children {
                        convert_node(child, out);
                    }
                }
            }
        }
    }
}

fn emit_heading(el: &ElementNode, style: &str, out: &mut String) {
    out.push_str(&format!("<w:p><w:pPr><w:pStyle w:val=\"{}\"/></w:pPr>", style));
    emit_inline_runs(el, false, false, false, false, out);
    out.push_str("</w:p>\n");
}

fn emit_inline_runs(
    el: &ElementNode,
    mut bold: bool,
    mut italic: bool,
    mut underline: bool,
    mut code: bool,
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
    }

    for child in &el.children {
        match child {
            Node::Text(t) => {
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
                if !rpr.is_empty() {
                    out.push_str(&format!("<w:rPr>{}</w:rPr>", rpr));
                }
                out.push_str(&format!("<w:t xml:space=\"preserve\">{}</w:t></w:r>", escape_xml(&t.content)));
            }
            Node::Element(sub_el) => {
                emit_inline_runs(sub_el, bold, italic, underline, code, out);
            }
            _ => {}
        }
    }
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

    // Recursively collect rows
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
                    if is_header {
                        out.push_str("        <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F8FAFC\"/>\n");
                    }
                    out.push_str("      </w:tcPr>\n");
                    out.push_str("      <w:p>");
                    emit_inline_runs(cell, is_header, false, false, false, out);
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

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const WORD_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:rPr>
      <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/>
      <w:sz w:val="22"/>
      <w:color w:val="334155"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/>
    <w:rPr>
      <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/>
      <w:b/>
      <w:sz w:val="48"/>
      <w:color w:val="1E3A8A"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2">
    <w:name w:val="heading 2"/>
    <w:rPr>
      <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/>
      <w:b/>
      <w:sz w:val="32"/>
      <w:color w:val="0F172A"/>
    </w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading3">
    <w:name w:val="heading 3"/>
    <w:rPr>
      <w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/>
      <w:b/>
      <w:sz w:val="26"/>
      <w:color w:val="1E293B"/>
    </w:rPr>
  </w:style>
</w:styles>"#;
