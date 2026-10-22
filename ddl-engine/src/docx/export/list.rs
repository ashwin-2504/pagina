use super::images::ImageEntry;
use super::styles::emit_inline_runs;
use super::utils::{to_lower_alpha, to_roman, to_upper_alpha};
use crate::ast::{ElementNode, Node};
use std::collections::HashMap;

pub fn emit_unordered_list<F>(
    el: &ElementNode,
    indent_level: usize,
    images: &HashMap<String, ImageEntry>,
    out: &mut String,
    mut convert_nested: F,
) where
    F: FnMut(&Node, usize, &HashMap<String, ImageEntry>, &mut String),
{
    let list_type = el
        .attributes
        .get("type")
        .map(|s| s.as_str())
        .or_else(|| el.inline_styles.get("list-style-type").map(|s| s.as_str()));
    let default_bullet = match indent_level % 3 {
        0 => "•",
        1 => "◦",
        _ => "▪",
    };
    let bullet_sym = match list_type {
        Some("circle") => "◦",
        Some("square") => "▪",
        Some("dash") => "–",
        Some("disc") => "•",
        _ => default_bullet,
    };
    let indent_dxa = (indent_level as u32) * 480 + 720;
    for li in &el.children {
        if let Node::Element(li_el) = li {
            if li_el.tag.to_lowercase() == "li" {
                out.push_str(&format!(
                    "<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/><w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"1\"/></w:numPr><w:ind w:left=\"{}\" w:hanging=\"360\"/><w:spacing w:after=\"60\" w:line=\"260\" w:lineRule=\"auto\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">{} </w:t></w:r>",
                    indent_level, indent_dxa, bullet_sym
                ));
                emit_inline_runs(
                    li_el, false, false, false, false, false, false, false, out,
                );
                out.push_str("</w:p>\n");

                for li_child in &li_el.children {
                    if let Node::Element(nested_list) = li_child {
                        let tag_lower = nested_list.tag.to_lowercase();
                        if tag_lower == "ul" || tag_lower == "ol" {
                            convert_nested(
                                &Node::Element(nested_list.clone()),
                                indent_level + 1,
                                images,
                                out,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn emit_ordered_list<F>(
    el: &ElementNode,
    indent_level: usize,
    images: &HashMap<String, ImageEntry>,
    out: &mut String,
    mut convert_nested: F,
) where
    F: FnMut(&Node, usize, &HashMap<String, ImageEntry>, &mut String),
{
    let num_type = el
        .attributes
        .get("type")
        .map(|s| s.as_str())
        .or_else(|| el.inline_styles.get("list-style-type").map(|s| s.as_str()));
    let default_type = match indent_level % 3 {
        0 => "1",
        1 => "a",
        _ => "i",
    };
    let effective_type = num_type.unwrap_or(default_type);
    let start_idx = el
        .attributes
        .get("start")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let indent_dxa = (indent_level as u32) * 480 + 720;
    let mut item_idx = 0;
    for li in &el.children {
        if let Node::Element(li_el) = li {
            if li_el.tag.to_lowercase() == "li" {
                let num_str = match effective_type {
                    "a" | "lower-alpha" => {
                        format!("{}.", to_lower_alpha(start_idx + item_idx))
                    }
                    "A" | "upper-alpha" => {
                        format!("{}.", to_upper_alpha(start_idx + item_idx))
                    }
                    "i" | "lower-roman" => {
                        format!("{}.", to_roman(start_idx + item_idx))
                    }
                    "I" | "upper-roman" => format!(
                        "{}.",
                        to_roman(start_idx + item_idx).to_uppercase()
                    ),
                    _ => format!("{}.", start_idx + item_idx),
                };
                item_idx += 1;
                out.push_str(&format!(
                    "<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/><w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"2\"/></w:numPr><w:ind w:left=\"{}\" w:hanging=\"360\"/><w:spacing w:after=\"60\" w:line=\"260\" w:lineRule=\"auto\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">{} </w:t></w:r>",
                    indent_level, indent_dxa, num_str
                ));
                emit_inline_runs(
                    li_el, false, false, false, false, false, false, false, out,
                );
                out.push_str("</w:p>\n");

                for li_child in &li_el.children {
                    if let Node::Element(nested_list) = li_child {
                        let tag_lower = nested_list.tag.to_lowercase();
                        if tag_lower == "ul" || tag_lower == "ol" {
                            convert_nested(
                                &Node::Element(nested_list.clone()),
                                indent_level + 1,
                                images,
                                out,
                            );
                        }
                    }
                }
            }
        }
    }
}
