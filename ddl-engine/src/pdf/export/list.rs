use super::context::PdfRenderContext;
use super::fonts::FontFamily;
use super::layout::collect_styled_runs;
use super::nodes::render_doc_nodes;
use super::text::render_styled_paragraph;
use super::types::{to_lower_alpha, to_roman, to_upper_alpha, StyledRun};
use crate::ast::{ElementNode, Node};

pub fn format_list_bullet(
    tag: &str,
    list_type: Option<&str>,
    idx: usize,
    indent_level: usize,
) -> String {
    if tag == "ul" {
        let default_bullet = match indent_level % 3 {
            0 => "disc",
            1 => "circle",
            _ => "square",
        };
        match list_type.unwrap_or(default_bullet) {
            "circle" => "◦ ".to_string(),
            "square" => "▪ ".to_string(),
            "dash" => "- ".to_string(),
            _ => "• ".to_string(),
        }
    } else {
        let default_num = match indent_level % 3 {
            0 => "1",
            1 => "a",
            _ => "i",
        };
        match list_type.unwrap_or(default_num) {
            "a" | "lower-alpha" => format!("{}. ", to_lower_alpha(idx)),
            "A" | "upper-alpha" => format!("{}. ", to_upper_alpha(idx)),
            "i" | "lower-roman" => format!("{}. ", to_roman(idx)),
            "I" | "upper-roman" => format!("{}. ", to_roman(idx).to_uppercase()),
            _ => format!("{}. ", idx),
        }
    }
}

pub fn render_list_element(
    el: &ElementNode,
    tag: &str,
    el_font_family: FontFamily,
    indent_level: usize,
    ctx: &mut PdfRenderContext,
) {
    let list_type_attr = el.attributes.get("type").map(|s| s.as_str());
    let list_style = el
        .inline_styles
        .get("list-style-type")
        .map(|s| s.as_str())
        .or(list_type_attr);
    let start_idx = el
        .attributes
        .get("start")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    *ctx.cursor_y -= 1.5;
    let mut item_idx = 0;
    for child in &el.children {
        if let Node::Element(li) = child {
            if li.tag.to_lowercase() == "li" {
                let bullet = format_list_bullet(
                    tag,
                    list_style,
                    start_idx + item_idx,
                    indent_level,
                );
                item_idx += 1;
                let mut runs = Vec::new();
                runs.push(StyledRun {
                    text: bullet,
                    family: FontFamily::SansSerif,
                    bold: true,
                    italic: false,
                });
                let mut nested_lists: Vec<Node> = Vec::new();
                for li_child in &li.children {
                    if let Node::Element(child_el) = li_child {
                        let child_tag = child_el.tag.to_lowercase();
                        if child_tag == "ul" || child_tag == "ol" {
                            nested_lists.push(li_child.clone());
                            continue;
                        }
                    }
                    collect_styled_runs(
                        li_child,
                        el_font_family,
                        false,
                        false,
                        &mut runs,
                    );
                }
                let indent_offset = (indent_level as f32) * 6.0 + 5.0;
                let orig_left = ctx.left_margin;
                let orig_width = ctx.content_width;
                ctx.left_margin += indent_offset;
                ctx.content_width -= indent_offset;

                render_styled_paragraph(&runs, 10.5, None, None, false, ctx);

                ctx.left_margin = orig_left;
                ctx.content_width = orig_width;

                *ctx.cursor_y -= 1.5;

                if !nested_lists.is_empty() {
                    render_doc_nodes(&nested_lists, ctx, indent_level + 1);
                }
            }
        }
    }
    *ctx.cursor_y -= 2.0;
}
