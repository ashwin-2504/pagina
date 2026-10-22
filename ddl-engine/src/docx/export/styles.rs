use super::css::{css_color, css_half_points, css_line_twips, css_value, detect_doc_font_family};
use super::utils::{
    append_direct_paragraph_formatting, escape_xml, get_p_align, normalize_inline_whitespace,
};
use crate::ast::{Document, ElementNode, Node};

pub fn generate_styles_xml(doc: &Document) -> String {
    let font_name = detect_doc_font_family(doc);
    let body_size = css_half_points(
        css_value(doc, "body", "font-size").or_else(|| css_value(doc, "doc", "font-size")),
        21,
    );
    let body_color = css_color(
        css_value(doc, "body", "color").or_else(|| css_value(doc, "doc", "color")),
        "334155",
    );
    let body_line = css_line_twips(
        css_value(doc, "body", "line-height").or_else(|| css_value(doc, "doc", "line-height")),
        260,
    );
    let h1_size = css_half_points(css_value(doc, "h1", "font-size"), 48);
    let h1_color = css_color(css_value(doc, "h1", "color"), "1E3A8A");
    let h2_size = css_half_points(css_value(doc, "h2", "font-size"), 32);
    let h2_color = css_color(css_value(doc, "h2", "color"), "0F172A");
    let h3_size = css_half_points(css_value(doc, "h3", "font-size"), 26);
    let h3_color = css_color(css_value(doc, "h3", "color"), "1E293B");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
    <w:pPr>
      <w:spacing w:after="140" w:line="{body_line}" w:lineRule="auto"/>
    </w:pPr>
    <w:rPr>
      <w:rFonts w:ascii="{font}" w:hAnsi="{font}" w:cs="{font}"/>
      <w:sz w:val="{body_size}"/>
      <w:color w:val="{body_color}"/>
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
      <w:sz w:val="{h1_size}"/>
      <w:color w:val="{h1_color}"/>
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
      <w:sz w:val="{h2_size}"/>
      <w:color w:val="{h2_color}"/>
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
      <w:sz w:val="{h3_size}"/>
      <w:color w:val="{h3_color}"/>
    </w:rPr>
  </w:style>
</w:styles>"#,
        font = escape_xml(&font_name),
        body_line = body_line,
        body_size = body_size,
        body_color = body_color,
        h1_size = h1_size,
        h1_color = h1_color,
        h2_size = h2_size,
        h2_color = h2_color,
        h3_size = h3_size,
        h3_color = h3_color
    )
}

pub fn inline_style_rpr(el: &ElementNode) -> String {
    let mut rpr = String::new();
    if let Some(font) = el.inline_styles.get("font-family") {
        let font = font
            .split(',')
            .next()
            .unwrap_or(font)
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if !font.is_empty() {
            let font = escape_xml(font);
            rpr.push_str(&format!(
                "<w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:cs=\"{font}\"/>"
            ));
        }
    }
    if let Some(size) = el.inline_styles.get("font-size") {
        if let Some(points) = size
            .trim()
            .strip_suffix("pt")
            .and_then(|value| value.trim().parse::<f32>().ok())
        {
            rpr.push_str(&format!(
                "<w:sz w:val=\"{}\"/>",
                (points * 2.0).round().max(2.0) as u32
            ));
        }
    }
    if let Some(color) = el.inline_styles.get("color") {
        let color = css_color(Some(color.clone()), "000000");
        rpr.push_str(&format!("<w:color w:val=\"{color}\"/>"));
    }
    if let Some(background) = el.inline_styles.get("background-color") {
        let fill = css_color(Some(background.clone()), "FFFFFF");
        rpr.push_str(&format!(
            "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"{fill}\"/>"
        ));
    }
    rpr
}

pub fn emit_heading(el: &ElementNode, style: &str, out: &mut String) {
    let mut ppr = format!("<w:pStyle w:val=\"{}\"/>", style);
    let (before, after) = match style {
        "Heading1" => (160, 160),
        "Heading2" => (240, 120),
        "Heading3" => (100, 80),
        _ => (120, 80),
    };
    ppr.push_str(&format!(
        "<w:spacing w:before=\"{}\" w:after=\"{}\"/>",
        before, after
    ));
    if let Some(align) = get_p_align(el) {
        ppr.push_str(&format!("<w:jc w:val=\"{}\"/>", align));
    }
    append_direct_paragraph_formatting(el, &mut ppr);
    out.push_str(&format!("<w:p><w:pPr>{}</w:pPr>", ppr));
    emit_inline_runs(el, false, false, false, false, false, false, false, out);
    out.push_str("</w:p>\n");
}

pub fn emit_inline_runs(
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
    if el
        .inline_styles
        .get("font-weight")
        .map(|v| v == "bold" || v.parse::<u32>().map(|n| n >= 600).unwrap_or(false))
        .unwrap_or(false)
    {
        bold = true;
    }
    if el
        .inline_styles
        .get("font-style")
        .map(|v| v.eq_ignore_ascii_case("italic") || v.eq_ignore_ascii_case("oblique"))
        .unwrap_or(false)
    {
        italic = true;
    }
    if el
        .inline_styles
        .get("text-decoration")
        .map(|v| v.contains("underline"))
        .unwrap_or(false)
    {
        underline = true;
    }
    let style_rpr = inline_style_rpr(el);

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
                rpr.push_str(&style_rpr);
                if !rpr.is_empty() {
                    out.push_str(&format!("<w:rPr>{}</w:rPr>", rpr));
                }
                out.push_str(&format!(
                    "<w:t xml:space=\"preserve\">{}</w:t></w:r>",
                    escape_xml(&norm)
                ));
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
