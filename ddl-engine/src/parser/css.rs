use crate::ast::*;
use std::collections::HashMap;

pub fn parse_css(css: &str, base_offset: usize) -> Stylesheet {
    let mut rules = Vec::new();
    let parts = css.split('}');

    for part in parts {
        let clean = part.trim();
        if clean.is_empty() {
            continue;
        }

        if let Some((sel_str, decls_str)) = clean.split_once('{') {
            let selectors = sel_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let mut declarations = HashMap::new();
            for decl in decls_str.split(';') {
                if let Some((prop, val)) = decl.split_once(':') {
                    let p = prop.trim().to_string();
                    let v = val.trim().to_string();
                    if !p.is_empty() && !v.is_empty() {
                        declarations.insert(p, v);
                    }
                }
            }

            rules.push(StyleRule {
                selectors,
                declarations,
                span: Span {
                    start: base_offset,
                    end: base_offset + clean.len(),
                    line: 1,
                    col: 1,
                },
            });
        }
    }

    Stylesheet { rules }
}

pub fn parse_inline_styles(style_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for decl in style_str.split(';') {
        if let Some((prop, val)) = decl.split_once(':') {
            let p = prop.trim().to_string();
            let v = val.trim().to_string();
            if !p.is_empty() && !v.is_empty() {
                map.insert(p, v);
            }
        }
    }
    map
}

pub fn parse_length(s: &str) -> Length {
    let s = s.trim();
    if s.ends_with("mm") {
        let val = s.trim_end_matches("mm").trim().parse().unwrap_or(20.0);
        Length {
            val,
            unit: Unit::Mm,
        }
    } else if s.ends_with("pt") {
        let val = s.trim_end_matches("pt").trim().parse().unwrap_or(12.0);
        Length {
            val,
            unit: Unit::Pt,
        }
    } else if s.ends_with("in") {
        let val = s.trim_end_matches("in").trim().parse().unwrap_or(1.0);
        Length {
            val,
            unit: Unit::Inch,
        }
    } else {
        let val = s.parse().unwrap_or(20.0);
        Length {
            val,
            unit: Unit::Mm,
        }
    }
}
