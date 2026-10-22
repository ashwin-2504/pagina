use super::types::{escape_xml, ImportedTextStyle, InlineToken};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

pub fn format_coalesced_inlines(tokens: &[InlineToken]) -> String {
    let mut merged: Vec<InlineToken> = Vec::new();
    for tok in tokens {
        if tok.text.is_empty() {
            continue;
        }
        if let Some(last) = merged.last_mut() {
            if last.bold == tok.bold
                && last.italic == tok.italic
                && last.underline == tok.underline
                && last.code == tok.code
                && last.sup == tok.sup
                && last.sub == tok.sub
                && last.highlight == tok.highlight
                && last.font_family == tok.font_family
                && last.font_size_half_points == tok.font_size_half_points
                && last.color == tok.color
                && last.highlight_color == tok.highlight_color
            {
                last.text.push_str(&tok.text);
                continue;
            }
        }
        merged.push(tok.clone());
    }

    let mut out = String::new();
    for tok in merged {
        if tok.text == "<br />" {
            out.push_str("<br />");
            continue;
        }
        let mut s = escape_xml(&tok.text);
        if tok.code {
            s = format!("<code>{}</code>", s);
        }
        if tok.bold && tok.italic {
            s = format!("<b><i>{}</i></b>", s);
        } else if tok.bold {
            s = format!("<b>{}</b>", s);
        } else if tok.italic {
            s = format!("<i>{}</i>", s);
        }
        if tok.underline {
            s = format!("<u>{}</u>", s);
        }
        if tok.sup {
            s = format!("<sup>{}</sup>", s);
        } else if tok.sub {
            s = format!("<sub>{}</sub>", s);
        }
        if tok.highlight {
            let style = tok
                .highlight_color
                .as_ref()
                .map(|color| format!(" style=\"background-color: #{};\"", color))
                .unwrap_or_default();
            s = format!("<mark{}>{}</mark>", style, s);
        }
        let mut styles = Vec::new();
        if let Some(font) = tok.font_family {
            styles.push(format!("font-family: '{}';", font.replace('\'', "\\'")));
        }
        if let Some(size) = tok.font_size_half_points {
            styles.push(format!("font-size: {}pt;", size as f32 / 2.0));
        }
        if let Some(color) = tok.color {
            styles.push(format!("color: #{};", color));
        }
        if !styles.is_empty() {
            s = format!("<span style=\"{}\">{}</span>", styles.join(" "), s);
        }
        out.push_str(&s);
    }
    out
}

pub fn paragraph_style(
    before_twips: Option<u32>,
    after_twips: Option<u32>,
    line_twips: Option<u32>,
    left_twips: Option<u32>,
    right_twips: Option<u32>,
    first_line_twips: Option<i32>,
) -> String {
    let mut declarations = Vec::new();
    let twips_to_mm = |twips: u32| twips as f32 * 25.4 / 1440.0;
    if let Some(value) = before_twips {
        declarations.push(format!("margin-top: {:.2}mm;", twips_to_mm(value)));
    }
    if let Some(value) = after_twips {
        declarations.push(format!("margin-bottom: {:.2}mm;", twips_to_mm(value)));
    }
    if let Some(value) = line_twips {
        declarations.push(format!("line-height: {:.2};", value as f32 / 240.0));
    }
    if let Some(value) = left_twips {
        declarations.push(format!("margin-left: {:.2}mm;", twips_to_mm(value)));
    }
    if let Some(value) = right_twips {
        declarations.push(format!("margin-right: {:.2}mm;", twips_to_mm(value)));
    }
    if let Some(value) = first_line_twips {
        declarations.push(format!(
            "text-indent: {:.2}mm;",
            value as f32 * 25.4 / 1440.0
        ));
    }
    declarations.join(" ")
}

pub fn parse_document_styles(xml: &str) -> HashMap<String, ImportedTextStyle> {
    let mut reader = Reader::from_str(xml);
    let mut styles = HashMap::new();
    let mut current_style: Option<String> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"w:style" {
                    current_style = e.attributes().flatten().find_map(|attr| {
                        (attr.key.as_ref() == b"w:styleId")
                            .then(|| std::str::from_utf8(&attr.value).ok().map(str::to_string))
                            .flatten()
                    });
                    if let Some(style) = &current_style {
                        styles
                            .entry(style.clone())
                            .or_insert_with(ImportedTextStyle::default);
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let Some(style_id) = current_style.as_ref() else {
                    buf.clear();
                    continue;
                };
                let style = styles
                    .entry(style_id.clone())
                    .or_insert_with(ImportedTextStyle::default);
                match e.name().as_ref() {
                    b"w:rFonts" => {
                        if let Some(font) = e.attributes().flatten().find_map(|attr| {
                            (attr.key.as_ref() == b"w:ascii" || attr.key.as_ref() == b"w:hAnsi")
                                .then(|| std::str::from_utf8(&attr.value).ok().map(str::to_string))
                                .flatten()
                        }) {
                            style.font_family = Some(font);
                        }
                    }
                    b"w:sz" | b"w:szCs" => {
                        if let Some(size) = e.attributes().flatten().find_map(|attr| {
                            (attr.key.as_ref() == b"w:val")
                                .then(|| {
                                    std::str::from_utf8(&attr.value)
                                        .ok()
                                        .and_then(|v| v.parse().ok())
                                })
                                .flatten()
                        }) {
                            style.font_size_half_points = Some(size);
                        }
                    }
                    b"w:color" => {
                        if let Some(color) = e.attributes().flatten().find_map(|attr| {
                            (attr.key.as_ref() == b"w:val")
                                .then(|| std::str::from_utf8(&attr.value).ok().map(str::to_string))
                                .flatten()
                        }) {
                            if color != "auto" {
                                style.color = Some(color);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"w:style" => current_style = None,
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    styles
}
