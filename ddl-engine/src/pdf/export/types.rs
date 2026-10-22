use crate::ast::{ElementNode, Length, Node, Unit};
use super::fonts::FontFamily;

#[derive(Debug, Clone)]
pub struct StyledRun {
    pub text: String,
    pub family: FontFamily,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub struct ParagraphWord {
    pub text: String,
    pub family: FontFamily,
    pub bold: bool,
    pub italic: bool,
    pub has_trailing_space: bool,
    pub width_mm: f32,
}

#[derive(Debug, Clone)]
pub struct LineSpan {
    pub text: String,
    pub family: FontFamily,
    pub bold: bool,
    pub italic: bool,
    pub font_size: f32,
    pub width_mm: f32,
}

pub fn length_to_mm(length: &Length) -> f32 {
    match length.unit {
        Unit::Mm => length.val,
        Unit::Cm => length.val * 10.0,
        Unit::Inch => length.val * 25.4,
        Unit::Pt => length.val * 25.4 / 72.0,
        Unit::Px => length.val * 25.4 / 96.0,
        Unit::Em => length.val * 4.233,
        Unit::Percent => length.val,
    }
}

pub fn get_align(el: &ElementNode) -> Option<&'static str> {
    if let Some(align) = el.attributes.get("align") {
        match align.to_lowercase().as_str() {
            "center" => return Some("center"),
            "right" => return Some("right"),
            "justify" => return Some("justify"),
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
        return Some("justify");
    }
    None
}

pub fn to_lower_alpha(mut n: usize) -> String {
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

pub fn to_upper_alpha(n: usize) -> String {
    to_lower_alpha(n).to_uppercase()
}

pub fn to_roman(mut n: usize) -> String {
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

pub fn extract_nodes_text(nodes: &[Node]) -> String {
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

pub fn extract_plain_text(el: &ElementNode) -> String {
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
