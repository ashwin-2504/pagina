use crate::ast::{ElementNode, Length, Node, Unit};

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn length_to_dxa(length: &Length) -> u32 {
    let dxa_per_unit = match length.unit {
        Unit::Mm => 1440.0 / 25.4,
        Unit::Cm => 1440.0 / 2.54,
        Unit::Inch => 1440.0,
        Unit::Pt => 20.0,
        Unit::Px => 15.0,
        Unit::Em => 240.0,
        Unit::Percent => 0.0,
    };
    (length.val.max(0.0) * dxa_per_unit).round() as u32
}

pub fn get_p_align(el: &ElementNode) -> Option<&'static str> {
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

pub fn css_length_to_twips(value: &str) -> Option<i32> {
    let value = value.trim();
    let (number, multiplier) = if let Some(number) = value.strip_suffix("mm") {
        (number, 1440.0 / 25.4)
    } else if let Some(number) = value.strip_suffix("cm") {
        (number, 1440.0 / 2.54)
    } else if let Some(number) = value.strip_suffix("in") {
        (number, 1440.0)
    } else if let Some(number) = value.strip_suffix("pt") {
        (number, 20.0)
    } else if let Some(number) = value.strip_suffix("px") {
        (number, 15.0)
    } else {
        return None;
    };
    number
        .trim()
        .parse::<f32>()
        .ok()
        .map(|number| (number * multiplier).round() as i32)
}

pub fn append_direct_paragraph_formatting(el: &ElementNode, ppr: &mut String) {
    let before = el
        .inline_styles
        .get("margin-top")
        .and_then(|v| css_length_to_twips(v));
    let after = el
        .inline_styles
        .get("margin-bottom")
        .and_then(|v| css_length_to_twips(v));
    let line = el.inline_styles.get("line-height").and_then(|v| {
        v.trim()
            .parse::<f32>()
            .ok()
            .map(|value| (value * 240.0).round() as i32)
    });
    if before.is_some() || after.is_some() || line.is_some() {
        let before = before.unwrap_or(0).max(0);
        let after = after.unwrap_or(0).max(0);
        let line = line.unwrap_or(0).max(0);
        if line > 0 {
            ppr.push_str(&format!("<w:spacing w:before=\"{before}\" w:after=\"{after}\" w:line=\"{line}\" w:lineRule=\"auto\"/>"));
        } else {
            ppr.push_str(&format!(
                "<w:spacing w:before=\"{before}\" w:after=\"{after}\"/>"
            ));
        }
    }
    let left = el
        .inline_styles
        .get("margin-left")
        .and_then(|v| css_length_to_twips(v));
    let right = el
        .inline_styles
        .get("margin-right")
        .and_then(|v| css_length_to_twips(v));
    let first = el
        .inline_styles
        .get("text-indent")
        .and_then(|v| css_length_to_twips(v));
    if left.is_some() || right.is_some() || first.is_some() {
        let mut attrs = String::new();
        if let Some(value) = left {
            attrs.push_str(&format!(" w:left=\"{}\"", value.max(0)));
        }
        if let Some(value) = right {
            attrs.push_str(&format!(" w:right=\"{}\"", value.max(0)));
        }
        if let Some(value) = first {
            if value < 0 {
                attrs.push_str(&format!(" w:hanging=\"{}\"", -value));
            } else {
                attrs.push_str(&format!(" w:firstLine=\"{}\"", value));
            }
        }
        ppr.push_str(&format!("<w:ind{attrs}/>"));
    }
}

pub fn is_inline_node(node: &Node) -> bool {
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

pub fn normalize_inline_whitespace(text: &str, is_first: bool, is_last: bool) -> String {
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
