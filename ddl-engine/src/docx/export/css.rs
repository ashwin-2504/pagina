use crate::ast::Document;

pub fn css_value(doc: &Document, selector: &str, property: &str) -> Option<String> {
    let mut value = None;
    for rule in &doc.props.stylesheet.rules {
        if rule
            .selectors
            .iter()
            .any(|candidate| candidate.trim() == selector)
        {
            if let Some(candidate) = rule.declarations.get(property) {
                value = Some(candidate.trim().to_string());
            }
        }
    }
    value
}

pub fn css_color(value: Option<String>, fallback: &str) -> String {
    let value = value.unwrap_or_else(|| fallback.to_string());
    let clean = value.trim().trim_start_matches('#');
    if clean.len() == 6 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        clean.to_ascii_uppercase()
    } else {
        fallback.to_string()
    }
}

pub fn css_half_points(value: Option<String>, fallback: u32) -> u32 {
    value
        .and_then(|v| {
            v.trim()
                .strip_suffix("pt")
                .map(str::trim)
                .and_then(|n| n.parse::<f32>().ok())
        })
        .map(|points| (points * 2.0).round().max(2.0) as u32)
        .unwrap_or(fallback)
}

pub fn css_line_twips(value: Option<String>, fallback: u32) -> u32 {
    value
        .and_then(|v| v.trim().parse::<f32>().ok())
        .map(|line_height| (line_height * 240.0).round().max(1.0) as u32)
        .unwrap_or(fallback)
}

pub fn detect_doc_font_family(doc: &Document) -> String {
    let family =
        css_value(doc, "body", "font-family").or_else(|| css_value(doc, "doc", "font-family"));
    if let Some(family) = family {
        let first = family
            .split(',')
            .next()
            .unwrap_or(&family)
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if !first.is_empty() {
            return first.to_string();
        }
    }
    for rule in &doc.props.stylesheet.rules {
        if rule.selectors.iter().any(|s| s == "body") {
            if let Some(ff) = rule.declarations.get("font-family") {
                let ffl = ff.to_lowercase();
                if ffl.contains("times") || ffl.contains("serif") || ffl.contains("georgia") {
                    return "Times New Roman".to_string();
                }
                if ffl.contains("arial") || ffl.contains("helvetica") {
                    return "Arial".to_string();
                }
                if ffl.contains("calibri") {
                    return "Calibri".to_string();
                }
            }
        }
    }
    "Segoe UI".to_string()
}
