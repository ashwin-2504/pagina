use super::list::{
    detect_bullet_item, detect_number_item, strip_bullet_prefix, strip_number_prefix,
};
use super::types::{FlatListItem, ImportedBlock, InlineToken, ListFormat, NumberingInfo};

pub fn detect_list_item(
    num_id: Option<&str>,
    ilvl: u32,
    numbering_info: &NumberingInfo,
    style_lower: &str,
    trimmed: &str,
) -> Option<(bool, String, String)> {
    let list_format_opt = if let Some(nid) = num_id {
        if nid != "0" {
            if let Some(abs_id) = numbering_info.num_to_abstract.get(nid) {
                numbering_info
                    .abstract_lvl_format
                    .get(&(abs_id.clone(), ilvl))
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

    if let Some(fmt) = list_format_opt {
        match fmt {
            ListFormat::Bullet(b_type) => Some((true, b_type, strip_bullet_prefix(trimmed))),
            ListFormat::Number(n_type) => Some((false, n_type, strip_number_prefix(trimmed))),
        }
    } else if num_id.is_some() && num_id != Some("0") {
        if style_lower.contains("bullet") {
            Some((true, "disc".to_string(), strip_bullet_prefix(trimmed)))
        } else if style_lower.contains("number") || style_lower.contains("decimal") {
            Some((false, "1".to_string(), strip_number_prefix(trimmed)))
        } else if let Some((b_type, clean_text)) = detect_bullet_item(trimmed) {
            Some((true, b_type.to_string(), strip_bullet_prefix(&clean_text)))
        } else if let Some((n_type, clean_text)) = detect_number_item(trimmed) {
            Some((false, n_type.to_string(), strip_number_prefix(&clean_text)))
        } else {
            Some((true, "disc".to_string(), strip_bullet_prefix(trimmed)))
        }
    } else if style_lower.contains("listbullet") {
        let clean_text = strip_bullet_prefix(trimmed);
        let (b_type, text_to_use) =
            detect_bullet_item(&clean_text).unwrap_or(("disc", clean_text));
        Some((true, b_type.to_string(), text_to_use))
    } else if style_lower.contains("listnumber") || style_lower.contains("numbered") {
        let clean_text = strip_number_prefix(trimmed);
        let (n_type, text_to_use) =
            detect_number_item(&clean_text).unwrap_or(("1", clean_text));
        Some((false, n_type.to_string(), text_to_use))
    } else if let Some((b_type, clean_text)) = detect_bullet_item(trimmed) {
        Some((true, b_type.to_string(), strip_bullet_prefix(&clean_text)))
    } else if let Some((n_type, clean_text)) = detect_number_item(trimmed) {
        Some((false, n_type.to_string(), strip_number_prefix(&clean_text)))
    } else {
        None
    }
}

pub fn resolve_paragraph_block(
    style: Option<&str>,
    align: Option<String>,
    direct_style: String,
    num_id: Option<&str>,
    ilvl: u32,
    is_callout: bool,
    tokens: &[InlineToken],
    numbering_info: &NumberingInfo,
    formatted_line: &str,
    trimmed: &str,
) -> ImportedBlock {
    let style_lower = style.unwrap_or("").to_lowercase();

    if is_callout || style_lower.contains("callout") {
        return ImportedBlock::Callout(formatted_line.to_string());
    }

    if style_lower.contains("heading 1") || style_lower == "1" || style_lower == "heading1" {
        return ImportedBlock::Heading(1, align, formatted_line.to_string());
    }
    if style_lower.contains("heading 2") || style_lower == "2" || style_lower == "heading2" {
        return ImportedBlock::Heading(2, align, formatted_line.to_string());
    }
    if style_lower.contains("heading 3") || style_lower == "3" || style_lower == "heading3" {
        return ImportedBlock::Heading(3, align, formatted_line.to_string());
    }

    if let Some((is_bullet, list_type, clean_text)) =
        detect_list_item(num_id, ilvl, numbering_info, &style_lower, trimmed)
    {
        return ImportedBlock::List(vec![FlatListItem {
            ilvl,
            is_bullet,
            list_type,
            text: clean_text,
        }]);
    }

    if style_lower.contains("quote") {
        return ImportedBlock::Blockquote(formatted_line.to_string());
    }

    let all_bold = !tokens.is_empty() && tokens.iter().all(|t| t.bold);
    let is_short = trimmed.len() < 45;
    let is_title = all_bold && is_short && !trimmed.ends_with('.');

    if is_title {
        ImportedBlock::Heading(2, align, formatted_line.to_string())
    } else {
        let is_lead = style_lower.contains("lead");
        ImportedBlock::Paragraph(
            align,
            is_lead,
            formatted_line.to_string(),
            direct_style,
        )
    }
}
