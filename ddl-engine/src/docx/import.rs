use crate::ImportResult;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::{Cursor, Read};

#[derive(Debug, Clone)]
struct InlineToken {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
    code: bool,
    sup: bool,
    sub: bool,
    highlight: bool,
}

#[derive(Debug)]
enum ImportedBlock {
    Heading(u8, Option<String>, String),
    Paragraph(Option<String>, bool, String),
    Callout(String),
    Blockquote(String),
    BulletList(String, Vec<String>),
    NumberList(String, Vec<String>),
    Table(Vec<Vec<Vec<String>>>),
    Grid(Vec<Vec<String>>),
}

fn format_coalesced_inlines(tokens: &[InlineToken]) -> String {
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
            {
                last.text.push_str(&tok.text);
                continue;
            }
        }
        merged.push(tok.clone());
    }

    let mut out = String::new();
    for tok in merged {
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
            s = format!("<mark>{}</mark>", s);
        }
        out.push_str(&s);
    }
    out
}

fn detect_bullet_item(trimmed: &str) -> Option<(&'static str, String)> {
    if let Some(rest) = trimmed.strip_prefix('•').or_else(|| trimmed.strip_prefix('\u{2022}')) {
        return Some(("disc", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix('◦').or_else(|| trimmed.strip_prefix('\u{25E6}')) {
        return Some(("circle", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix('▪').or_else(|| trimmed.strip_prefix('\u{25AA}')).or_else(|| trimmed.strip_prefix('■')) {
        return Some(("square", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix('–').or_else(|| trimmed.strip_prefix('—')) {
        return Some(("dash", rest.trim_start().to_string()));
    }
    if trimmed.starts_with("- ") {
        return Some(("dash", trimmed[2..].trim_start().to_string()));
    }
    None
}

fn strip_bullet_prefix(text: &str) -> String {
    let trimmed = text.trim();

    // 1. Direct bullet prefixes
    for b in &["•", "\u{2022}", "◦", "\u{25E6}", "▪", "\u{25AA}", "■", "–", "—", "- "] {
        if let Some(rest) = trimmed.strip_prefix(b) {
            return rest.trim_start().to_string();
        }
    }

    // 2. Formatted bullet prefixes like <b>• </b> or <strong>• </strong>
    for (start_tag, end_tag) in &[("<b>", "</b>"), ("<strong>", "</strong>"), ("<i>", "</i>"), ("<em>", "</em>")] {
        if let Some(rest) = trimmed.strip_prefix(start_tag) {
            for b in &["•", "\u{2022}", "◦", "\u{25E6}", "▪", "\u{25AA}", "■", "–", "—", "-"] {
                let with_space = format!("{} ", b);
                let variations = [with_space.as_str(), *b];
                for var in &variations {
                    if let Some(after_b) = rest.strip_prefix(var) {
                        let after_trimmed = after_b.trim_start();
                        if let Some(after_end) = after_trimmed.strip_prefix(end_tag) {
                            return after_end.trim_start().to_string();
                        }
                    }
                }
            }
        }
    }

    trimmed.to_string()
}

fn is_number_prefix_token(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if let Some(first) = t.chars().next() {
        if first.is_ascii_digit() {
            let rest = t.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest == "." || rest == ")" || rest == ".-" || rest == "-" {
                return true;
            }
        }
    }
    if (t.len() == 2 || t.len() == 3)
        && t.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
        && (t.ends_with('.') || t.ends_with(')'))
    {
        return true;
    }
    let roman = [
        "i.", "ii.", "iii.", "iv.", "v.", "vi.", "vii.", "viii.", "ix.", "x.",
        "i)", "ii)", "iii)", "iv)", "v)",
        "I.", "II.", "III.", "IV.", "V.", "VI.", "VII.", "VIII.", "IX.", "X.",
        "I)", "II)", "III)", "IV)", "V)",
    ];
    roman.contains(&t)
}

fn strip_number_prefix(text: &str) -> String {
    let trimmed = text.trim();

    // 1. Formatted number prefixes like <b>1. </b> or <strong>1. </strong>
    for (start_tag, end_tag) in &[("<b>", "</b>"), ("<strong>", "</strong>"), ("<i>", "</i>"), ("<em>", "</em>")] {
        if let Some(rest) = trimmed.strip_prefix(start_tag) {
            if let Some(end_idx) = rest.find(end_tag) {
                let inner = &rest[..end_idx];
                let after_tag = &rest[end_idx + end_tag.len()..];
                if is_number_prefix_token(inner.trim()) {
                    return after_tag.trim_start().to_string();
                }
            }
        }
    }

    // 2. Direct decimal prefix
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(content) = rest.strip_prefix(". ")
                .or_else(|| rest.strip_prefix(") "))
                .or_else(|| rest.strip_prefix(" - "))
            {
                return content.trim_start().to_string();
            }
        }
    }

    // 3. Alpha prefixes
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3 && chars[0].is_ascii_alphabetic() && (chars[1] == '.' || chars[1] == ')') && chars[2].is_whitespace() {
        return trimmed[3..].trim_start().to_string();
    }

    // 4. Roman numeral prefixes
    let roman_prefixes = [
        "i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. ",
        "I. ", "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. ",
    ];
    for p in &roman_prefixes {
        if trimmed.starts_with(p) {
            return trimmed[p.len()..].trim_start().to_string();
        }
    }

    trimmed.to_string()
}

fn detect_number_item(trimmed: &str) -> Option<(&'static str, String)> {
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(content) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
                return Some(("1", content.trim().to_string()));
            }
        }
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3 && chars[0].is_ascii_lowercase() && (chars[1] == '.' || chars[1] == ')') && chars[2].is_whitespace() {
        return Some(("a", trimmed[3..].trim().to_string()));
    }
    if chars.len() >= 3 && chars[0].is_ascii_uppercase() && (chars[1] == '.' || chars[1] == ')') && chars[2].is_whitespace() {
        return Some(("A", trimmed[3..].trim().to_string()));
    }
    let lower_roman_prefixes = ["i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. "];
    for p in &lower_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(("i", trimmed[p.len()..].trim().to_string()));
        }
    }
    let upper_roman_prefixes = ["I. ", "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. "];
    for p in &upper_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(("I", trimmed[p.len()..].trim().to_string()));
        }
    }
    None
}

#[derive(Debug, Clone)]
enum ListFormat {
    Bullet(String), // "disc", "circle", "square", "dash"
    Number(String), // "1", "a", "A", "i", "I"
}

#[derive(Default)]
struct NumberingInfo {
    num_to_abstract: std::collections::HashMap<String, String>,
    abstract_lvl_format: std::collections::HashMap<(String, u32), ListFormat>,
}

fn parse_numbering_xml(xml: &str) -> NumberingInfo {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut num_to_abstract = std::collections::HashMap::new();
    let mut abstract_lvl_format = std::collections::HashMap::new();

    let mut current_abstract_num_id: Option<String> = None;
    let mut current_ilvl: Option<u32> = None;
    let mut current_num_id: Option<String> = None;
    let mut current_num_fmt: Option<String> = None;
    let mut current_lvl_text: Option<String> = None;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:abstractNum" => {
                        current_abstract_num_id = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:abstractNumId" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_abstract_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:lvl" => {
                        current_ilvl = None;
                        current_num_fmt = None;
                        current_lvl_text = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:ilvl" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_ilvl = val.parse::<u32>().ok();
                                }
                            }
                        }
                    }
                    b"w:numFmt" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_num_fmt = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:lvlText" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_lvl_text = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:num" => {
                        current_num_id = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:numId" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:abstractNumId" => {
                        if let Some(ref nid) = current_num_id {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        num_to_abstract.insert(nid.clone(), val.to_string());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:lvl" => {
                        if let (Some(aid), Some(ilvl)) = (&current_abstract_num_id, current_ilvl) {
                            let fmt = match current_num_fmt.as_deref().unwrap_or("bullet") {
                                "bullet" => {
                                    let b_type = match current_lvl_text.as_deref() {
                                        Some(txt) if txt.contains('o') || txt.contains('◦') => "circle",
                                        Some(txt) if txt.contains('▪') || txt.contains('■') || txt.contains('§') => "square",
                                        Some(txt) if txt.contains('–') || txt.contains('-') => "dash",
                                        _ => "disc",
                                    };
                                    ListFormat::Bullet(b_type.to_string())
                                }
                                "decimal" => ListFormat::Number("1".to_string()),
                                "lowerLetter" => ListFormat::Number("a".to_string()),
                                "upperLetter" => ListFormat::Number("A".to_string()),
                                "lowerRoman" => ListFormat::Number("i".to_string()),
                                "upperRoman" => ListFormat::Number("I".to_string()),
                                other => {
                                    if other.to_lowercase().contains("bullet") {
                                        ListFormat::Bullet("disc".to_string())
                                    } else {
                                        ListFormat::Number("1".to_string())
                                    }
                                }
                            };
                            abstract_lvl_format.insert((aid.clone(), ilvl), fmt);
                        }
                        current_ilvl = None;
                        current_num_fmt = None;
                        current_lvl_text = None;
                    }
                    b"w:abstractNum" => {
                        current_abstract_num_id = None;
                    }
                    b"w:num" => {
                        current_num_id = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    NumberingInfo {
        num_to_abstract,
        abstract_lvl_format,
    }
}

pub fn import_docx(bytes: &[u8]) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;

    let numbering_info = {
        let mut num_xml = String::new();
        if let Ok(mut file) = archive.by_name("word/numbering.xml") {
            let _ = file.read_to_string(&mut num_xml);
        }
        if !num_xml.is_empty() {
            parse_numbering_xml(&num_xml)
        } else {
            NumberingInfo::default()
        }
    };

    let mut doc_xml = String::new();
    {
        let mut file = archive.by_name("word/document.xml")?;
        file.read_to_string(&mut doc_xml)?;
    }

    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(false);

    let mut blocks: Vec<ImportedBlock> = Vec::new();

    let mut current_p_style: Option<String> = None;
    let mut current_p_align: Option<String> = None;
    let mut current_p_is_callout = false;
    let mut current_p_num_id: Option<String> = None;
    let mut current_p_ilvl: u32 = 0;
    let mut current_p_tokens: Vec<InlineToken> = Vec::new();
    let mut font_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let mut in_r = false;
    let mut in_t = false;
    let mut in_p_pr = false;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_underline = false;
    let mut is_code = false;
    let mut is_sup = false;
    let mut is_sub = false;
    let mut is_highlight = false;

    // Paper settings
    let mut paper_size = "A4".to_string();
    let mut paper_orientation = "portrait".to_string();
    let mut paper_margin_mm = 20.0f32;

    // Table state
    let mut in_tbl = false;
    let mut current_table_rows: Vec<Vec<Vec<String>>> = Vec::new();
    let mut current_row: Vec<Vec<String>> = Vec::new();
    let mut current_cell_paragraphs: Vec<String> = Vec::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:tbl" => {
                        in_tbl = true;
                        current_table_rows.clear();
                    }
                    b"w:tr" => {
                        current_row.clear();
                    }
                    b"w:tc" => {
                        current_cell_paragraphs.clear();
                    }
                    b"w:p" => {
                        current_p_style = None;
                        current_p_align = None;
                        current_p_is_callout = false;
                        current_p_num_id = None;
                        current_p_ilvl = 0;
                        current_p_tokens.clear();
                    }
                    b"w:numId" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:ilvl" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_ilvl = val.parse::<u32>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    b"w:pPr" => {
                        in_p_pr = true;
                    }
                    b"w:r" => {
                        in_r = true;
                        is_bold = false;
                        is_italic = false;
                        is_underline = false;
                        is_code = false;
                        is_sup = false;
                        is_sub = false;
                        is_highlight = false;
                    }
                    b"w:t" => {
                        in_t = true;
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:pStyle" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_style = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:numId" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_num_id = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:ilvl" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_ilvl = val.parse::<u32>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    b"w:jc" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_align = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"w:b" => {
                        if in_r {
                            let mut disabled = false;
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if attr.value.as_ref() == b"0"
                                        || attr.value.as_ref() == b"false"
                                        || attr.value.as_ref() == b"none"
                                    {
                                        disabled = true;
                                    }
                                }
                            }
                            if !disabled {
                                is_bold = true;
                            }
                        }
                    }
                    b"w:i" => {
                        if in_r {
                            let mut disabled = false;
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if attr.value.as_ref() == b"0"
                                        || attr.value.as_ref() == b"false"
                                        || attr.value.as_ref() == b"none"
                                    {
                                        disabled = true;
                                    }
                                }
                            }
                            if !disabled {
                                is_italic = true;
                            }
                        }
                    }
                    b"w:u" => {
                        if in_r {
                            let mut is_val_underline = true;
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if attr.value.as_ref() == b"none" || attr.value.as_ref() == b"nil" || attr.value.as_ref() == b"0" {
                                        is_val_underline = false;
                                    }
                                }
                            }
                            if is_val_underline {
                                is_underline = true;
                            }
                        }
                    }
                    b"w:highlight" => {
                        if in_r {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        if val != "none" && val != "nil" && val != "0" && !val.is_empty() {
                                            is_highlight = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:rFonts" => {
                        if in_r {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:ascii" || attr.key.as_ref() == b"w:hAnsi" {
                                    if let Ok(font_name) = std::str::from_utf8(&attr.value) {
                                        let f_lower = font_name.to_lowercase();
                                        if f_lower.contains("consolas")
                                            || f_lower.contains("courier")
                                            || f_lower.contains("mono")
                                            || f_lower.contains("code")
                                        {
                                            is_code = true;
                                        } else if !font_name.trim().is_empty() {
                                            *font_counts.entry(font_name.to_string()).or_insert(0) += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:shd" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:fill" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    let fill_lower = val.to_lowercase();
                                    if in_r && (fill_lower.starts_with("f1f5") || fill_lower.starts_with("e2e8")) {
                                        is_code = true;
                                    }
                                    if in_p_pr && (fill_lower.starts_with("f0fd") || fill_lower.starts_with("dcfc")) {
                                        current_p_is_callout = true;
                                    }
                                }
                            }
                        }
                    }
                    b"w:left" => {
                        if in_p_pr {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:color" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let col_lower = val.to_lowercase();
                                        if col_lower.starts_with("16a") || col_lower.starts_with("22c") {
                                            current_p_is_callout = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:vertAlign" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:val" {
                                if attr.value.as_ref() == b"superscript" {
                                    is_sup = true;
                                } else if attr.value.as_ref() == b"subscript" {
                                    is_sub = true;
                                }
                            }
                        }
                    }
                    b"w:br" => {
                        current_p_tokens.push(InlineToken {
                            text: "<br />".to_string(),
                            bold: false,
                            italic: false,
                            underline: false,
                            code: false,
                            sup: false,
                            sub: false,
                            highlight: false,
                        });
                    }
                    b"w:pgSz" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:orient" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    paper_orientation = val.to_string();
                                }
                            } else if attr.key.as_ref() == b"w:w" {
                                if let Ok(w_str) = std::str::from_utf8(&attr.value) {
                                    if let Ok(w_val) = w_str.parse::<u32>() {
                                        if (12200..=12300).contains(&w_val) {
                                            paper_size = "Letter".to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:pgMar" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:top" {
                                if let Ok(m_str) = std::str::from_utf8(&attr.value) {
                                    if let Ok(m_val) = m_str.parse::<f32>() {
                                        let mm = (m_val * 0.0176389).round();
                                        if (10.0..=35.0).contains(&mm) {
                                            paper_margin_mm = mm;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(e) => {
                if in_t {
                    let text = e.unescape()?.to_string();
                    if !text.is_empty() {
                        current_p_tokens.push(InlineToken {
                            text,
                            bold: is_bold,
                            italic: is_italic,
                            underline: is_underline,
                            code: is_code,
                            sup: is_sup,
                            sub: is_sub,
                            highlight: is_highlight,
                        });
                    }
                }
            }
            Event::End(e) => {
                let name = e.name();
                match name.as_ref() {
                    b"w:t" => {
                        in_t = false;
                    }
                    b"w:r" => {
                        in_r = false;
                    }
                    b"w:pPr" => {
                        in_p_pr = false;
                    }
                    b"w:p" => {
                        let formatted_line = format_coalesced_inlines(&current_p_tokens);
                        let trimmed = formatted_line.trim();

                        if !trimmed.is_empty() {
                            let style_lower = current_p_style.as_deref().unwrap_or("").to_lowercase();
                            let align = current_p_align.clone();

                            let list_format_opt = if let Some(ref nid) = current_p_num_id {
                                if nid != "0" {
                                    if let Some(abs_id) = numbering_info.num_to_abstract.get(nid) {
                                        numbering_info.abstract_lvl_format.get(&(abs_id.clone(), current_p_ilvl)).cloned()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if in_tbl {
                                current_cell_paragraphs.push(formatted_line);
                            } else if current_p_is_callout || style_lower.contains("callout") {
                                blocks.push(ImportedBlock::Callout(formatted_line));
                            } else if style_lower.contains("heading 1") || style_lower == "1" || style_lower == "heading1" {
                                blocks.push(ImportedBlock::Heading(1, align, formatted_line));
                            } else if style_lower.contains("heading 2") || style_lower == "2" || style_lower == "heading2" {
                                blocks.push(ImportedBlock::Heading(2, align, formatted_line));
                            } else if style_lower.contains("heading 3") || style_lower == "3" || style_lower == "heading3" {
                                blocks.push(ImportedBlock::Heading(3, align, formatted_line));
                            } else if let Some(fmt) = list_format_opt {
                                match fmt {
                                    ListFormat::Bullet(b_type) => {
                                        let clean_text = strip_bullet_prefix(trimmed);
                                        if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                            if cur_type == &b_type {
                                                items.push(clean_text);
                                            } else {
                                                blocks.push(ImportedBlock::BulletList(b_type, vec![clean_text]));
                                            }
                                        } else {
                                            blocks.push(ImportedBlock::BulletList(b_type, vec![clean_text]));
                                        }
                                    }
                                    ListFormat::Number(n_type) => {
                                        let clean_text = strip_number_prefix(trimmed);
                                        if let Some(ImportedBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                                            if cur_type == &n_type {
                                                items.push(clean_text);
                                            } else {
                                                blocks.push(ImportedBlock::NumberList(n_type, vec![clean_text]));
                                            }
                                        } else {
                                            blocks.push(ImportedBlock::NumberList(n_type, vec![clean_text]));
                                        }
                                    }
                                }
                            } else if current_p_num_id.is_some() && current_p_num_id.as_deref() != Some("0") {
                                if style_lower.contains("bullet") {
                                    let clean_text = strip_bullet_prefix(trimmed);
                                    if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                        if cur_type == "disc" {
                                            items.push(clean_text);
                                        } else {
                                            blocks.push(ImportedBlock::BulletList("disc".to_string(), vec![clean_text]));
                                        }
                                    } else {
                                        blocks.push(ImportedBlock::BulletList("disc".to_string(), vec![clean_text]));
                                    }
                                } else if style_lower.contains("number") || style_lower.contains("decimal") {
                                    let clean_text = strip_number_prefix(trimmed);
                                    if let Some(ImportedBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                                        if cur_type == "1" {
                                            items.push(clean_text);
                                        } else {
                                            blocks.push(ImportedBlock::NumberList("1".to_string(), vec![clean_text]));
                                        }
                                    } else {
                                        blocks.push(ImportedBlock::NumberList("1".to_string(), vec![clean_text]));
                                    }
                                } else if let Some((b_type, clean_text)) = detect_bullet_item(trimmed) {
                                    let final_text = strip_bullet_prefix(&clean_text);
                                    if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                        if cur_type == b_type {
                                            items.push(final_text);
                                        } else {
                                            blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![final_text]));
                                        }
                                    } else {
                                        blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![final_text]));
                                    }
                                } else if let Some((n_type, clean_text)) = detect_number_item(trimmed) {
                                    let final_text = strip_number_prefix(&clean_text);
                                    if let Some(ImportedBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                                        if cur_type == n_type {
                                            items.push(final_text);
                                        } else {
                                            blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![final_text]));
                                        }
                                    } else {
                                        blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![final_text]));
                                    }
                                } else {
                                    let clean_text = strip_bullet_prefix(trimmed);
                                    if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                        if cur_type == "disc" {
                                            items.push(clean_text);
                                        } else {
                                            blocks.push(ImportedBlock::BulletList("disc".to_string(), vec![clean_text]));
                                        }
                                    } else {
                                        blocks.push(ImportedBlock::BulletList("disc".to_string(), vec![clean_text]));
                                    }
                                }
                            } else if style_lower.contains("listbullet") {
                                let clean_text = strip_bullet_prefix(trimmed);
                                let (b_type, text_to_use) = detect_bullet_item(&clean_text).unwrap_or(("disc", clean_text));
                                if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                    if cur_type == b_type {
                                        items.push(text_to_use);
                                    } else {
                                        blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![text_to_use]));
                                    }
                                } else {
                                    blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![text_to_use]));
                                }
                            } else if style_lower.contains("listnumber") || style_lower.contains("numbered") {
                                let clean_text = strip_number_prefix(trimmed);
                                let (n_type, text_to_use) = detect_number_item(&clean_text).unwrap_or(("1", clean_text));
                                if let Some(ImportedBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                                    if cur_type == n_type {
                                        items.push(text_to_use);
                                    } else {
                                        blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![text_to_use]));
                                    }
                                } else {
                                    blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![text_to_use]));
                                }
                            } else if let Some((b_type, clean_text)) = detect_bullet_item(trimmed) {
                                let final_text = strip_bullet_prefix(&clean_text);
                                if let Some(ImportedBlock::BulletList(cur_type, items)) = blocks.last_mut() {
                                    if cur_type == b_type {
                                        items.push(final_text);
                                    } else {
                                        blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![final_text]));
                                    }
                                } else {
                                    blocks.push(ImportedBlock::BulletList(b_type.to_string(), vec![final_text]));
                                }
                            } else if let Some((n_type, clean_text)) = detect_number_item(trimmed) {
                                let final_text = strip_number_prefix(&clean_text);
                                if let Some(ImportedBlock::NumberList(cur_type, items)) = blocks.last_mut() {
                                    if cur_type == n_type {
                                        items.push(final_text);
                                    } else {
                                        blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![final_text]));
                                    }
                                } else {
                                    blocks.push(ImportedBlock::NumberList(n_type.to_string(), vec![final_text]));
                                }
                            } else if style_lower.contains("quote") {
                                blocks.push(ImportedBlock::Blockquote(formatted_line));
                            } else {
                                // Intelligent heading detection heuristic:
                                // Short title with all bold tokens and no terminating period
                                let all_bold = !current_p_tokens.is_empty() && current_p_tokens.iter().all(|t| t.bold);
                                let is_short = trimmed.len() < 45;
                                let is_title = all_bold && is_short && !trimmed.ends_with('.');

                                if is_title {
                                    blocks.push(ImportedBlock::Heading(2, align, formatted_line));
                                } else {
                                    let is_lead = style_lower.contains("lead");
                                    blocks.push(ImportedBlock::Paragraph(align, is_lead, formatted_line));
                                }
                            }
                        }
                    }
                    b"w:tc" => {
                        current_row.push(current_cell_paragraphs.clone());
                        current_cell_paragraphs.clear();
                    }
                    b"w:tr" => {
                        current_table_rows.push(current_row.clone());
                        current_row.clear();
                    }
                    b"w:tbl" => {
                        in_tbl = false;

                        // Check if this table is actually a multi-column card grid:
                        // 1 row, 2-3 non-empty cells
                        let is_single_row = current_table_rows.len() == 1;
                        let non_empty_cells: Vec<Vec<String>> = if is_single_row {
                            current_table_rows[0]
                                .iter()
                                .filter(|cell_ps| !cell_ps.is_empty())
                                .cloned()
                                .collect()
                        } else {
                            Vec::new()
                        };

                        if is_single_row && non_empty_cells.len() >= 2 && non_empty_cells.len() <= 3 {
                            blocks.push(ImportedBlock::Grid(non_empty_cells));
                        } else if !current_table_rows.is_empty() {
                            blocks.push(ImportedBlock::Table(current_table_rows.clone()));
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    // Convert structured blocks into clean DXML
    let mut body_nodes: Vec<String> = Vec::new();

    for block in blocks {
        match block {
            ImportedBlock::Heading(level, align, text) => {
                let align_attr = align.map(|a| format!(" align=\"{}\"", a)).unwrap_or_default();
                body_nodes.push(format!("      <h{}{}>{}</h{}>", level, align_attr, text, level));
            }
            ImportedBlock::Paragraph(align, is_lead, text) => {
                let mut attrs = String::new();
                if is_lead {
                    attrs.push_str(" class=\"lead\"");
                }
                if let Some(a) = align {
                    attrs.push_str(&format!(" align=\"{}\"", a));
                }
                body_nodes.push(format!("      <p{}>{}</p>", attrs, text));
            }
            ImportedBlock::Callout(text) => {
                body_nodes.push(format!(
                    "      <div class=\"callout\">\n        {}\n      </div>",
                    text
                ));
            }
            ImportedBlock::Blockquote(text) => {
                body_nodes.push(format!(
                    "      <blockquote>\n        <p>{}</p>\n      </blockquote>",
                    text
                ));
            }
            ImportedBlock::BulletList(b_type, items) => {
                let type_attr = if b_type != "disc" {
                    format!(" type=\"{}\"", b_type)
                } else {
                    String::new()
                };
                let mut list_xml = format!("      <ul{}>\n", type_attr);
                for item in items {
                    list_xml.push_str(&format!("        <li>{}</li>\n", item));
                }
                list_xml.push_str("      </ul>");
                body_nodes.push(list_xml);
            }
            ImportedBlock::NumberList(n_type, items) => {
                let type_attr = if n_type != "1" {
                    format!(" type=\"{}\"", n_type)
                } else {
                    String::new()
                };
                let mut list_xml = format!("      <ol{}>\n", type_attr);
                for item in items {
                    list_xml.push_str(&format!("        <li>{}</li>\n", item));
                }
                list_xml.push_str("      </ol>");
                body_nodes.push(list_xml);
            }
            ImportedBlock::Grid(cols) => {
                let cols_spec = vec!["1fr"; cols.len()].join(" ");
                let mut grid_xml = format!("      <grid cols=\"{}\" gap=\"6mm\">\n", cols_spec);
                for col_ps in cols {
                    grid_xml.push_str("        <div class=\"col-box\">\n");
                    for p in col_ps {
                        grid_xml.push_str(&format!("          <p>{}</p>\n", p));
                    }
                    grid_xml.push_str("        </div>\n");
                }
                grid_xml.push_str("      </grid>");
                body_nodes.push(grid_xml);
            }
            ImportedBlock::Table(rows) => {
                let mut tbl_xml = String::from("      <table>\n");
                for (row_idx, row) in rows.iter().enumerate() {
                    tbl_xml.push_str("        <tr>\n");
                    for cell_ps in row {
                        let tag = if row_idx == 0 { "th" } else { "td" };
                        let content = if cell_ps.is_empty() {
                            String::new()
                        } else {
                            cell_ps.join("<br/>")
                        };
                        tbl_xml.push_str(&format!("          <{}>{}</{}>\n", tag, content.trim(), tag));
                    }
                    tbl_xml.push_str("        </tr>\n");
                }
                tbl_xml.push_str("      </table>");
                body_nodes.push(tbl_xml);
            }
        }
    }

    let dxml = format!(
        r#"<doc>
  <props>
    <meta>
      <title>Imported OpenXML Document</title>
      <author>Pagina Reverse-Engineering Engine</author>
    </meta>
    <paper size="{}" orientation="{}" margin="{}mm" />
    <link rel="stylesheet" href="styles.dcss" />
    <page-styles>
      <page-style name="body" default="true">
        <running-header>
          <left><text>Imported Document</text></left>
          <right><page-number/> of <total-pages/></right>
        </running-header>
        <running-footer>
          <center><text>Pagina Engine</text></center>
        </running-footer>
      </page-style>
    </page-styles>
  </props>

  <body>
    <section page-style="body">
{}
    </section>
  </body>
</doc>"#,
        paper_size,
        paper_orientation,
        paper_margin_mm,
        if body_nodes.is_empty() {
            "      <p>Empty document</p>".to_string()
        } else {
            body_nodes.join("\n")
        }
    );

    let primary_font = font_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(name, _)| name)
        .unwrap_or_else(|| "Inter, system-ui, sans-serif".to_string());

    let dcss = format!(
        r#"/* Generated by Pagina OpenXML Reverse Engine */
body {{
  font-family: "{primary_font}", system-ui, sans-serif;
  color: #334155;
}}

h1 {{
  font-size: 24pt;
  font-weight: bold;
  color: #1e3a8a;
  margin-bottom: 5mm;
}}

h2 {{
  font-size: 16pt;
  font-weight: bold;
  color: #0f172a;
  margin-top: 6mm;
  margin-bottom: 3mm;
}}

h3 {{
  font-size: 12pt;
  font-weight: 600;
  color: #1e293b;
  margin-bottom: 2mm;
}}

p {{
  font-size: 10.5pt;
  line-height: 1.6;
  color: #334155;
  margin-bottom: 3.5mm;
}}

p.lead {{
  font-size: 12pt;
  color: #475569;
}}

blockquote {{
  margin-left: 6mm;
  padding-left: 4mm;
  border-left: 3px solid #cbd5e1;
  color: #475569;
  margin-bottom: 4mm;
}}

ul, ol {{
  padding-left: 24px;
  margin-top: 2mm;
  margin-bottom: 3.5mm;
}}

li {{
  margin-bottom: 1.5mm;
  line-height: 1.6;
}}

.callout {{
  background-color: #f0fdf4;
  border-left: 4px solid #16a34a;
  padding: 4mm 6mm;
  margin: 4mm 0;
  border-radius: 4px;
}}

.col-box {{
  background-color: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 4mm;
}}

code {{
  font-family: Consolas, monospace;
  background-color: #f1f5f9;
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 9.5pt;
}}

table {{
  width: 100%;
  border-collapse: collapse;
  margin: 4mm 0;
}}

th, td {{
  border: 1px solid #cbd5e1;
  padding: 2.5mm 3.5mm;
  text-align: left;
}}

th {{
  background-color: #f8fafc;
  font-weight: bold;
}}

p[align="center"], h1[align="center"], h2[align="center"] {{
  text-align: center;
}}

p[align="right"] {{
  text-align: right;
}}

p[align="justify"] {{
  text-align: justify;
}}
"#,
        primary_font = primary_font
    );

    Ok(ImportResult { dxml, dcss })
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
