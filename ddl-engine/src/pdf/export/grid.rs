use super::context::PdfRenderContext;
use super::drawing::draw_rounded_rect;
use super::nodes::render_doc_nodes;
use super::types::extract_plain_text;
use crate::ast::{ElementNode, Node};
use printpdf::path::PaintMode;
use printpdf::{Color, Rgb};

pub fn estimate_nodes_height(nodes: &[Node], width: f32) -> f32 {
    let mut h = 0.0f32;
    for node in nodes {
        match node {
            Node::Element(el) => {
                let tag = el.tag.to_lowercase();
                match tag.as_str() {
                    "h1" => h += 16.0,
                    "h2" => h += 11.0,
                    "h3" => h += 8.5,
                    "p" => {
                        let text = extract_plain_text(el);
                        let chars = text.chars().count();
                        let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                        h += est_lines * 5.2 + 4.0;
                    }
                    _ => h += estimate_nodes_height(&el.children, width),
                }
            }
            Node::Text(t) => {
                let chars = t.content.trim().chars().count();
                if chars > 0 {
                    let est_lines = ((chars as f32 * 2.0) / width).ceil().max(1.0);
                    h += est_lines * 5.2;
                }
            }
            _ => {}
        }
    }
    h
}

pub fn render_grid(el: &ElementNode, ctx: &mut PdfRenderContext) {
    let col_divs: Vec<&ElementNode> = el
        .children
        .iter()
        .filter_map(|c| {
            if let Node::Element(child_el) = c {
                Some(child_el)
            } else {
                None
            }
        })
        .collect();

    if col_divs.is_empty() {
        return;
    }

    let n = col_divs.len();
    let gap_mm = 6.0f32;
    let total_gap = (n - 1) as f32 * gap_mm;
    let col_width = (ctx.content_width - total_gap) / (n as f32);

    let start_y = *ctx.cursor_y;
    let inner_padding = 4.0f32;

    let max_col_height = col_divs
        .iter()
        .map(|c| estimate_nodes_height(&c.children, col_width - (inner_padding * 2.0)))
        .fold(0.0f32, f32::max)
        .max(24.0)
        + (inner_padding * 2.0);

    let uniform_bottom = start_y - max_col_height;
    if uniform_bottom < ctx.bottom_margin {
        ctx.check_page_overflow(max_col_height);
    }

    let actual_start_y = *ctx.cursor_y;
    let actual_uniform_bottom = actual_start_y - max_col_height;
    let card_height = actual_start_y - actual_uniform_bottom;

    for (i, col_el) in col_divs.iter().enumerate() {
        let is_boxed = col_el.classes.iter().any(|c| c == "col-box");
        if is_boxed {
            let col_left = ctx.left_margin + (i as f32) * (col_width + gap_mm);
            ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                248.0 / 255.0,
                250.0 / 255.0,
                252.0 / 255.0,
                None,
            )));
            ctx.current_layer.set_outline_color(Color::Rgb(Rgb::new(
                226.0 / 255.0,
                232.0 / 255.0,
                240.0 / 255.0,
                None,
            )));
            ctx.current_layer.set_outline_thickness(0.6);

            draw_rounded_rect(
                ctx.current_layer,
                col_left,
                actual_uniform_bottom,
                col_width,
                card_height,
                1.8,
                PaintMode::FillStroke,
            );
        }
    }

    let orig_left = ctx.left_margin;
    let orig_width = ctx.content_width;

    for (i, col_el) in col_divs.iter().enumerate() {
        let is_boxed = col_el.classes.iter().any(|c| c == "col-box");
        let col_left = orig_left + (i as f32) * (col_width + gap_mm);
        let pad = if is_boxed { inner_padding } else { 0.0 };

        *ctx.cursor_y = actual_start_y - pad - 1.5;
        ctx.left_margin = col_left + pad;
        ctx.content_width = col_width - (pad * 2.0);

        render_doc_nodes(&col_el.children, ctx, 0);
    }

    ctx.left_margin = orig_left;
    ctx.content_width = orig_width;
    *ctx.cursor_y = actual_uniform_bottom - 6.0;
}
