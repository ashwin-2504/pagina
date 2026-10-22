use std::collections::HashMap;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::relationships::extract_pt_from_style;
use super::types::{escape_xml, ParsedGenericPart, PartImg};

fn is_disabled(e: &BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:val" {
            let v = attr.value.as_ref();
            if v == b"0" || v == b"false" || v == b"none" || v == b"nil" {
                return true;
            }
        }
    }
    false
}

fn record_image(
    r_id: &str,
    cx: Option<u64>,
    cy: Option<u64>,
    pos_offset: Option<i64>,
    vml_style: &str,
    images: &mut Vec<PartImg>,
) {
    if !images.iter().any(|im| im.r_id == r_id) {
        images.push(PartImg {
            r_id: r_id.to_string(),
            cx,
            cy,
            pos_offset,
            vml_style: vml_style.to_string(),
        });
    }
}

pub fn parse_generic_part(
    xml_str: &str,
    rels: &HashMap<String, String>,
    is_footer: bool,
) -> ParsedGenericPart {
    let mut part = ParsedGenericPart::default();
    let mut reader = Reader::from_str(xml_str);
    reader.config_mut().trim_text(false);

    let mut current_p_align: Option<String> = None;
    let mut current_p_texts: Vec<String> = Vec::new();
    let mut current_p_images: Vec<PartImg> = Vec::new();

    let mut in_r = false;
    let mut in_t = false;
    let mut in_pos_offset = false;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_underline = false;
    let mut current_text = String::new();

    let mut current_cx: Option<u64> = None;
    let mut current_cy: Option<u64> = None;
    let mut current_pos_offset: Option<i64> = None;
    let mut current_vml_style = String::new();

    let mut in_instr_text = false;
    let mut instr_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"w:p" => {
                    current_p_align = None;
                    current_p_texts.clear();
                    current_p_images.clear();
                    current_cx = None;
                    current_cy = None;
                    current_pos_offset = None;
                    current_vml_style.clear();
                }
                b"w:jc" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                current_p_align = Some(v.to_string());
                            }
                        }
                    }
                }
                b"w:r" => {
                    in_r = true;
                    is_bold = false;
                    is_italic = false;
                    is_underline = false;
                }
                b"w:b" if in_r => is_bold = !is_disabled(e),
                b"w:i" if in_r => is_italic = !is_disabled(e),
                b"w:u" if in_r => is_underline = !is_disabled(e),
                b"w:t" => {
                    in_t = true;
                    current_text.clear();
                }
                b"w:instrText" => {
                    in_instr_text = true;
                    instr_text.clear();
                }
                b"w:fldSimple" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:instr" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                let upper = val.to_uppercase();
                                if upper.contains("PAGE") && !upper.contains("NUMPAGES") {
                                    current_p_texts.push("<page-number/>".to_string());
                                } else if upper.contains("NUMPAGES") {
                                    current_p_texts.push("<total-pages/>".to_string());
                                }
                            }
                        }
                    }
                }
                b"wp:posOffset" => in_pos_offset = true,
                b"wp:extent" | b"a:ext" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"cx" {
                            if let Ok(s) = std::str::from_utf8(&attr.value) {
                                current_cx = s.parse::<u64>().ok();
                            }
                        } else if attr.key.as_ref() == b"cy" {
                            if let Ok(s) = std::str::from_utf8(&attr.value) {
                                current_cy = s.parse::<u64>().ok();
                            }
                        }
                    }
                }
                b"a:blip" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"r:embed" {
                            if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                record_image(
                                    rid,
                                    current_cx,
                                    current_cy,
                                    current_pos_offset,
                                    &current_vml_style,
                                    &mut current_p_images,
                                );
                            }
                        }
                    }
                }
                b"v:shape" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"style" {
                            if let Ok(st) = std::str::from_utf8(&attr.value) {
                                current_vml_style = st.to_string();
                            }
                        }
                    }
                }
                b"v:imagedata" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"r:id" {
                            if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                record_image(
                                    rid,
                                    current_cx,
                                    current_cy,
                                    current_pos_offset,
                                    &current_vml_style,
                                    &mut current_p_images,
                                );
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_t {
                    if let Ok(t) = e.unescape() {
                        current_text.push_str(&t);
                    }
                } else if in_instr_text {
                    if let Ok(t) = e.unescape() {
                        instr_text.push_str(&t);
                    }
                } else if in_pos_offset {
                    if let Ok(t) = e.unescape() {
                        if let Ok(val) = t.trim().parse::<i64>() {
                            current_pos_offset = Some(val);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"w:t" => {
                    in_t = false;
                    if !current_text.trim().is_empty() {
                        let mut s = escape_xml(&current_text);
                        if is_bold {
                            s = format!("<b>{}</b>", s);
                        }
                        if is_italic {
                            s = format!("<i>{}</i>", s);
                        }
                        if is_underline {
                            s = format!("<u>{}</u>", s);
                        }
                        current_p_texts.push(s);
                    }
                }
                b"w:instrText" => {
                    in_instr_text = false;
                    let upper = instr_text.to_uppercase();
                    if upper.contains("PAGE") && !upper.contains("NUMPAGES") {
                        current_p_texts.push("<page-number/>".to_string());
                    } else if upper.contains("NUMPAGES") {
                        current_p_texts.push("<total-pages/>".to_string());
                    }
                }
                b"wp:posOffset" => in_pos_offset = false,
                b"w:r" => in_r = false,
                b"w:p" => {
                    for img in &current_p_images {
                        if let Some(fname) = rels.get(&img.r_id) {
                            let mut height_mm = img.cy.map(|cy| (cy as f32) / 914400.0 * 25.4);
                            if height_mm.is_none() && !img.vml_style.is_empty() {
                                if let Some(h_pt) = extract_pt_from_style(&img.vml_style, "height:") {
                                    height_mm = Some(h_pt * 25.4 / 72.0);
                                }
                            }
                            if let Some(h) = height_mm {
                                part.max_image_height_mm = Some(
                                    part.max_image_height_mm.map(|p| p.max(h)).unwrap_or(h),
                                );
                            }

                            let base_class = if is_footer { "footer-image" } else { "header-image" };
                            let img_dxml = format!(r#"<img class="{}" src="{}" />"#, base_class, fname);

                            if current_p_align.as_deref() == Some("left") {
                                part.left_items.push(img_dxml);
                            } else if current_p_align.as_deref() == Some("right") {
                                part.right_items.push(img_dxml);
                            } else {
                                part.center_items.push(img_dxml);
                            }
                        }
                    }

                    if !current_p_texts.is_empty() {
                        let text_content = current_p_texts.join(" ");
                        let dxml_text = format!("<text>{}</text>", text_content);
                        if current_p_align.as_deref() == Some("right")
                            || (current_p_texts.iter().any(|t| t.contains("page-number"))
                                && current_p_align.is_none())
                        {
                            part.right_items.push(dxml_text);
                        } else if current_p_align.as_deref() == Some("center") {
                            part.center_items.push(dxml_text);
                        } else {
                            part.left_items.push(dxml_text);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    part
}
