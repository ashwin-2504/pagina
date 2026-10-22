use super::context::PdfRenderContext;
use super::drawing::{draw_image_on_layer, load_image_bytes};
use crate::ast::ElementNode;

pub fn render_image(el: &ElementNode, ctx: &mut PdfRenderContext) {
    let src = match el.attributes.get("src") {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };

    let bytes = match load_image_bytes(src) {
        Some(b) => b,
        None => return,
    };

    let dyn_img = match ::image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => return,
    };

    let px_w = dyn_img.width() as f32;
    let px_h = dyn_img.height() as f32;
    if px_w <= 0.0 || px_h <= 0.0 {
        return;
    }

    let aspect = px_h / px_w;
    let mut target_w = ctx.content_width;
    if let Some(w_str) = el
        .attributes
        .get("width")
        .or_else(|| el.inline_styles.get("width"))
    {
        if let Ok(w) = w_str
            .trim_end_matches("mm")
            .trim_end_matches("px")
            .parse::<f32>()
        {
            if w > 0.0 {
                target_w = w.min(ctx.content_width);
            }
        }
    }
    let mut target_h = target_w * aspect;
    if let Some(h_str) = el
        .attributes
        .get("height")
        .or_else(|| el.inline_styles.get("max-height"))
    {
        if let Ok(h) = h_str
            .trim_end_matches("mm")
            .trim_end_matches("px")
            .parse::<f32>()
        {
            if h > 0.0 && target_h > h {
                target_h = h;
                target_w = target_h / aspect;
            }
        }
    }

    let max_avail_h = ctx.height_mm - ctx.top_margin - ctx.bottom_margin;
    if target_h > max_avail_h {
        target_h = max_avail_h;
        target_w = target_h / aspect;
    }

    ctx.check_page_overflow(target_h);

    let align = el
        .attributes
        .get("align")
        .map(|s| s.as_str())
        .unwrap_or("center");
    let img_x = match align {
        "left" => ctx.left_margin,
        "right" => ctx.left_margin + ctx.content_width - target_w,
        _ => ctx.left_margin + (ctx.content_width - target_w) / 2.0,
    };

    let img_bottom_y = *ctx.cursor_y - target_h;
    draw_image_on_layer(
        ctx.current_layer,
        &dyn_img,
        img_x,
        img_bottom_y,
        target_w,
        target_h,
    );

    *ctx.cursor_y = img_bottom_y - 4.0;
}
