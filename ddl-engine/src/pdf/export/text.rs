use super::context::PdfRenderContext;
use super::drawing::draw_rounded_rect;
use super::fonts::FontFamily;
use super::layout::layout_paragraph_lines;
use super::types::StyledRun;
use printpdf::path::PaintMode;
use printpdf::{Color, IndirectFontRef, Mm, Rgb};

pub fn render_heading(
    text: &str,
    font_size: f32,
    align: Option<&'static str>,
    color: Option<(f32, f32, f32)>,
    font: &IndirectFontRef,
    ctx: &mut PdfRenderContext,
) {
    let line_height_mm = font_size * 0.45;
    ctx.check_page_overflow(line_height_mm);

    if let Some((r, g, b)) = color {
        ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));
    }

    let mut x = ctx.left_margin;
    if align == Some("center") {
        let text_width_approx = (text.len() as f32) * font_size * 0.22;
        x = (ctx.left_margin + (ctx.content_width - text_width_approx) / 2.0).max(ctx.left_margin);
    } else if align == Some("right") {
        let text_width_approx = (text.len() as f32) * font_size * 0.22;
        x = (ctx.left_margin + ctx.content_width - text_width_approx).max(ctx.left_margin);
    }

    ctx.current_layer.use_text(text, font_size, Mm(x), Mm(*ctx.cursor_y), font);
    *ctx.cursor_y -= line_height_mm;

    ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}

pub fn render_styled_paragraph(
    runs: &[StyledRun],
    font_size: f32,
    align: Option<&'static str>,
    color: Option<(f32, f32, f32)>,
    is_callout: bool,
    ctx: &mut PdfRenderContext,
) {
    let line_height_mm = font_size * 0.44;
    let actual_left = if is_callout {
        ctx.left_margin + 6.0
    } else {
        ctx.left_margin
    };
    let actual_width = if is_callout {
        ctx.content_width - 12.0
    } else {
        ctx.content_width
    };

    let rendered_lines = layout_paragraph_lines(runs, font_size, actual_width);
    if rendered_lines.is_empty() {
        return;
    }

    let total_paragraph_height = (rendered_lines.len() as f32) * line_height_mm;
    ctx.check_page_overflow(total_paragraph_height);

    if is_callout {
        let callout_top_y = *ctx.cursor_y + 3.0;
        let callout_bottom_y = *ctx.cursor_y - total_paragraph_height - 2.5;
        let callout_h = callout_top_y - callout_bottom_y;

        ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
            240.0 / 255.0,
            253.0 / 255.0,
            244.0 / 255.0,
            None,
        )));
        draw_rounded_rect(
            ctx.current_layer,
            ctx.left_margin,
            callout_bottom_y,
            ctx.content_width,
            callout_h,
            1.8,
            PaintMode::Fill,
        );

        ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
            22.0 / 255.0,
            163.0 / 255.0,
            74.0 / 255.0,
            None,
        )));
        draw_rounded_rect(
            ctx.current_layer,
            ctx.left_margin,
            callout_bottom_y,
            2.2,
            callout_h,
            1.0,
            PaintMode::Fill,
        );
    }

    for (spans, line_width) in rendered_lines {
        let mut x = actual_left;
        if align == Some("center") {
            x = (actual_left + (actual_width - line_width) / 2.0).max(actual_left);
        } else if align == Some("right") {
            x = (actual_left + actual_width - line_width).max(actual_left);
        }

        for span in &spans {
            let font = ctx.fonts.select(span.family, span.bold, span.italic);

            if span.family == FontFamily::Monospace {
                let pill_pad_x = 0.8;
                let pill_pad_y = 0.4;
                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    241.0 / 255.0,
                    245.0 / 255.0,
                    249.0 / 255.0,
                    None,
                )));
                let pill_x = x - pill_pad_x;
                let pill_y = *ctx.cursor_y - pill_pad_y;
                let pill_w = span.width_mm + (pill_pad_x * 2.0);
                let pill_h = span.font_size * 0.35 + (pill_pad_y * 2.0);
                draw_rounded_rect(
                    ctx.current_layer,
                    pill_x,
                    pill_y,
                    pill_w,
                    pill_h,
                    0.8,
                    PaintMode::Fill,
                );
            }

            if span.family == FontFamily::Monospace {
                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
            } else if let Some((r, g, b)) = color {
                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));
            } else {
                ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
                    15.0 / 255.0,
                    23.0 / 255.0,
                    42.0 / 255.0,
                    None,
                )));
            }

            ctx.current_layer.use_text(&span.text, span.font_size, Mm(x), Mm(*ctx.cursor_y), font);
            x += span.width_mm;
        }

        *ctx.cursor_y -= line_height_mm;
    }

    ctx.current_layer.set_fill_color(Color::Rgb(Rgb::new(
        15.0 / 255.0,
        23.0 / 255.0,
        42.0 / 255.0,
        None,
    )));
}
