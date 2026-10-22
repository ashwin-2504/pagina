#[derive(Debug)]
pub enum ListPrefix {
    Bullet(&'static str, String),
    Number(&'static str, String),
}

#[derive(Debug)]
pub enum PdfBlock {
    Heading(u8, String),
    Paragraph(String),
    Blockquote(String),
    BulletList(String, Vec<String>),
    NumberList(String, Vec<String>),
    AddressBlock(Vec<String>),
}

pub fn is_page_number_or_header(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return true;
    }
    if t.chars().all(|c| c.is_ascii_digit()) && t.len() <= 4 {
        return true;
    }
    if (t.contains(" of ") || t.contains(" OF ")) && t.len() <= 15 {
        return true;
    }
    false
}

pub fn parse_list_item(line: &str) -> Option<ListPrefix> {
    let trimmed = line.trim();
    if trimmed.ends_with(':') {
        return None;
    }

    // Numbered list: "1. ", "2. ", "1) ", "(1) "
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(content) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
                if !content.trim().is_empty() {
                    return Some(ListPrefix::Number("1", content.trim().to_string()));
                }
            }
        }
    }

    // Lowercase alphabetic: "a. ", "b. ", "a) "
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_lowercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(ListPrefix::Number("a", trimmed[3..].trim().to_string()));
    }

    // Uppercase alphabetic: "A. ", "B. ", "A) "
    if chars.len() >= 3
        && chars[0].is_ascii_uppercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(ListPrefix::Number("A", trimmed[3..].trim().to_string()));
    }

    // Roman numerals (lower): "i. ", "ii. ", "iii. "
    let lower_roman_prefixes = [
        "i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. ",
    ];
    for p in &lower_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(ListPrefix::Number(
                "i",
                trimmed[p.len()..].trim().to_string(),
            ));
        }
    }

    // Roman numerals (upper): "I. ", "II. ", "III. "
    let upper_roman_prefixes = [
        "I. ", "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. ",
    ];
    for p in &upper_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(ListPrefix::Number(
                "I",
                trimmed[p.len()..].trim().to_string(),
            ));
        }
    }

    // Bullet: square
    for prefix in &["▪ ", "▪", "\u{25AA} ", "■ ", "\u{25A0} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("square", content.trim().to_string()));
            }
        }
    }

    // Bullet: circle
    for prefix in &["◦ ", "◦", "\u{25E6} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("circle", content.trim().to_string()));
            }
        }
    }

    // Bullet: dash
    for prefix in &["– ", "— ", "- "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("dash", content.trim().to_string()));
            }
        }
    }

    // Bullet: disc (default)
    for prefix in &["• ", "•", "* ", "\u{2022} "] {
        if let Some(content) = trimmed.strip_prefix(prefix) {
            if !content.trim().is_empty() {
                return Some(ListPrefix::Bullet("disc", content.trim().to_string()));
            }
        }
    }

    None
}

pub fn is_heading_candidate(line: &str) -> Option<(u8, String)> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }

    if let Some(h) = t.strip_prefix("### ") {
        return Some((3, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("## ") {
        return Some((2, h.trim().to_string()));
    }
    if let Some(h) = t.strip_prefix("# ") {
        return Some((1, h.trim().to_string()));
    }

    if t.ends_with(':') && t.len() < 55 {
        if t.chars().next().map_or(false, |c| c.is_ascii_digit()) && t.contains(')') {
            return Some((2, t.to_string()));
        }
        return Some((2, t.to_string()));
    }

    let alphabetic_chars: Vec<char> = t.chars().filter(|c| c.is_alphabetic()).collect();
    if alphabetic_chars.len() >= 4
        && alphabetic_chars.iter().all(|c| c.is_uppercase())
        && t.len() < 60
    {
        return Some((1, t.to_string()));
    }

    let lower = t.to_lowercase();
    let common_titles = [
        "abstract",
        "introduction",
        "existing system",
        "proposed system",
        "system architecture",
        "hardware requirements",
        "software requirements",
        "methodology",
        "literature survey",
        "conclusion",
        "references",
        "acknowledgement",
    ];
    for title in &common_titles {
        if lower == *title || lower == format!("{}:", title) {
            return Some((2, t.to_string()));
        }
    }

    None
}
