use super::images::{emit_image, ImageEntry};
use super::list::{emit_ordered_list, emit_unordered_list};
use super::styles::{emit_heading, emit_inline_runs};
use super::table::{emit_grid, emit_table};
use super::utils::{
    append_direct_paragraph_formatting, escape_xml, get_p_align, is_inline_node, length_to_dxa,
};
use crate::ast::{Document, Node};
use std::collections::HashMap;

pub fn convert_node(node: &Node, images: &HashMap<String, ImageEntry>, out: &mut String) {
    convert_node_with_indent(node, 0, images, out);
}

pub fn convert_node_with_indent(
    node: &Node,
    indent_level: usize,
    images: &HashMap<String, ImageEntry>,
    out: &mut String,
) {
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
                convert_node_with_indent(child, indent_level, images, out);
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
                    append_direct_paragraph_formatting(el, &mut ppr);
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
                    emit_unordered_list(el, indent_level, images, out, convert_node_with_indent);
                }
                "ol" => {
                    emit_ordered_list(el, indent_level, images, out, convert_node_with_indent);
                }
                "img" => {
                    emit_image(el, images, out);
                }
                "table" => {
                    emit_table(el, out);
                }
                "grid" => {
                    emit_grid(el, out, |child, o| convert_node(child, images, o));
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
                            convert_node_with_indent(child, indent_level, images, out);
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_document_xml(doc: &Document, images: &HashMap<String, ImageEntry>) -> String {
    let mut body_xml = String::new();

    for child in &doc.body.children {
        convert_node(child, images, &mut body_xml);
    }

    let is_landscape = doc.props.paper.orientation.to_lowercase() == "landscape";
    let is_letter = doc.props.paper.size.to_lowercase() == "letter";
    let (mut w, mut h) = if is_letter {
        (12240, 15840)
    } else {
        (11906, 16838)
    };
    if is_landscape {
        std::mem::swap(&mut w, &mut h);
    }
    let orient_attr = if is_landscape {
        " w:orient=\"landscape\""
    } else {
        ""
    };

    let margin_dxa = length_to_dxa(&doc.props.paper.margin);
    let default_running_distance = (margin_dxa / 2).clamp(360, 720);
    let header_dxa = doc
        .props
        .paper
        .header_distance
        .as_ref()
        .map(length_to_dxa)
        .unwrap_or(default_running_distance);
    let footer_dxa = doc
        .props
        .paper
        .footer_distance
        .as_ref()
        .map(length_to_dxa)
        .unwrap_or(default_running_distance);

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
      <w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="{}" w:footer="{}"/>
    </w:sectPr>
  </w:body>
</w:document>"#,
        body_xml, w, h, orient_attr, margin_dxa, margin_dxa, margin_dxa, margin_dxa, header_dxa, footer_dxa
    )
}
