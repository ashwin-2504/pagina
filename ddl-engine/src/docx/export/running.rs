use super::utils::{escape_xml, length_to_dxa};
use crate::ast::{Document, Node, PageStyle};

pub fn get_active_page_style(doc: &Document) -> Option<&PageStyle> {
    if let Some(ps) = doc.props.page_styles.get("body") {
        return Some(ps);
    }
    doc.props
        .page_styles
        .values()
        .find(|ps| ps.is_default)
        .or_else(|| doc.props.page_styles.values().next())
}

pub fn extract_nodes_plain_text(nodes: &[Node]) -> String {
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

pub fn generate_header_xml(doc: &Document) -> String {
    let ps = get_active_page_style(doc);
    let running_header = match ps.and_then(|p| p.running_header.as_ref()) {
        Some(h) => h,
        None => {
            return r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>"#
                .to_string();
        }
    };

    let left_text = extract_nodes_plain_text(&running_header.left);
    let _right_text = extract_nodes_plain_text(&running_header.right);
    let left_display = if !left_text.is_empty() {
        left_text
    } else {
        extract_nodes_plain_text(&running_header.center)
    };

    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let (mut w, mut _h): (u32, u32) = if is_letter {
        (12240, 15840)
    } else {
        (11906, 16838)
    };
    if is_landscape {
        std::mem::swap(&mut w, &mut _h);
    }
    let margin_dxa = length_to_dxa(&doc.props.paper.margin);
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
        escape_xml(&left_display)
    )
}

pub fn generate_footer_xml(doc: &Document) -> String {
    let ps = get_active_page_style(doc);
    let running_footer = match ps.and_then(|p| p.running_footer.as_ref()) {
        Some(f) => f,
        None => {
            return r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>"#
                .to_string();
        }
    };

    let center_text = extract_nodes_plain_text(&running_footer.center);
    let footer_text = if !center_text.is_empty() {
        center_text
    } else {
        extract_nodes_plain_text(&running_footer.left)
    };

    if footer_text.is_empty() {
        return r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/>"#
            .to_string();
    }

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
        escape_xml(&footer_text)
    )
}
