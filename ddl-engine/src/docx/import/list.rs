use super::types::{FlatListItem, ItemNode, ListNode};

pub fn build_list_tree(items: &[FlatListItem]) -> Vec<ListNode> {
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

pub fn render_list_node_dxml(node: &ListNode, indent: &str, out: &mut String) {
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

pub fn detect_bullet_item(trimmed: &str) -> Option<(&'static str, String)> {
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

pub fn strip_bullet_prefix(text: &str) -> String {
    let trimmed = text.trim();

    for b in &[
        "•", "\u{2022}", "◦", "\u{25E6}", "▪", "\u{25AA}", "■", "–", "—", "- ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(b) {
            return rest.trim_start().to_string();
        }
    }

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

pub fn is_number_prefix_token(s: &str) -> bool {
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

pub fn strip_number_prefix(text: &str) -> String {
    let trimmed = text.trim();

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

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3
        && chars[0].is_ascii_alphabetic()
        && (chars[1] == '.' || chars[1] == ')')
        && chars[2].is_whitespace()
    {
        return trimmed[3..].trim_start().to_string();
    }

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

pub fn detect_number_item(trimmed: &str) -> Option<(&'static str, String)> {
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
