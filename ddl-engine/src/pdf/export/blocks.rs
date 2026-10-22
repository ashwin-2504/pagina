use super::context::PdfRenderContext;
use super::fonts::FontFamily;
use super::layout::collect_styled_runs;
use super::text::{render_heading, render_styled_paragraph};
use super::types::extract_plain_text;
use crate::ast::ElementNode;
use printpdf::{Color, Line, Mm, Point, Rgb};

pub fn render_element_block(
    el: &ElementNode,
    tag: &str,
    align: Option<&'static str>,
    el_font_family: FontFamily,
    ctx: &mut PdfRenderContext,
) -> bool {
    let is_callout = tag == "callout" || el.classes.iter().any(|c| c == "callout");
    if is_callout {
        let mut runs = Vec::new();
        for child in &el.children {
            collect_styled_runs(child, el_font_family, false, false, &mut runs);
        }
        *ctx.cursor_y -= 4.0;
        render_styled_paragraph(&runs, 10.0, None, None, true, ctx);
        *ctx.cursor_y -= 5.0;
        return true;
    }

    match tag {
        "h1" => {
            let text = extract_plain_text(el);
            *ctx.cursor_y -= 6.0;
            render_heading(
                &text,
                24.0,
                align,
                Some((30.0 / 255.0, 58.0 / 255.0, 138.0 / 255.0)),
                ctx.fonts.select(el_font_family, true, false),
                ctx,
            );
            *ctx.cursor_y -= 4.0;
            true
        }
        "h2" => {
            let text = extract_plain_text(el);
            *ctx.cursor_y -= 6.0;
            render_heading(
                &text,
                15.0,
                align,
                Some((15.0 / 255.0, 23.0 / 255.0, 42.0 / 255.0)),
                ctx.fonts.select(el_font_family, true, false),
                ctx,
            );
            *ctx.cursor_y -= 3.0;
            true
        }
        "h3" => {
            let text = extract_plain_text(el);
            *ctx.cursor_y -= 4.0;
            render_heading(
                &text,
                11.5,
                align,
                Some((30.0 / 255.0, 41.0 / 255.0, 59.0 / 255.0)),
                ctx.fonts.select(el_font_family, true, false),
                ctx,
            );
            *ctx.cursor_y -= 2.0;
            true
        }
        "p" => {
            let is_lead = el.classes.iter().any(|c| c == "lead");
            let p_font_size = if is_lead { 12.0 } else { 10.5 };
            let p_color = if is_lead {
                Some((71.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0))
            } else {
                Some((51.0 / 255.0, 65.0 / 255.0, 85.0 / 255.0))
            };

            let mut runs = Vec::new();
            for child in &el.children {
                collect_styled_runs(child, el_font_family, false, false, &mut runs);
            }
            render_styled_paragraph(&runs, p_font_size, align, p_color, false, ctx);
            *ctx.cursor_y -= 3.5;
            true
        }
        "blockquote" => {
            *ctx.cursor_y -= 3.0;
            let bq_start_y = *ctx.cursor_y;
            let mut runs = Vec::new();
            for child in &el.children {
                collect_styled_runs(child, el_font_family, false, true, &mut runs);
            }
            let bq_indent = 7.0;
            let bq_color = Some((71.0 / 255.0, 85.0 / 255.0, 105.0 / 255.0));

            let orig_left = ctx.left_margin;
            let orig_width = ctx.content_width;
            ctx.left_margin += bq_indent;
            ctx.content_width -= bq_indent;

            render_styled_paragraph(&runs, 10.0, None, bq_color, false, ctx);

            ctx.left_margin = orig_left;
            ctx.content_width = orig_width;

            let bq_end_y = *ctx.cursor_y + 1.0;
            ctx.current_layer.set_outline_color(Color::Rgb(Rgb::new(
                203.0 / 255.0,
                213.0 / 255.0,
                225.0 / 255.0,
                None,
            )));
            ctx.current_layer.set_outline_thickness(1.2);
            let bar_line = Line {
                points: vec![
                    (Point::new(Mm(ctx.left_margin + 2.0), Mm(bq_end_y)), false),
                    (Point::new(Mm(ctx.left_margin + 2.0), Mm(bq_start_y)), false),
                ],
                is_closed: false,
            };
            ctx.current_layer.add_line(bar_line);
            *ctx.cursor_y -= 3.5;
            true
        }
        _ => false,
    }
}
