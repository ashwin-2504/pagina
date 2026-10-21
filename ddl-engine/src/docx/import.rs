use crate::ImportResult;
use base64::Engine;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
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
    font_family: Option<String>,
    font_size_half_points: Option<u32>,
    color: Option<String>,
    highlight_color: Option<String>,
}

#[derive(Debug, Clone)]
struct FlatListItem {
    ilvl: u32,
    is_bullet: bool,
    list_type: String,
    text: String,
}

#[derive(Debug)]
struct ItemNode {
    text: String,
    sublists: Vec<ListNode>,
}

#[derive(Debug)]
struct ListNode {
    is_bullet: bool,
    list_type: String,
    items: Vec<ItemNode>,
}

fn build_list_tree(items: &[FlatListItem]) -> Vec<ListNode> {
    fn parse_items_at_level(
        items: &[FlatListItem],
        idx: &mut usize,
        current_level: u32,
    ) -> Vec<ListNode> {
        let mut lists: Vec<ListNode> = Vec::new();

        while *idx < items.len() {
            let item = &items[*idx];
            if item.ilvl < current_level {
                break;
            }

            if item.ilvl > current_level {
                if let Some(cur_list) = lists.last_mut() {
                    if let Some(last_item) = cur_list.items.last_mut() {
                        let sublists = parse_items_at_level(items, idx, item.ilvl);
                        last_item.sublists.extend(sublists);
                        continue;
                    }
                }
            }

            let it = &items[*idx];
            *idx += 1;

            let is_bullet = it.is_bullet;
            let list_type = it.list_type.clone();
            let text = it.text.clone();

            let item_node = ItemNode {
                text,
                sublists: Vec::new(),
            };

            if let Some(cur_list) = lists.last_mut() {
                if cur_list.is_bullet == is_bullet && cur_list.list_type == list_type {
                    cur_list.items.push(item_node);
                    continue;
                }
            }

            lists.push(ListNode {
                is_bullet,
                list_type,
                items: vec![item_node],
            });
        }

        lists
    }

    let mut idx = 0;
    let min_lvl = items.iter().map(|i| i.ilvl).min().unwrap_or(0);
    parse_items_at_level(items, &mut idx, min_lvl)
}

fn render_list_node_dxml(node: &ListNode, indent: &str, out: &mut String) {
    let tag = if node.is_bullet { "ul" } else { "ol" };
    let default_type = if node.is_bullet { "disc" } else { "1" };
    let type_attr = if !node.list_type.is_empty() && node.list_type != default_type {
        format!(" type=\"{}\"", node.list_type)
    } else {
        String::new()
    };

    out.push_str(&format!("{}<{}{}>\n", indent, tag, type_attr));
    let next_indent = format!("{}  ", indent);
    let sub_indent = format!("{}    ", indent);

    for itm in &node.items {
        if itm.sublists.is_empty() {
            out.push_str(&format!("{}<li>{}</li>\n", next_indent, itm.text));
        } else {
            if itm.text.is_empty() {
                out.push_str(&format!("{}<li>\n", next_indent));
            } else {
                out.push_str(&format!("{}<li>{}\n", next_indent, itm.text));
            }
            for sub in &itm.sublists {
                render_list_node_dxml(sub, &sub_indent, out);
                out.push_str("\n");
            }
            out.push_str(&format!("{}</li>\n", next_indent));
        }
    }
    out.push_str(&format!("{}</{}>", indent, tag));
}

