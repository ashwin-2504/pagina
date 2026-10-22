use std::collections::HashMap;
use quick_xml::events::BytesStart;

use super::document_attrs::attr_val;
use super::types::InlineToken;

#[derive(Default)]
pub struct DocumentRunState {
    pub in_r: bool,
    pub in_t: bool,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub is_code: bool,
    pub is_sup: bool,
    pub is_sub: bool,
    pub is_highlight: bool,
    pub current_font_family: Option<String>,
    pub current_font_size_half_points: Option<u32>,
    pub current_font_color: Option<String>,
    pub current_highlight_color: Option<String>,
}

impl DocumentRunState {
    pub fn reset_run(&mut self) {
        self.in_r = true;
        self.is_bold = false;
        self.is_italic = false;
        self.is_underline = false;
        self.is_code = false;
        self.is_sup = false;
        self.is_sub = false;
        self.is_highlight = false;
        self.current_font_family = None;
        self.current_font_size_half_points = None;
        self.current_font_color = None;
        self.current_highlight_color = None;
    }

    pub fn make_token(&self, text: String) -> InlineToken {
        InlineToken {
            text,
            bold: self.is_bold,
            italic: self.is_italic,
            underline: self.is_underline,
            code: self.is_code,
            sup: self.is_sup,
            sub: self.is_sub,
            highlight: self.is_highlight,
            font_family: self.current_font_family.clone(),
            font_size_half_points: self.current_font_size_half_points,
            color: self.current_font_color.clone(),
            highlight_color: self.current_highlight_color.clone(),
        }
    }
}

pub fn handle_run_element(
    name: &[u8],
    e: &BytesStart,
    state: &mut DocumentRunState,
    font_counts: &mut HashMap<String, usize>,
    font_size_counts: &mut HashMap<u32, usize>,
    font_color_counts: &mut HashMap<String, usize>,
) {
    if !state.in_r {
        return;
    }

    match name {
        b"w:b" => {
            let v = attr_val(e);
            state.is_bold = !matches!(v.as_deref(), Some("0") | Some("false") | Some("none"));
        }
        b"w:i" => {
            let v = attr_val(e);
            state.is_italic = !matches!(v.as_deref(), Some("0") | Some("false") | Some("none"));
        }
        b"w:u" => {
            let v = attr_val(e);
            state.is_underline = !matches!(v.as_deref(), Some("0") | Some("none") | Some("nil"));
        }
        b"w:highlight" => {
            if let Some(val) = attr_val(e) {
                if val != "none" && val != "nil" && val != "0" && !val.is_empty() {
                    state.is_highlight = true;
                    state.current_highlight_color = Some(val);
                }
            }
        }
        b"w:rFonts" => {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"w:ascii" || attr.key.as_ref() == b"w:hAnsi" {
                    if let Ok(font_name) = std::str::from_utf8(&attr.value) {
                        state.current_font_family = Some(font_name.to_string());
                        let f_lower = font_name.to_lowercase();
                        if f_lower.contains("consolas")
                            || f_lower.contains("courier")
                            || f_lower.contains("mono")
                            || f_lower.contains("code")
                        {
                            state.is_code = true;
                        } else if !font_name.trim().is_empty() {
                            *font_counts.entry(font_name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        b"w:sz" | b"w:szCs" => {
            if let Some(val) = attr_val(e) {
                if let Ok(half_pts) = val.parse::<u32>() {
                    *font_size_counts.entry(half_pts).or_insert(0) += 1;
                    state.current_font_size_half_points = Some(half_pts);
                }
            }
        }
        b"w:color" => {
            if let Some(val) = attr_val(e) {
                let v = val.trim();
                if v != "auto" && !v.is_empty() {
                    state.current_font_color = Some(v.to_string());
                }
                if v != "auto" && !v.is_empty() && v != "000000" {
                    *font_color_counts.entry(format!("#{}", v)).or_insert(0) += 1;
                }
            }
        }
        b"w:shd" => {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"w:fill" {
                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                        let fill = val.to_lowercase();
                        if fill.starts_with("f1f5") || fill.starts_with("e2e8") {
                            state.is_code = true;
                        }
                    }
                }
            }
        }
        b"w:vertAlign" => {
            if let Some(v) = attr_val(e) {
                if v == "superscript" {
                    state.is_sup = true;
                } else if v == "subscript" {
                    state.is_sub = true;
                }
            }
        }
        _ => {}
    }
}