#[derive(Debug)]
enum ImportedBlock {
    Heading(u8, Option<String>, String),
    Paragraph(Option<String>, bool, String, String),
    Callout(String),
    Blockquote(String),
    List(Vec<FlatListItem>),
    Table {
        rows: Vec<Vec<Vec<String>>>,
        is_borderless: bool,
        is_align_right: bool,
    },
    Grid(Vec<Vec<String>>),
    Image(String),
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

fn paragraph_style(
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

fn detect_bullet_item(trimmed: &str) -> Option<(&'static str, String)> {
    if let Some(rest) = trimmed
        .strip_prefix('•')
        .or_else(|| trimmed.strip_prefix('\u{2022}'))
    {
        return Some(("disc", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed
        .strip_prefix('◦')
        .or_else(|| trimmed.strip_prefix('\u{25E6}'))
    {
        return Some(("circle", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed
        .strip_prefix('▪')
        .or_else(|| trimmed.strip_prefix('\u{25AA}'))
        .or_else(|| trimmed.strip_prefix('■'))
    {
        return Some(("square", rest.trim_start().to_string()));
    }
    if let Some(rest) = trimmed
        .strip_prefix('–')
        .or_else(|| trimmed.strip_prefix('—'))
    {
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
    for b in &[
        "•", "\u{2022}", "◦", "\u{25E6}", "▪", "\u{25AA}", "■", "–", "—", "- ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(b) {
            return rest.trim_start().to_string();
        }
    }

    // 2. Formatted bullet prefixes like <b>• </b> or <strong>• </strong>
    for (start_tag, end_tag) in &[
        ("<b>", "</b>"),
        ("<strong>", "</strong>"),
        ("<i>", "</i>"),
        ("<em>", "</em>"),
    ] {
        if let Some(rest) = trimmed.strip_prefix(start_tag) {
            for b in &[
                "•", "\u{2022}", "◦", "\u{25E6}", "▪", "\u{25AA}", "■", "–", "—", "-",
            ] {
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
        && t.chars()
            .next()
            .map(|c| c.is_ascii_alphabetic())
            .unwrap_or(false)
        && (t.ends_with('.') || t.ends_with(')'))
    {
        return true;
    }
    let roman = [
        "i.", "ii.", "iii.", "iv.", "v.", "vi.", "vii.", "viii.", "ix.", "x.", "i)", "ii)", "iii)",
        "iv)", "v)", "I.", "II.", "III.", "IV.", "V.", "VI.", "VII.", "VIII.", "IX.", "X.", "I)",
        "II)", "III)", "IV)", "V)",
    ];
    roman.contains(&t)
}

fn strip_number_prefix(text: &str) -> String {
    let trimmed = text.trim();

    // 1. Formatted number prefixes like <b>1. </b> or <strong>1. </strong>
    for (start_tag, end_tag) in &[
        ("<b>", "</b>"),
        ("<strong>", "</strong>"),
        ("<i>", "</i>"),
        ("<em>", "</em>"),
    ] {
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
            if let Some(content) = rest
                .strip_prefix(". ")
                .or_else(|| rest.strip_prefix(") "))
                .or_else(|| rest.strip_prefix(" - "))
            {
                return content.trim_start().to_string();
            }
        }
    }

    // 3. Alpha prefixes
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_alphabetic()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return trimmed[3..].trim_start().to_string();
    }

    // 4. Roman numeral prefixes
    let roman_prefixes = [
        "i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. ", "I. ",
        "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. ",
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
    if chars.len() >= 3
        && chars[0].is_ascii_lowercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(("a", trimmed[3..].trim().to_string()));
    }
    if chars.len() >= 3
        && chars[0].is_ascii_uppercase()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return Some(("A", trimmed[3..].trim().to_string()));
    }
    let lower_roman_prefixes = [
        "i. ", "ii. ", "iii. ", "iv. ", "v. ", "vi. ", "vii. ", "viii. ", "ix. ", "x. ",
    ];
    for p in &lower_roman_prefixes {
        if trimmed.starts_with(p) {
            return Some(("i", trimmed[p.len()..].trim().to_string()));
        }
    }
    let upper_roman_prefixes = [
        "I. ", "II. ", "III. ", "IV. ", "V. ", "VI. ", "VII. ", "VIII. ", "IX. ", "X. ",
    ];
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

#[derive(Debug, Default, Clone)]
struct ImportedTextStyle {
    font_family: Option<String>,
    font_size_half_points: Option<u32>,
    color: Option<String>,
}

fn parse_document_styles(xml: &str) -> HashMap<String, ImportedTextStyle> {
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
                                        Some(txt) if txt.contains('o') || txt.contains('◦') => {
                                            "circle"
                                        }
                                        Some(txt)
                                            if txt.contains('▪')
                                                || txt.contains('■')
                                                || txt.contains('§') =>
                                        {
                                            "square"
                                        }
                                        Some(txt) if txt.contains('–') || txt.contains('-') => {
                                            "dash"
                                        }
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

fn parse_relationships(xml_str: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut id_map = HashMap::new();
    let mut type_map = HashMap::new();
    let mut reader = Reader::from_str(xml_str);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"Relationship" {
                    let mut id = String::new();
                    let mut target = String::new();
                    let mut rel_type = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Id" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                id = v.to_string();
                            }
                        } else if attr.key.as_ref() == b"Target" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                target = v.to_string();
                            }
                        } else if attr.key.as_ref() == b"Type" {
                            if let Ok(v) = std::str::from_utf8(&attr.value) {
                                rel_type = v.to_string();
                            }
                        }
                    }
                    if !id.is_empty() && !target.is_empty() {
                        id_map.insert(id, target.clone());
                    }
                    if !rel_type.is_empty() && !target.is_empty() {
                        let short_type = rel_type
                            .split('/')
                            .last()
                            .unwrap_or(&rel_type)
                            .to_lowercase();
                        type_map.insert(short_type, target);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    (id_map, type_map)
}

fn extract_pt_from_style(style: &str, key: &str) -> Option<f32> {
    if let Some(pos) = style.find(key) {
        let after = &style[pos + key.len()..];
        let num_str: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        return num_str.parse::<f32>().ok();
    }
    None
}

#[derive(Debug, Default, Clone)]
pub struct ParsedGenericPart {
    pub left_items: Vec<String>,
    pub center_items: Vec<String>,
    pub right_items: Vec<String>,
    pub has_full_bleed: bool,
    pub max_image_height_mm: Option<f32>,
}

impl ParsedGenericPart {
    pub fn is_empty(&self) -> bool {
        self.left_items.is_empty() && self.center_items.is_empty() && self.right_items.is_empty()
    }

    pub fn to_dxml(&self, tag_name: &str) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut out = format!("        <{}>\n", tag_name);
        if !self.left_items.is_empty() {
            out.push_str("          <left>\n");
            for item in &self.left_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </left>\n");
        }
        if !self.center_items.is_empty() {
            out.push_str("          <center>\n");
            for item in &self.center_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </center>\n");
        }
        if !self.right_items.is_empty() {
            out.push_str("          <right>\n");
            for item in &self.right_items {
                out.push_str(&format!("            {}\n", item));
            }
            out.push_str("          </right>\n");
        }
        out.push_str(&format!("        </{}>", tag_name));
        out
    }
}

#[derive(Debug, Clone)]
struct PartImg {
    r_id: String,
    cx: Option<u64>,
    cy: Option<u64>,
    pos_offset: Option<i64>,
    vml_style: String,
}

fn parse_generic_part(
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
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                match name.as_ref() {
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
                            let mut disabled = false;
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if attr.value.as_ref() == b"0"
                                        || attr.value.as_ref() == b"none"
                                        || attr.value.as_ref() == b"nil"
                                    {
                                        disabled = true;
                                    }
                                }
                            }
                            if !disabled {
                                is_underline = true;
                            }
                        }
                    }
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
                    b"wp:posOffset" => {
                        in_pos_offset = true;
                    }
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
                                    if !current_p_images.iter().any(|im| im.r_id == rid) {
                                        current_p_images.push(PartImg {
                                            r_id: rid.to_string(),
                                            cx: current_cx,
                                            cy: current_cy,
                                            pos_offset: current_pos_offset,
                                            vml_style: current_vml_style.clone(),
                                        });
                                    }
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
                                    if !current_p_images.iter().any(|im| im.r_id == rid) {
                                        current_p_images.push(PartImg {
                                            r_id: rid.to_string(),
                                            cx: current_cx,
                                            cy: current_cy,
                                            pos_offset: current_pos_offset,
                                            vml_style: current_vml_style.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
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
            Ok(Event::End(ref e)) => {
                let name = e.name();
                match name.as_ref() {
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
                    b"wp:posOffset" => {
                        in_pos_offset = false;
                    }
                    b"w:r" => {
                        in_r = false;
                    }
                    b"w:p" => {
                        for img in &current_p_images {
                            if let Some(fname) = rels.get(&img.r_id) {
                                let is_bleed = img.pos_offset.map(|o| o < -50000).unwrap_or(false)
                                    || img.cx.map(|cx| cx >= 7000000).unwrap_or(false)
                                    || img.vml_style.contains("margin-left:-")
                                    || img.vml_style.contains("width:612")
                                    || img.vml_style.contains("width:595");
                                if is_bleed {
                                    part.has_full_bleed = true;
                                }

                                let mut height_mm = img.cy.map(|cy| (cy as f32) / 914400.0 * 25.4);
                                if height_mm.is_none() && !img.vml_style.is_empty() {
                                    if let Some(h_pt) =
                                        extract_pt_from_style(&img.vml_style, "height:")
                                    {
                                        height_mm = Some(h_pt * 25.4 / 72.0);
                                    }
                                }
                                if let Some(h) = height_mm {
                                    part.max_image_height_mm = Some(
                                        part.max_image_height_mm.map(|p| p.max(h)).unwrap_or(h),
                                    );
                                }

                                let base_class = if is_footer {
                                    "footer-image"
                                } else {
                                    "header-image"
                                };
                                let class_attr = if is_bleed {
                                    format!("{} full-bleed", base_class)
                                } else {
                                    base_class.to_string()
                                };
                                let img_dxml =
                                    format!(r#"<img class="{}" src="{}" />"#, class_attr, fname);

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
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    part
}

pub fn import_docx(bytes: &[u8]) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;

    // 1. Extract all media files into assets map (clean filenames -> data URLs) and path lookup map
    let mut assets: HashMap<String, String> = HashMap::new();
    let mut media_path_to_filename: HashMap<String, String> = HashMap::new();
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().to_string();
            let is_media = name.contains("media/")
                || name.ends_with(".png")
                || name.ends_with(".jpg")
                || name.ends_with(".jpeg")
                || name.ends_with(".gif")
                || name.ends_with(".bmp")
                || name.ends_with(".webp");
            if is_media {
                let mut img_bytes = Vec::new();
                if file.read_to_end(&mut img_bytes).is_ok() && !img_bytes.is_empty() {
                    let ext = name.split('.').last().unwrap_or("png").to_lowercase();
                    let mime = match ext.as_str() {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "bmp" => "image/bmp",
                        "webp" => "image/webp",
                        _ => "image/png",
                    };
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&img_bytes);
                    let data_url = format!("data:{};base64,{}", mime, b64);
                    let filename = name.split('/').last().unwrap_or(&name).to_string();
                    assets.insert(filename.clone(), data_url);
                    media_path_to_filename.insert(name.clone(), filename.clone());
                    media_path_to_filename.insert(filename.clone(), filename.clone());
                    media_path_to_filename.insert(format!("media/{}", filename), filename.clone());
                    media_path_to_filename
                        .insert(format!("../media/{}", filename), filename.clone());
                    media_path_to_filename
                        .insert(format!("word/media/{}", filename), filename.clone());
                }
            }
        }
    }

    // 2. Parse relationship files to map rId -> clean filename (e.g. "image1.png")
    let (doc_id_rels, doc_type_rels) = {
        let mut xml = String::new();
        if let Ok(mut f) = archive.by_name("word/_rels/document.xml.rels") {
            let _ = f.read_to_string(&mut xml);
        }
        parse_relationships(&xml)
    };

    let doc_img_rels: HashMap<String, String> = doc_id_rels
        .iter()
        .filter_map(|(id, target)| {
            let direct = media_path_to_filename.get(target);
            let fallback = media_path_to_filename.get(target.split('/').last().unwrap_or(target));
            direct.or(fallback).map(|fname| (id.clone(), fname.clone()))
        })
        .collect();

    // 3. Extract Header and Footer generically if present
    let parsed_header = {
        let target = doc_type_rels
            .get("header")
            .cloned()
            .unwrap_or_else(|| "header1.xml".to_string());
        let part_name = if target.starts_with("word/") {
            target.clone()
        } else {
            format!("word/{}", target)
        };
        let rels_name = format!(
            "word/_rels/{}.rels",
            target.split('/').last().unwrap_or(&target)
        );

        let mut h_xml = String::new();
        if let Ok(mut f) = archive.by_name(&part_name) {
            let _ = f.read_to_string(&mut h_xml);
        }

        let mut h_rels_xml = String::new();
        if let Ok(mut f) = archive.by_name(&rels_name) {
            let _ = f.read_to_string(&mut h_rels_xml);
        }
        let (raw_h_rels, _) = parse_relationships(&h_rels_xml);
        let h_rels: HashMap<String, String> = raw_h_rels
            .into_iter()
            .filter_map(|(id, tgt)| {
                let direct = media_path_to_filename.get(&tgt);
                let fallback = media_path_to_filename.get(tgt.split('/').last().unwrap_or(&tgt));
                direct.or(fallback).map(|fname| (id, fname.clone()))
            })
            .collect();

        if !h_xml.is_empty() {
            parse_generic_part(&h_xml, &h_rels, false)
        } else {
            ParsedGenericPart::default()
        }
    };

    let parsed_footer = {
        let target = doc_type_rels
            .get("footer")
            .cloned()
            .unwrap_or_else(|| "footer1.xml".to_string());
        let part_name = if target.starts_with("word/") {
            target.clone()
        } else {
            format!("word/{}", target)
        };
        let rels_name = format!(
            "word/_rels/{}.rels",
            target.split('/').last().unwrap_or(&target)
        );

        let mut f_xml = String::new();
        if let Ok(mut f) = archive.by_name(&part_name) {
            let _ = f.read_to_string(&mut f_xml);
        }

        let mut f_rels_xml = String::new();
        if let Ok(mut f) = archive.by_name(&rels_name) {
            let _ = f.read_to_string(&mut f_rels_xml);
        }
        let (raw_f_rels, _) = parse_relationships(&f_rels_xml);
        let f_rels: HashMap<String, String> = raw_f_rels
            .into_iter()
            .filter_map(|(id, tgt)| {
                let direct = media_path_to_filename.get(&tgt);
                let fallback = media_path_to_filename.get(tgt.split('/').last().unwrap_or(&tgt));
                direct.or(fallback).map(|fname| (id, fname.clone()))
            })
            .collect();

        if !f_xml.is_empty() {
            parse_generic_part(&f_xml, &f_rels, true)
        } else {
            ParsedGenericPart::default()
        }
    };

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
    let document_styles = {
        let mut styles_xml = String::new();
        if let Ok(mut file) = archive.by_name("word/styles.xml") {
            let _ = file.read_to_string(&mut styles_xml);
        }
        parse_document_styles(&styles_xml)
    };

    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(false);

    let mut blocks: Vec<ImportedBlock> = Vec::new();

    let mut current_p_style: Option<String> = None;
    let mut current_p_align: Option<String> = None;
    let mut current_p_is_callout = false;
    let mut current_p_num_id: Option<String> = None;
    let mut current_p_ilvl: u32 = 0;
    let mut current_p_tokens: Vec<InlineToken> = Vec::new();
    let mut font_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut font_size_counts: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    let mut font_color_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

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
    let mut current_font_family: Option<String> = None;
    let mut current_font_size_half_points: Option<u32> = None;
    let mut current_font_color: Option<String> = None;
    let mut current_highlight_color: Option<String> = None;

    // Keep direct paragraph formatting at the paragraph where it belongs. The
    // previous importer only kept an alignment attribute and flattened all
    // other spacing/indentation into one generated stylesheet.
    let mut current_p_before_twips: Option<u32> = None;
    let mut current_p_after_twips: Option<u32> = None;
    let mut current_p_line_twips: Option<u32> = None;
    let mut current_p_left_twips: Option<u32> = None;
    let mut current_p_right_twips: Option<u32> = None;
    let mut current_p_first_line_twips: Option<i32> = None;

    // Paper settings
    let mut paper_size = "A4".to_string();
    let mut paper_orientation = "portrait".to_string();
    let mut paper_margin_mm = 20.0f32;

    // Table state
    let mut in_tbl = false;
    let mut current_tbl_indent: u32 = 0;
    let mut current_tbl_has_borders = false;
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
                        current_tbl_indent = 0;
                        current_tbl_has_borders = false;
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
                        current_p_before_twips = None;
                        current_p_after_twips = None;
                        current_p_line_twips = None;
                        current_p_left_twips = None;
                        current_p_right_twips = None;
                        current_p_first_line_twips = None;
                    }
                    b"w:tblInd" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:w" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_tbl_indent = val.parse::<u32>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    b"w:top" | b"w:left" | b"w:bottom" | b"w:right" | b"w:insideH"
                    | b"w:insideV" => {
                        if in_tbl {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let v_lower = val.to_lowercase();
                                        if v_lower != "none"
                                            && v_lower != "nil"
                                            && v_lower != "0"
                                            && !v_lower.is_empty()
                                        {
                                            current_tbl_has_borders = true;
                                        }
                                    }
                                }
                            }
                        }
                        if in_p_pr && name.as_ref() == b"w:left" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:color" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let col_lower = val.to_lowercase();
                                        if col_lower.starts_with("16a")
                                            || col_lower.starts_with("22c")
                                        {
                                            current_p_is_callout = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"a:blip" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r:embed" {
                                if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                    if let Some(url) = doc_img_rels.get(rid) {
                                        blocks.push(ImportedBlock::Image(url.clone()));
                                    }
                                }
                            }
                        }
                    }
                    b"v:imagedata" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r:id" {
                                if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                    if let Some(url) = doc_img_rels.get(rid) {
                                        blocks.push(ImportedBlock::Image(url.clone()));
                                    }
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
                        current_font_family = None;
                        current_font_size_half_points = None;
                        current_font_color = None;
                        current_highlight_color = None;
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
                    b"w:tblInd" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:w" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_tbl_indent = val.parse::<u32>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    b"w:top" | b"w:left" | b"w:bottom" | b"w:right" | b"w:insideH"
                    | b"w:insideV" => {
                        if in_tbl {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let v_lower = val.to_lowercase();
                                        if v_lower != "none"
                                            && v_lower != "nil"
                                            && v_lower != "0"
                                            && !v_lower.is_empty()
                                        {
                                            current_tbl_has_borders = true;
                                        }
                                    }
                                }
                            }
                        }
                        if in_p_pr && name.as_ref() == b"w:left" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:color" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let col_lower = val.to_lowercase();
                                        if col_lower.starts_with("16a")
                                            || col_lower.starts_with("22c")
                                        {
                                            current_p_is_callout = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"a:blip" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r:embed" {
                                if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                    if let Some(url) = doc_img_rels.get(rid) {
                                        blocks.push(ImportedBlock::Image(url.clone()));
                                    }
                                }
                            }
                        }
                    }
                    b"v:imagedata" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r:id" {
                                if let Ok(rid) = std::str::from_utf8(&attr.value) {
                                    if let Some(url) = doc_img_rels.get(rid) {
                                        blocks.push(ImportedBlock::Image(url.clone()));
                                    }
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
                    b"w:spacing" => {
                        if in_p_pr {
                            for attr in e.attributes().flatten() {
                                let value = std::str::from_utf8(&attr.value)
                                    .ok()
                                    .and_then(|v| v.parse::<u32>().ok());
                                match attr.key.as_ref() {
                                    b"w:before" => current_p_before_twips = value,
                                    b"w:after" => current_p_after_twips = value,
                                    b"w:line" => current_p_line_twips = value,
                                    _ => {}
                                }
                            }
                        }
                    }
                    b"w:ind" => {
                        if in_p_pr {
                            for attr in e.attributes().flatten() {
                                let value = std::str::from_utf8(&attr.value)
                                    .ok()
                                    .and_then(|v| v.parse::<i32>().ok());
                                match attr.key.as_ref() {
                                    b"w:left" | b"w:start" => {
                                        current_p_left_twips =
                                            value.and_then(|v| u32::try_from(v).ok())
                                    }
                                    b"w:right" | b"w:end" => {
                                        current_p_right_twips =
                                            value.and_then(|v| u32::try_from(v).ok())
                                    }
                                    b"w:firstLine" => current_p_first_line_twips = value,
                                    b"w:hanging" => current_p_first_line_twips = value.map(|v| -v),
                                    _ => {}
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
                                    if attr.value.as_ref() == b"none"
                                        || attr.value.as_ref() == b"nil"
                                        || attr.value.as_ref() == b"0"
                                    {
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
                                        if val != "none"
                                            && val != "nil"
                                            && val != "0"
                                            && !val.is_empty()
                                        {
                                            is_highlight = true;
                                            current_highlight_color = Some(val.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:rFonts" => {
                        if in_r {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:ascii"
                                    || attr.key.as_ref() == b"w:hAnsi"
                                {
                                    if let Ok(font_name) = std::str::from_utf8(&attr.value) {
                                        current_font_family = Some(font_name.to_string());
                                        let f_lower = font_name.to_lowercase();
                                        if f_lower.contains("consolas")
                                            || f_lower.contains("courier")
                                            || f_lower.contains("mono")
                                            || f_lower.contains("code")
                                        {
                                            is_code = true;
                                        } else if !font_name.trim().is_empty() {
                                            *font_counts
                                                .entry(font_name.to_string())
                                                .or_insert(0) += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:sz" | b"w:szCs" => {
                        if in_r {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        if let Ok(half_pts) = val.parse::<u32>() {
                                            *font_size_counts.entry(half_pts).or_insert(0) += 1;
                                            current_font_size_half_points = Some(half_pts);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"w:color" => {
                        if in_r {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        let v = val.trim();
                                        if v != "auto" && !v.is_empty() {
                                            current_font_color = Some(v.to_string());
                                        }
                                        if v != "auto" && !v.is_empty() && v != "000000" {
                                            *font_color_counts
                                                .entry(format!("#{}", v))
                                                .or_insert(0) += 1;
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
                                    if in_r
                                        && (fill_lower.starts_with("f1f5")
                                            || fill_lower.starts_with("e2e8"))
                                    {
                                        is_code = true;
                                    }
                                    if in_p_pr
                                        && (fill_lower.starts_with("f0fd")
                                            || fill_lower.starts_with("dcfc"))
                                    {
                                        current_p_is_callout = true;
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
                            font_family: None,
                            font_size_half_points: None,
                            color: None,
                            highlight_color: None,
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
                            font_family: current_font_family.clone(),
                            font_size_half_points: current_font_size_half_points,
                            color: current_font_color.clone(),
                            highlight_color: current_highlight_color.clone(),
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
                            let style_lower =
                                current_p_style.as_deref().unwrap_or("").to_lowercase();
                            let align = current_p_align.clone();
                            let direct_style = paragraph_style(
                                current_p_before_twips,
                                current_p_after_twips,
                                current_p_line_twips,
                                current_p_left_twips,
                                current_p_right_twips,
                                current_p_first_line_twips,
                            );

                            let list_format_opt = if let Some(ref nid) = current_p_num_id {
                                if nid != "0" {
                                    if let Some(abs_id) = numbering_info.num_to_abstract.get(nid) {
                                        numbering_info
                                            .abstract_lvl_format
                                            .get(&(abs_id.clone(), current_p_ilvl))
                                            .cloned()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            let detected_list_item: Option<(bool, String, String)> =
                                if let Some(fmt) = list_format_opt {
                                    match fmt {
                                        ListFormat::Bullet(b_type) => {
                                            Some((true, b_type, strip_bullet_prefix(trimmed)))
                                        }
                                        ListFormat::Number(n_type) => {
                                            Some((false, n_type, strip_number_prefix(trimmed)))
                                        }
                                    }
                                } else if current_p_num_id.is_some()
                                    && current_p_num_id.as_deref() != Some("0")
                                {
                                    if style_lower.contains("bullet") {
                                        Some((
                                            true,
                                            "disc".to_string(),
                                            strip_bullet_prefix(trimmed),
                                        ))
                                    } else if style_lower.contains("number")
                                        || style_lower.contains("decimal")
                                    {
                                        Some((false, "1".to_string(), strip_number_prefix(trimmed)))
                                    } else if let Some((b_type, clean_text)) =
                                        detect_bullet_item(trimmed)
                                    {
                                        Some((
                                            true,
                                            b_type.to_string(),
                                            strip_bullet_prefix(&clean_text),
                                        ))
                                    } else if let Some((n_type, clean_text)) =
                                        detect_number_item(trimmed)
                                    {
                                        Some((
                                            false,
                                            n_type.to_string(),
                                            strip_number_prefix(&clean_text),
                                        ))
                                    } else {
                                        Some((
                                            true,
                                            "disc".to_string(),
                                            strip_bullet_prefix(trimmed),
                                        ))
                                    }
                                } else if style_lower.contains("listbullet") {
                                    let clean_text = strip_bullet_prefix(trimmed);
                                    let (b_type, text_to_use) = detect_bullet_item(&clean_text)
                                        .unwrap_or(("disc", clean_text));
                                    Some((true, b_type.to_string(), text_to_use))
                                } else if style_lower.contains("listnumber")
                                    || style_lower.contains("numbered")
                                {
                                    let clean_text = strip_number_prefix(trimmed);
                                    let (n_type, text_to_use) = detect_number_item(&clean_text)
                                        .unwrap_or(("1", clean_text));
                                    Some((false, n_type.to_string(), text_to_use))
                                } else if let Some((b_type, clean_text)) =
                                    detect_bullet_item(trimmed)
                                {
                                    Some((
                                        true,
                                        b_type.to_string(),
                                        strip_bullet_prefix(&clean_text),
                                    ))
                                } else if let Some((n_type, clean_text)) =
                                    detect_number_item(trimmed)
                                {
                                    Some((
                                        false,
                                        n_type.to_string(),
                                        strip_number_prefix(&clean_text),
                                    ))
                                } else {
                                    None
                                };

                            if in_tbl {
                                let align_attr = align
                                    .as_ref()
                                    .map(|a| format!(" align=\"{}\"", a))
                                    .unwrap_or_default();
                                let style_attr = if direct_style.is_empty() {
                                    String::new()
                                } else {
                                    format!(" style=\"{}\"", direct_style)
                                };
                                let p_tag = format!(
                                    "<p{}{}>{}</p>",
                                    align_attr, style_attr, formatted_line
                                );
                                current_cell_paragraphs.push(p_tag);
                            } else if current_p_is_callout || style_lower.contains("callout") {
                                blocks.push(ImportedBlock::Callout(formatted_line));
                            } else if style_lower.contains("heading 1")
                                || style_lower == "1"
                                || style_lower == "heading1"
                            {
                                blocks.push(ImportedBlock::Heading(1, align, formatted_line));
                            } else if style_lower.contains("heading 2")
                                || style_lower == "2"
                                || style_lower == "heading2"
                            {
                                blocks.push(ImportedBlock::Heading(2, align, formatted_line));
                            } else if style_lower.contains("heading 3")
                                || style_lower == "3"
                                || style_lower == "heading3"
                            {
                                blocks.push(ImportedBlock::Heading(3, align, formatted_line));
                            } else if let Some((is_bullet, list_type, clean_text)) =
                                detected_list_item
                            {
                                let item = FlatListItem {
                                    ilvl: current_p_ilvl,
                                    is_bullet,
                                    list_type,
                                    text: clean_text,
                                };
                                if let Some(ImportedBlock::List(items)) = blocks.last_mut() {
                                    items.push(item);
                                } else {
                                    blocks.push(ImportedBlock::List(vec![item]));
                                }
                            } else if style_lower.contains("quote") {
                                blocks.push(ImportedBlock::Blockquote(formatted_line));
                            } else {
                                // Intelligent heading detection heuristic:
                                // Short title with all bold tokens and no terminating period
                                let all_bold = !current_p_tokens.is_empty()
                                    && current_p_tokens.iter().all(|t| t.bold);
                                let is_short = trimmed.len() < 45;
                                let is_title = all_bold && is_short && !trimmed.ends_with('.');

                                if is_title {
                                    blocks.push(ImportedBlock::Heading(2, align, formatted_line));
                                } else {
                                    let is_lead = style_lower.contains("lead");
                                    blocks.push(ImportedBlock::Paragraph(
                                        align,
                                        is_lead,
                                        formatted_line,
                                        direct_style,
                                    ));
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
                        let is_borderless = !current_tbl_has_borders;
                        let is_align_right = current_tbl_indent >= 4000;

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

                        if is_single_row
                            && !is_borderless
                            && non_empty_cells.len() >= 2
                            && non_empty_cells.len() <= 3
                        {
                            blocks.push(ImportedBlock::Grid(non_empty_cells));
                        } else if !current_table_rows.is_empty() {
                            blocks.push(ImportedBlock::Table {
                                rows: current_table_rows.clone(),
                                is_borderless,
                                is_align_right,
                            });
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
                let align_attr = align
                    .map(|a| format!(" align=\"{}\"", a))
                    .unwrap_or_default();
                body_nodes.push(format!(
                    "      <h{}{}>{}</h{}>",
                    level, align_attr, text, level
                ));
            }
            ImportedBlock::Paragraph(align, is_lead, text, direct_style) => {
                let mut attrs = String::new();
                if is_lead {
                    attrs.push_str(" class=\"lead\"");
                }
                if let Some(a) = align {
                    attrs.push_str(&format!(" align=\"{}\"", a));
                }
                if !direct_style.is_empty() {
                    attrs.push_str(&format!(" style=\"{}\"", direct_style));
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
            ImportedBlock::List(items) => {
                let list_trees = build_list_tree(&items);
                for tree in list_trees {
                    let mut dxml_out = String::new();
                    render_list_node_dxml(&tree, "      ", &mut dxml_out);
                    body_nodes.push(dxml_out);
                }
            }
            ImportedBlock::Grid(cols) => {
                let cols_spec = vec!["1fr"; cols.len()].join(" ");
                let mut grid_xml = format!("      <grid cols=\"{}\" gap=\"6mm\">\n", cols_spec);
                for col_ps in cols {
                    grid_xml.push_str("        <div class=\"col-box\">\n");
                    for p in col_ps {
                        grid_xml.push_str(&format!("          {}\n", p));
                    }
                    grid_xml.push_str("        </div>\n");
                }
                grid_xml.push_str("      </grid>");
                body_nodes.push(grid_xml);
            }
            ImportedBlock::Image(url) => {
                body_nodes.push(format!(
                    "      <div align=\"center\" style=\"margin: 4mm 0;\"><img src=\"{}\" style=\"max-width: 100%; height: auto;\" /></div>",
                    url
                ));
            }
            ImportedBlock::Table {
                rows,
                is_borderless,
                is_align_right,
            } => {
                let mut classes = Vec::new();
                if is_borderless {
                    classes.push("borderless");
                }
                if is_align_right {
                    classes.push("align-right");
                }
                let class_attr = if !classes.is_empty() {
                    format!(" class=\"{}\"", classes.join(" "))
                } else {
                    String::new()
                };
                let style_attr = if is_align_right {
                    " style=\"margin-left: auto; width: 45%;\""
                } else {
                    ""
                };
                let mut tbl_xml = format!("      <table{}{}>\n", class_attr, style_attr);
                for (row_idx, row) in rows.iter().enumerate() {
                    tbl_xml.push_str("        <tr>\n");
                    for cell_ps in row {
                        let tag = if row_idx == 0 && !is_borderless {
                            "th"
                        } else {
                            "td"
                        };
                        let td_style = if is_borderless {
                            " style=\"border: none;\""
                        } else {
                            ""
                        };
                        let content = if cell_ps.is_empty() {
                            String::new()
                        } else {
                            cell_ps.join("\n          ")
                        };
                        tbl_xml.push_str(&format!(
                            "          <{}{}>\n            {}\n          </{}>\n",
                            tag,
                            td_style,
                            content.trim(),
                            tag
                        ));
                    }
                    tbl_xml.push_str("        </tr>\n");
                }
                tbl_xml.push_str("      </table>");
                body_nodes.push(tbl_xml);
            }
        }
    }

    let header_xml = parsed_header.to_dxml("running-header");
    let footer_xml = parsed_footer.to_dxml("running-footer");

    let mut page_style_inner = String::new();
    if !header_xml.is_empty() {
        page_style_inner.push_str(&header_xml);
        page_style_inner.push('\n');
    }
    if !footer_xml.is_empty() {
        page_style_inner.push_str(&footer_xml);
        page_style_inner.push('\n');
    }

    let page_styles_block = if !page_style_inner.is_empty() {
        format!(
            r#"    <page-styles>
      <page-style name="body" default="true">
{}
      </page-style>
    </page-styles>"#,
            page_style_inner.trim_end()
        )
    } else {
        r#"    <page-styles>
      <page-style name="body" default="true" />
    </page-styles>"#
            .to_string()
    };

    let dxml = format!(
        r#"<doc>
  <props>
    <meta>
      <title>Imported OpenXML Document</title>
      <author>Pagina Reverse-Engineering Engine</author>
    </meta>
    <paper size="{}" orientation="{}" margin="{}mm" />
    <link rel="stylesheet" href="styles.dcss" />
{}
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
        page_styles_block,
        if body_nodes.is_empty() {
            "      <p>Empty document</p>".to_string()
        } else {
            body_nodes.join("\n")
        }
    );

    let normal_style = document_styles
        .get("Normal")
        .or_else(|| document_styles.get("normal"));
    let body_half_pts = normal_style
        .and_then(|style| style.font_size_half_points)
        .or_else(|| {
            font_size_counts
                .iter()
                .max_by_key(|&(_, count)| count)
                .map(|(&sz, _)| sz)
        })
        .unwrap_or(28);

    let body_pt = (body_half_pts as f32) / 2.0;

    let mut sorted_larger_sizes: Vec<u32> = font_size_counts
        .keys()
        .copied()
        .filter(|&sz| sz > body_half_pts)
        .collect();
    sorted_larger_sizes.sort_unstable();

    let style_size = |name: &str| {
        document_styles
            .get(name)
            .and_then(|style| style.font_size_half_points)
            .map(|size| size as f32 / 2.0)
    };
    let h1_pt = style_size("Heading1")
        .or_else(|| sorted_larger_sizes.pop().map(|sz| (sz as f32) / 2.0))
        .unwrap_or(body_pt + 4.0);
    let h2_pt = style_size("Heading2")
        .or_else(|| sorted_larger_sizes.pop().map(|sz| (sz as f32) / 2.0))
        .unwrap_or((body_pt + 2.0).min(h1_pt));
    let h3_pt = style_size("Heading3").unwrap_or(body_pt);

    let primary_font = normal_style
        .and_then(|style| style.font_family.clone())
        .or_else(|| {
            font_counts
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(name, _)| name)
        })
        .unwrap_or_else(|| "Times New Roman".to_string());

    let font_family_stack = {
        let lower = primary_font.to_lowercase();
        if lower.contains("times")
            || lower.contains("serif")
            || lower.contains("georgia")
            || lower.contains("garamond")
            || lower.contains("cambria")
        {
            format!("\"{}\", Times, serif", primary_font)
        } else if lower.contains("consolas") || lower.contains("courier") || lower.contains("mono")
        {
            format!("\"{}\", Consolas, monospace", primary_font)
        } else {
            format!(
                "\"{}\", -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif",
                primary_font
            )
        }
    };

    let primary_color = normal_style
        .and_then(|style| style.color.as_ref().map(|color| format!("#{}", color)))
        .or_else(|| {
            font_color_counts
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(c, _)| c)
        })
        .unwrap_or_else(|| "#000000".to_string());
    let style_color = |name: &str| {
        document_styles
            .get(name)
            .and_then(|style| style.color.as_ref().map(|color| format!("#{}", color)))
    };
    let h1_color = style_color("Heading1").unwrap_or_else(|| primary_color.clone());
    let h2_color = style_color("Heading2").unwrap_or_else(|| primary_color.clone());
    let h3_color = style_color("Heading3").unwrap_or_else(|| primary_color.clone());

    let header_max_h = parsed_header
        .max_image_height_mm
        .map(|h| (h + 1.0).round().max(25.0) as u32)
        .unwrap_or(35);

    let dcss = format!(
        r#"/* Generated by Pagina OpenXML Reverse Engine */
body {{
  font-family: {font_family_stack};
  font-size: {body_pt}pt;
  color: {primary_color};
  line-height: 1.35;
}}

h1 {{
  font-family: {font_family_stack};
  font-size: {h1_pt}pt;
  font-weight: bold;
  color: {h1_color};
  margin-top: 4mm;
  margin-bottom: 2.5mm;
}}

h2 {{
  font-family: {font_family_stack};
  font-size: {h2_pt}pt;
  font-weight: bold;
  color: {h2_color};
  margin-top: 3.5mm;
  margin-bottom: 2mm;
}}

h3 {{
  font-family: {font_family_stack};
  font-size: {h3_pt}pt;
  font-weight: bold;
  color: {h3_color};
  margin-top: 3mm;
  margin-bottom: 1.5mm;
}}

p {{
  font-family: {font_family_stack};
  font-size: {body_pt}pt;
  line-height: 1.35;
  color: {primary_color};
  margin-bottom: 2.5mm;
}}

p.lead {{
  font-size: {lead_pt}pt;
  color: {primary_color};
}}

blockquote {{
  margin-left: 6mm;
  padding-left: 4mm;
  border-left: 3px solid #cbd5e1;
  color: {primary_color};
  margin-bottom: 4mm;
}}

ul, ol {{
  padding-left: 24px;
  margin-top: 2mm;
  margin-bottom: 2.5mm;
}}

li {{
  font-family: {font_family_stack};
  font-size: {body_pt}pt;
  margin-bottom: 1.5mm;
  line-height: 1.35;
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
  font-family: {font_family_stack};
  font-size: {body_pt}pt;
  width: 100%;
  border-collapse: collapse;
  margin: 3mm 0;
}}

table.borderless {{
  border: none !important;
}}

table.align-right {{
  margin-left: auto !important;
  margin-right: 0 !important;
  width: 48% !important;
}}

th, td {{
  font-family: {font_family_stack};
  font-size: {body_pt}pt;
  padding: 1.5mm 3mm;
  line-height: 1.3;
}}

th {{
  border: 1px solid #cbd5e1;
  background-color: #f8fafc;
  font-weight: bold;
}}

td {{
  border: 1px solid #cbd5e1;
}}

table.borderless td,
table.borderless th {{
  border: none !important;
  background: transparent !important;
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

/* Header & Footer Layout & Positioning */
header, running-header, .running-header {{
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 3mm;
}}

.running-header .header-left, header .header-left {{
  text-align: left;
}}

.running-header .header-center, header .header-center {{
  text-align: center;
}}

.running-header .header-right, header .header-right {{
  text-align: right;
}}

.header-image, running-header img, header img {{
  max-height: {header_max_h}mm;
  width: auto;
  max-width: 100%;
  display: block;
  margin: 0 auto;
}}

footer, running-footer, .running-footer {{
  width: 100%;
  margin-top: auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
}}

.running-footer .footer-left, footer .footer-left {{
  text-align: left;
}}

.running-footer .footer-center, footer .footer-center {{
  text-align: center;
}}

.running-footer .footer-right, footer .footer-right {{
  text-align: right;
}}

.footer-image, running-footer img, footer img {{
  max-width: 100%;
  height: auto;
  display: block;
  margin: 0 auto;
}}

.footer-image.full-bleed,
.header-image.full-bleed,
running-footer.full-bleed,
footer.full-bleed,
img.full-bleed,
.full-bleed {{
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100% !important;
  height: auto;
  max-height: 35mm;
  display: block;
  margin: 0;
  padding: 0;
  z-index: 10;
}}
"#,
        font_family_stack = font_family_stack,
        body_pt = body_pt,
        h1_pt = h1_pt,
        h2_pt = h2_pt,
        h3_pt = h3_pt,
        lead_pt = body_pt + 1.0,
        primary_color = primary_color,
        h1_color = h1_color,
        h2_color = h2_color,
        h3_color = h3_color,
        header_max_h = header_max_h
    );

    Ok(ImportResult { dxml, dcss, assets })
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
